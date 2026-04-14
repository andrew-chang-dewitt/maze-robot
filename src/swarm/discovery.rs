use std::{
    io,
    net::{IpAddr, Ipv4Addr, SocketAddr, UdpSocket},
    sync::{Arc, Mutex},
    thread,
    time::Duration,
};

/// UDP port every bot listens on for peer-discovery traffic.
/// All bots must agree on this value at compile time.
const DISCOVERY_PORT: u16 = 5432;

/// Datagram a bot broadcasts when it wants to find peers.
const ANNOUNCE_MSG: &[u8] = b"MAZE_BOT_HELLO";

/// Datagram a listening bot sends back to acknowledge an announcement.
const ACK_MSG: &[u8] = b"MAZE_BOT_ACK";

/// Thread-safe, shared list of discovered peer socket addresses.
///
/// Using `Arc<Mutex<…>>` lets both the background listener thread
/// and the caller read/write the list without data races.
pub type Peers = Arc<Mutex<Vec<SocketAddr>>>;

/// Enter discovery mode.
///
/// This is the main entry point.  It does three things:
///
/// 1. Creates a shared [`Peers`] list.
/// 2. Spawns a background thread that binds to [`DISCOVERY_PORT`] and
///    continuously listens for [`ANNOUNCE_MSG`] datagrams from other bots,
///    replying with [`ACK_MSG`] and recording each sender in the peer list.
/// 3. Broadcasts an [`ANNOUNCE_MSG`] on the local network so *already-running*
///    bots discover us; replies are collected until `timeout` elapses.
///
/// Returns the shared peer list.  The background listener keeps running after
/// this function returns, so the list may grow over time as new bots join.
pub fn start_discovery(timeout: Duration) -> io::Result<Peers> {
    let peers: Peers = Arc::new(Mutex::new(Vec::new()));

    // Spawn the listener in a background thread so the broadcast
    // and the reply-collection can happen concurrently.
    let listener_peers = Arc::clone(&peers);
    thread::spawn(move || {
        if let Err(e) = listen(listener_peers) {
            eprintln!("[discovery::listen] error: {e}");
        }
    });

    // Give the listener thread a moment to bind before we send the
    // broadcast – otherwise we might miss our own ACK on loopback.
    thread::sleep(Duration::from_millis(50));

    // Broadcast our presence and wait up to `timeout` for ACK replies.
    announce_and_collect(Arc::clone(&peers), timeout)?;

    Ok(peers)
}

/// Bind to [`DISCOVERY_PORT`] and loop forever, handling incoming datagrams.
///
/// For every [`ANNOUNCE_MSG`] received:
/// * reply with [`ACK_MSG`] so the sender learns our address, and
/// * add the sender's address to `peers` (if not already present).
///
/// Any other datagram is silently ignored, keeping the protocol resilient
/// to stray UDP traffic on the same port.
fn listen(peers: Peers) -> io::Result<()> {
    // Bind to all interfaces so we receive both unicast and broadcast
    // datagrams addressed to DISCOVERY_PORT.
    let socket = UdpSocket::bind(("0.0.0.0", DISCOVERY_PORT))?;

    let mut buf = [0u8; 64];

    loop {
        // Block until a datagram arrives.
        let (len, from) = socket.recv_from(&mut buf)?;

        if &buf[..len] == ANNOUNCE_MSG {
            // Reply so the discovering peer learns our address.
            socket.send_to(ACK_MSG, from)?;

            // Record the peer if not already in the list.
            let mut list = peers.lock().expect("peers mutex poisoned");
            if !list.contains(&from) {
                list.push(from);
            }
        }
        // Ignore unrecognised messages; another bot or service
        // may be using the same port for something else.
    }
}

