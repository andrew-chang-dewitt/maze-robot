//! implementation of peer discovery for building swarms of bots

use anyhow::{self, Context};

use std::{
    io,
    net::{Ipv4Addr, SocketAddr, UdpSocket},
    sync::{Arc, Barrier, Mutex},
    thread,
    time::Duration,
};

// ---- CONFIGURATION -------------------------------------
// TODO: make configurable by building from some Config obj?
/// HELLO datagram
const ANNOUNCE: &[u8] = b"MAZE_BOT_HELLO";
/// ACK datagram
const ACKNOWLEDGE: &[u8] = b"MAZE_BOT_ACK";

/// collection of peers. thread-safe & growable as more peers are discovered.
pub type Peers = Arc<Mutex<Vec<SocketAddr>>>;

/// main discovery function. listens for new peers & announces to existing peers on separate
/// sockets bound in separate threads, each updating the shared [`Peers`] collection. returns a
/// reference to the collection.
pub fn start(timeout: Duration, port: u16) -> anyhow::Result<Peers> {
    // init peers collection
    let peers: Peers = Arc::new(Mutex::new(Vec::new()));
    // set signal indicating that discovery is active
    let is_discovering = Arc::new(Mutex::new(true));
    // set signal to know when listener thread is ready
    let ready = Arc::new(Barrier::new(2));

    // spawn listener
    let peers_copy = Arc::clone(&peers);
    let listen_socket = UdpSocket::bind(("0.0.0.0", port))?;
    let ready_copy = Arc::clone(&ready);
    let is_discovering_copy = Arc::clone(&is_discovering);
    thread::spawn(move || listen(listen_socket, ready_copy, is_discovering_copy, peers_copy));

    // wait for listener to be ready
    // FIXME: might not be necessary as we don't need to add this bot's address to peers list...
    ready.wait();

    // announce this bot to network
    announce(port, peers.clone(), Duration::from_secs(30))?;

    // return ref to peers collection -- it will continue to receive updates as new peers are
    // discovered
    Ok(peers)
}

/// listen on given port for incoming messages
///
/// whenever a new peer announces its presence (via [`ANNOUNCE`]), this acknowledges it with an
/// [`ACKNOWLEDGE`] reply & adds the new peer to the [`Peers`] collection.
///
/// silently ignores all other messages
fn listen(
    socket: UdpSocket,
    ready: Arc<Barrier>,
    is_discovering: Arc<Mutex<bool>>,
    peers: Peers,
) -> anyhow::Result<()> {
    // buffer for incoming messages
    let mut buf = [0u8; 64];

    // let spawning thread know listener is ready
    ready.wait();

    // set socket recv timeout to stop from blocking indefinitely
    socket.set_read_timeout(Some(Duration::from_millis(10)));
    // when discovery ends
    // listen for new peer announcements until discovery ends
    while *is_discovering
        .lock()
        .expect("discovery flag mutex poisoned!")
    {
        // get messages on listen socket
        let (len, from) = socket.recv_from(&mut buf)?;
        // & when new peer announced
        if &buf[..len] == ANNOUNCE {
            // aknowledge new peer
            socket.send_to(ACKNOWLEDGE, from)?;
            // then save peer in collection
            let mut peers_local = peers.lock().expect("peers collection mutex poisoned!");
            if !peers_local.contains(&from) {
                peers_local.push(from);
            }
        }
    }

    Ok(())
}