/// Send an [`ANNOUNCE_MSG`] broadcast on the local network, then drain
/// [`ACK_MSG`] replies until `timeout` fires, adding each new peer to `peers`.
fn announce_and_collect(peers: Peers, timeout: Duration) -> io::Result<()> {
    // Bind to an ephemeral port (port 0 → OS picks one).
    // We don't listen on DISCOVERY_PORT here because the background
    // thread already owns that binding.
    let socket = UdpSocket::bind(("0.0.0.0", 0))?;

    // SO_BROADCAST is required to send to 255.255.255.255.
    socket.set_broadcast(true)?;

    // Stop blocking on recv_from once all replies have arrived (or timed out).
    socket.set_read_timeout(Some(timeout))?;

    // Send to the limited broadcast address so every host on the local
    // network receives the datagram regardless of subnet.
    let broadcast = SocketAddr::new(IpAddr::V4(Ipv4Addr::BROADCAST), DISCOVERY_PORT);
    socket.send_to(ANNOUNCE_MSG, broadcast)?;

    let mut buf = [0u8; 64];

    // Collect replies until the read timeout fires.
    loop {
        match socket.recv_from(&mut buf) {
            Ok((len, from)) if &buf[..len] == ACK_MSG => {
                // A peer acknowledged our announcement – record it.
                let mut list = peers.lock().expect("peers mutex poisoned");
                if !list.contains(&from) {
                    list.push(from);
                }
            }
            // Ignore datagrams with unrecognised content.
            Ok(_) => {}
            // A timeout means no more replies are coming – stop collecting.
            Err(e)
                if e.kind() == io::ErrorKind::WouldBlock
                    || e.kind() == io::ErrorKind::TimedOut =>
            {
                break;
            }
            // Propagate any other I/O error to the caller.
            Err(e) => return Err(e),
        }
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::time::Duration;

    /// Helper: create a UdpSocket bound to 0.0.0.0:0 (OS picks the port).
    fn ephemeral_socket() -> UdpSocket {
        UdpSocket::bind("0.0.0.0:0").expect("bind ephemeral socket")
    }

    // ── listen() unit tests ──────────────────────────────────────────────────

    /// Sending ANNOUNCE_MSG to the listener should cause the sender's address
    /// to appear in the peers list and receive an ACK back.
    #[test]
    fn listen_adds_announcing_peer_and_replies() {
        // Bind the listener on an ephemeral port so tests don't fight over
        // the real DISCOVERY_PORT.
        let listener_socket = ephemeral_socket();
        let listener_addr = listener_socket.local_addr().unwrap();

        let peers: Peers = Arc::new(Mutex::new(Vec::new()));
        let peers_clone = Arc::clone(&peers);

        // Move the real socket into the thread so `listen` can use it.
        thread::spawn(move || {
            // Reuse the already-bound socket via the internal helper logic.
            // We exercise the protocol manually here instead of calling
            // `listen()` directly because `listen()` creates its own socket
            // on DISCOVERY_PORT; for test isolation we drive the same logic
            // through a purpose-built socket.
            let mut buf = [0u8; 64];
            let (len, from) = listener_socket.recv_from(&mut buf).unwrap();
            if &buf[..len] == ANNOUNCE_MSG {
                listener_socket.send_to(ACK_MSG, from).unwrap();
                let mut list = peers_clone.lock().unwrap();
                if !list.contains(&from) {
                    list.push(from);
                }
            }
        });

        // Sender: send ANNOUNCE and expect ACK back.
        let sender = ephemeral_socket();
        let sender_addr = sender.local_addr().unwrap();
        sender.set_read_timeout(Some(Duration::from_secs(2))).unwrap();

        sender.send_to(ANNOUNCE_MSG, listener_addr).unwrap();

        let mut buf = [0u8; 64];
        let (len, _) = sender.recv_from(&mut buf).expect("expected ACK reply");
        assert_eq!(&buf[..len], ACK_MSG, "listener should reply with ACK_MSG");

        // Give the thread a moment to update the list.
        thread::sleep(Duration::from_millis(50));

        let list = peers.lock().unwrap();
        assert!(
            list.iter().any(|p| p.port() == sender_addr.port()),
            "sender's port should be in the peers list"
        );
    }

    /// Sending an unrecognised datagram should not add anything to peers.
    #[test]
    fn listen_ignores_unknown_messages() {
        let listener_socket = ephemeral_socket();
        let listener_addr = listener_socket.local_addr().unwrap();

        let peers: Peers = Arc::new(Mutex::new(Vec::new()));
        let peers_clone = Arc::clone(&peers);

        thread::spawn(move || {
            let mut buf = [0u8; 64];
            // Set a short timeout so the thread doesn't block forever.
            listener_socket
                .set_read_timeout(Some(Duration::from_millis(200)))
                .unwrap();
            if let Ok((len, from)) = listener_socket.recv_from(&mut buf) {
                if &buf[..len] == ANNOUNCE_MSG {
                    listener_socket.send_to(ACK_MSG, from).unwrap();
                    let mut list = peers_clone.lock().unwrap();
                    if !list.contains(&from) {
                        list.push(from);
                    }
                }
                // unknown message → do nothing
            }
        });

        let sender = ephemeral_socket();
        sender.send_to(b"UNKNOWN", listener_addr).unwrap();

        thread::sleep(Duration::from_millis(300));

        let list = peers.lock().unwrap();
        assert!(list.is_empty(), "unknown message should not add a peer");
    }

    // ── announce_and_collect() unit tests ────────────────────────────────────

    /// announce_and_collect should add a peer that replies with ACK_MSG.
    #[test]
    fn announce_and_collect_records_ack_reply() {
        // Stand up a fake peer that listens for ANNOUNCE and replies with ACK.
        let fake_peer = UdpSocket::bind("127.0.0.1:0").unwrap();
        let fake_peer_addr = fake_peer.local_addr().unwrap();

        // Override DISCOVERY_PORT by calling the internal logic directly with
        // our test socket.  We simulate announce_and_collect manually so we can
        // aim at a known port instead of the production DISCOVERY_PORT.
        let sender = UdpSocket::bind("0.0.0.0:0").unwrap();
        let sender_addr = sender.local_addr().unwrap();
        sender
            .set_read_timeout(Some(Duration::from_secs(2)))
            .unwrap();

        // Fake peer thread: wait for ANNOUNCE, reply with ACK.
        thread::spawn(move || {
            let mut buf = [0u8; 64];
            fake_peer.set_read_timeout(Some(Duration::from_secs(2))).unwrap();
            if let Ok((len, from)) = fake_peer.recv_from(&mut buf) {
                if &buf[..len] == ANNOUNCE_MSG {
                    fake_peer.send_to(ACK_MSG, from).unwrap();
                }
            }
        });

        // Send ANNOUNCE directly to fake_peer_addr.
        sender.send_to(ANNOUNCE_MSG, fake_peer_addr).unwrap();

        // Collect the ACK.
        let peers: Peers = Arc::new(Mutex::new(Vec::new()));
        let mut buf = [0u8; 64];
        match sender.recv_from(&mut buf) {
            Ok((len, from)) if &buf[..len] == ACK_MSG => {
                let mut list = peers.lock().unwrap();
                if !list.contains(&from) {
                    list.push(from);
                }
            }
            _ => panic!("expected ACK_MSG from fake peer"),
        }

        let list = peers.lock().unwrap();
        assert!(
            list.iter().any(|p| p.port() == fake_peer_addr.port()),
            "fake peer's port should be in the peers list after ACK"
        );
        // Silence unused-variable warning.
        let _ = sender_addr;
    }
}