/// announce bot to network to join swarm (if it exists already)
///
/// broadcasts [`ANNOUNCE`] to local network, then listens for [`ACKNOWLEDGE`] responses & adds
/// each responder to the [`Peers`] collection until the given timeout is reached.
fn announce(discovery_port: u16, peers: Peers, timeout: Duration) -> anyhow::Result<()> {
    // setup arbitrary port to broacast new peer announcement
    let socket = UdpSocket::bind("0.0.0.0:0")?;
    socket.set_broadcast(true)?;
    socket.set_read_timeout(Some(timeout))?;
    // get broadcast address
    let broadcast_addr = SocketAddr::new(std::net::IpAddr::V4(Ipv4Addr::BROADCAST), discovery_port);

    // announce!
    socket.send_to(ANNOUNCE, broadcast_addr)?;

    // listen for ACKNOWLEDGE responses
    let mut buf = [0u8; 64];
    loop {
        match socket.recv_from(&mut buf) {
            // add responders to peers
            Ok((len, from)) if &buf[..len] == ACKNOWLEDGE => {
                println!(
                    "[announce] received '{}' from peer @ {from}",
                    String::from_utf8((&buf[..len]).to_vec()).expect("message should be decodable")
                );
                let mut peers_local = peers.lock().expect("peers collection mutex poisoned!");
                if !peers_local.contains(&from) {
                    peers_local.push(from);
                }
            }
            // ignore other messages
            Ok(_) => (),
            // end listening loop uneventfully at timeout
            Err(e)
                if e.kind() == io::ErrorKind::WouldBlock || e.kind() == io::ErrorKind::TimedOut =>
            {
                break;
            }
            // otherwise, exit announce w/ Err
            Err(e) => {
                return Err(anyhow::anyhow!(e))
                    .context("Error caught while listening for ACKNOWLEDGE after announcing");
            }
        }
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use std::net::UdpSocket;

    use local_ip_address::local_ip;

    use super::*;

    /// listen worker aknowledges received announcements from new peers & adds new peers to
    /// local [`Peer`] collection
    #[test]
    fn listen_adds_announcing_peer() {
        // config listener
        // at some socket
        let listen_socket = UdpSocket::bind("0.0.0.0:0").expect("listener should be bound");
        let listen_addr = listen_socket.local_addr().expect("listener has address");
        // w/ barrier to notify listener is ready
        let ready = Arc::new(Barrier::new(2));
        let ready_copy = ready.clone();
        // in discovery mode
        let is_discovering = Arc::new(Mutex::new(true));
        let is_discovering_copy = is_discovering.clone();
        // with peers
        let peers: Peers = Arc::new(Mutex::new(Vec::new()));
        let peers_copy = Arc::clone(&peers);

        // init listener in thread
        thread::Builder::new()
            .name("test_listener".to_string())
            .spawn(move || {
                // TODO: add some sort of logging/error handling?
                listen(listen_socket, ready_copy, is_discovering_copy, peers_copy)
            })
            .expect("listener thread should spawn");

        // config a mock peer
        let send_socket = UdpSocket::bind("0.0.0.0:0").expect("sender should be bound");
        send_socket
            .set_read_timeout(Some(Duration::from_secs(2)))
            .expect("send socket read timeout should be set");

        // wait for listener to be ready
        ready.wait();
        // announce mock peer
        send_socket
            .send_to(ANNOUNCE, listen_addr)
            .expect("sender should announce");
        let send_addr = send_socket.local_addr().expect("sender has address");

        // shut down listener thread
        *is_discovering
            .lock()
            .expect("discovery flag mutex poisoned!") = false;

        // ASSERT sender receives ACKNOWLEDGE
        let mut buf = [0u8; 64];
        let (len, _) = send_socket
            .recv_from(&mut buf)
            .expect("listener should acknowledge");
        assert_eq!(
            &buf[..len],
            ACKNOWLEDGE,
            "listener should reply with `ACKNOWLEDGE`"
        );

        // ASSERT peers updated w/ sender address
        let list = peers.lock().expect("peers should unlock");
        assert!(
            list.iter()
                .any(|addr| addr.ip().is_loopback() && addr.port() == send_addr.port()),
            "peers list must contain sending socket address!\n  peers list: {list:#?}\n  expected address: {send_addr}",
        );
    }

    /// announcing should also add hosts that acknowledge the annoument to the peers collection
    #[test]
    fn announce_adds_peers_that_acknowledge() {
        // setup mock peer already listening
        let listening_peer =
            UdpSocket::bind("0.0.0.0:0").expect("mock listener socket should bind");
        let listening_addr = listening_peer
            .local_addr()
            .expect("listening peer should have address");
        let listening_port = listening_addr.port();
        println!(
            "#test[announce_adds_peers_that_acknowledge] mock listener has addr: {listening_addr:#?}"
        );
        // w/ barrier to notify listener is ready
        let ready = Arc::new(Barrier::new(2));
        let ready_copy = ready.clone();

        // start mock peer in thread, ready to acknowledge received announcements
        thread::Builder::new()
            .name("test_listener".to_string())
            .spawn(move || {
                let mut buf = [0u8; 64];
                listening_peer
                    .set_read_timeout(Some(Duration::from_secs(10)))
                    .expect("mock listener shouldn't wait more than 10 seconds");
                ready_copy.wait();

                if let Ok((len, from)) = listening_peer.recv_from(&mut buf) {
                    if &buf[..len] == ANNOUNCE {
                        listening_peer
                            .send_to(ACKNOWLEDGE, from)
                            .expect("mock peer should acknowledge");
                    }
                }
            })
            .expect("mock listener should spawn in own thread");
        // wait for listener to be ready
        ready.wait();

        // set up shared peers list
        let peers: Peers = Arc::new(Mutex::new(Vec::new()));

        // announce at listening port on network w/ 1 second timeout for ack
        announce(listening_port, peers.clone(), Duration::from_secs(1))
            .expect("announcing should happen w/out incident");

        // ASSERT peers updated w/ sender address
        let list = peers.lock().expect("peers should unlock");
        assert!(
            list.iter()
                .any(|addr| addr.ip() == local_ip().expect("should get host ip")
                    && addr.port() == listening_port),
            "peers list must contain mock listener address!\n  peers list: {list:#?}\n  expected address: {listening_addr}",
        );
    }
}
