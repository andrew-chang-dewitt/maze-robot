use std::{
    io,
    net::{IpAddr, Ipv4Addr, SocketAddr, UdpSocket},
    sync::{Arc, oneshot},
    thread,
    time::Duration,
};

#[derive(Debug)]
/// obj encapsulating ability to broadcast messages/data to the local network over UDP.
pub struct Broadcaster {
    /// socket address to broadcast message to
    tgt_addr: SocketAddr,
    /// a prefix to be prepended to every message broadcasted
    tgt_prfx: String,
    /// socket used to send messages
    snd_sock: UdpSocket,
}

impl Broadcaster {
    /// send
    pub fn send<'a>(&self, msg: impl Into<String>) -> Result<(), CommsErr> {
        let buf_str = format!("{}_{}", self.tgt_prfx, msg.into());
        let buf = buf_str.as_bytes();
        self.snd_sock.send_to(buf, self.tgt_addr)?;

        Ok(())
    }
}

#[derive(Debug)]
/// obj encapsulating ability to listen for messages broadcasted on the local network over UDP.
pub struct Listener {
    /// socket to listen on
    sock: Arc<UdpSocket>,
}

impl Listener {
    /// block until a message is received or timeout is reached (defaults to 10 seconds), return
    /// message received on success
    pub fn rcv_once(self, timeout: Option<Duration>) -> Result<(usize, [u8; 64]), CommsErr> {
        // set read timeout, using default if not given
        self.sock
            .set_read_timeout(timeout.or(Some(Duration::from_secs(10))))?;

        // listen for message
        let mut buf = [0u8; 64];
        let (len, _) = self.sock.recv_from(&mut buf)?;

        // clear timeout
        self.sock.set_read_timeout(None)?;

        Ok((len, buf))
    }

    /// start listener loop in new thread, executing given handler on each received message
    ///
    /// returns [`StopListenHandle`] if listener start successfully, used to kill listener
    /// thread if needed.
    pub fn start(
        &mut self,
        on_msg: impl Fn(usize, [u8; 64], SocketAddr) -> Result<(), CommsErr> + Send + 'static,
    ) -> Result<StopListenHandle, CommsErr> {
        // set read timeout, using default if not given
        self.sock
            .set_read_timeout(Some(Duration::from_millis(10)))?;
        // get ref to socket to pass to looped listening thread
        let sock_copy = Arc::clone(&self.sock);
        // create kill signal
        let (sndr, rcvr) = oneshot::channel();

        thread::spawn(move || {
            let mut rcvr_copy = rcvr;

            loop {
                // check if received kill command
                match rcvr_copy.try_recv() {
                    Ok(_) | Err(oneshot::TryRecvError::Disconnected) => break,
                    Err(oneshot::TryRecvError::Empty(rx)) => {
                        // restore kill signal receiver
                        rcvr_copy = rx;
                    }
                };
                let mut buf = [0u8; 64];
                match sock_copy.recv_from(&mut buf) {
                    // call handler when message received successfully
                    Ok((len, from)) => on_msg(len, buf, from),
                    // end loop iter if timed out or blocked; gives chance to check if is_listening flag
                    // has been unset
                    Err(e)
                        if e.kind() == io::ErrorKind::WouldBlock
                            || e.kind() == io::ErrorKind::TimedOut =>
                    {
                        continue;
                    }
                    // otherwise exit in error state
                    Err(e) => return Err(CommsErr::HandlerError),
                };
            }

            Ok(())
        });

        Ok(StopListenHandle(sndr))
    }
}

struct StopListenHandle(oneshot::Sender<()>);

impl StopListenHandle {
    fn stop(self) -> Result<(), CommsErr> {
        self.0.send(()).map_err(|_| CommsErr::ThreadError)
    }
}

#[derive(Debug)]
pub enum CommsErr {
    Io(io::Error),
    Build(BuildError),
    ThreadError,
    HandlerError,
}

impl From<BuildError> for CommsErr {
    fn from(value: BuildError) -> Self {
        Self::Build(value)
    }
}

#[derive(Debug)]
pub struct BroadcastBuilder {
    /// the port number that messages should be broadcast to
    pub tgt_port: Option<u16>,
    /// a prefix to be prepended to every message broadcasted
    pub tgt_prfx: Option<String>,
    /// address at which to open socket for broadcasting from
    pub snd_addr: Option<SocketAddr>,
}

impl BroadcastBuilder {
    pub fn new() -> Self {
        Self {
            tgt_port: None,
            tgt_prfx: None,
            snd_addr: None,
        }
    }

    pub fn prefix(self, prefix: impl Into<String>) -> Self {
        Self {
            tgt_prfx: Some(prefix.into()),
            ..self
        }
    }

    pub fn port(self, port: u16) -> Self {
        Self {
            tgt_port: Some(port),
            ..self
        }
    }

    pub fn try_init(self) -> Result<Broadcaster, CommsErr> {
        // message prefix must be given
        let tgt_prfx = self.tgt_prfx.ok_or(BuildError::MissingPrefix)?;
        // broadcast port must be given
        let tgt_addr = SocketAddr::new(
            IpAddr::V4(Ipv4Addr::BROADCAST),
            self.tgt_port.ok_or(BuildError::MissingPort)?,
        );

        // create send socket
        let snd_sock = self
            // using given socket address
            .snd_addr
            .map(|addr| UdpSocket::bind(addr))
            // or letting the os choose if none provided
            .unwrap_or(UdpSocket::bind("0.0.0.0:0"))?;
        // & enable broadcast mode
        snd_sock.set_broadcast(true)?;

        Ok(Broadcaster {
            tgt_addr,
            tgt_prfx,
            snd_sock,
        })
    }
}

#[derive(Debug)]
pub enum BuildError {
    MissingPort,
    MissingPrefix,
    MissingSocketAddr,
}

impl From<io::Error> for CommsErr {
    fn from(value: io::Error) -> Self {
        Self::Io(value)
    }
}

#[derive(Debug)]
pub struct ListenBuilder {
    port: Option<u16>,
    addr: Option<IpAddr>,
}

impl ListenBuilder {
    pub fn new() -> Self {
        Self {
            port: None,
            addr: None,
        }
    }

    pub fn addr(self, addr: IpAddr) -> Self {
        Self {
            addr: Some(addr),
            ..self
        }
    }

    pub fn port(self, port: u16) -> Self {
        Self {
            port: Some(port),
            ..self
        }
    }

    pub fn try_init(self) -> Result<Listener, CommsErr> {
        // use given address
        let addr = SocketAddr::new(
            self.addr.unwrap_or(IpAddr::V4(Ipv4Addr::LOCALHOST)),
            self.port.ok_or(BuildError::MissingPort)?,
        );
        // then bind socket to this addr
        let sock = UdpSocket::bind(addr)?;

        Ok(Listener {
            sock: Arc::new(sock),
        })
    }
}

#[cfg(test)]
mod tests {
    use std::{
        sync::{Arc, Barrier, Mutex},
        thread::{self, sleep},
    };

    use super::*;

    #[test]
    fn a_scream_into_void_can_be_heard() {
        let prefix = "AAAAAAAAA";
        let msg = "AAAAAAAAA";
        // expected message to be received
        let exp = format!("{}_{}", prefix, msg);

        // broadcast port to target
        let port = 6002;
        // configure Broadcaster to be tested
        let broadcaster = BroadcastBuilder::new()
            .prefix(prefix.to_string())
            .port(port)
            .try_init()
            .expect("failed to initialize Broadcaster for testing");

        // setup mock peer already listening
        let listening_peer =
            UdpSocket::bind(format!("0.0.0.0:{port}")).expect("mock listener socket should bind");

        // start mock peer in thread, ready to acknowledge received announcements
        let listen_t = thread::Builder::new()
            .name("mock_listener".to_string())
            .spawn(move || {
                let mut buf = [0u8; 64];
                listening_peer
                    .set_read_timeout(Some(Duration::from_secs(10)))
                    .expect("timeout unable to be set");

                let (len, _) = listening_peer
                    .recv_from(&mut buf)
                    .expect("unable to receive message");

                (len, buf)
            })
            .expect("mock listener should spawn in own thread");
        // then send broadcast message
        broadcaster.send(msg).expect("message failed to broadcast");

        // collect message received in listener thread
        let (len, rcvd) = listen_t.join().expect("failed to join");
        // trim to received length
        let act = &rcvd[..len];

        // compare to expected message
        assert_eq!(act, exp.as_bytes());
    }

    #[test]
    fn listen_closely_and_you_can_still_hear_the_screams() {
        let prefix = "AAAAAAAAA";
        let msg = "AAAAAAAAA";
        // expected message to be received
        let exp = format!("{}_{}", prefix, msg);

        // address to target
        let addr = IpAddr::V4(Ipv4Addr::LOCALHOST);
        let port = 6003;
        // configure test Listener
        let listener = ListenBuilder::new()
            .addr(addr)
            .port(port)
            .try_init()
            .expect("failed to initialize Listener");
        // start Listener in new thread
        let listen_t = thread::Builder::new()
            .name("test_listener".to_string())
            .spawn(move || {
                listener
                    .rcv_once(Some(Duration::from_secs(1)))
                    .expect("error encountered while listening")
            })
            .expect("mock listener should spawn in own thread");

        // configure mock peer to send test message
        let peer_socket = UdpSocket::bind("0.0.0.0:0").expect("sender should be bound");
        // send test message from mock peer
        peer_socket
            .send_to(format!("{prefix}_{msg}").as_bytes(), (addr, port))
            .expect("failed to send");

        // collect message received in listener thread
        let (len, rcvd) = listen_t.join().expect("failed to join");
        println!("recieved {len} bytes: {rcvd:?}");
        // trim to received length
        let act = &rcvd[..len];

        // compare to expected message
        assert_eq!(act, exp.as_bytes());
    }

    #[test]
    fn many_screams_can_be_heard() {
        // address to target
        let addr = IpAddr::V4(Ipv4Addr::LOCALHOST);
        let port = 6104;

        // configure test Listener
        let mut listener = ListenBuilder::new()
            .addr(addr)
            .port(port)
            .try_init()
            .expect("failed to initialize Listener");

        // vec to store messages received by testing listener
        let received = Arc::new(Mutex::new(Vec::new()));
        let received_copy = received.clone();

        // start Listener
        let listening_handle = listener
            .start(move |len, msg, _| {
                // push each message received to vec
                received_copy
                    .lock()
                    .map_err(|_| CommsErr::HandlerError)?
                    .push((len, msg));
                Ok(())
            })
            .expect("listener failed to start");

        // configure mock peer to send test message
        let peer_socket = UdpSocket::bind("0.0.0.0:0").expect("sender should be bound");
        peer_socket
            .set_read_timeout(Some(Duration::from_secs(2)))
            .expect("send socket read timeout should be set");
        // send test messages from mock peer
        peer_socket
            .send_to(format!("AAAAA").as_bytes(), (addr, port))
            .expect("failed to send");
        peer_socket
            .send_to(format!("BBBBB").as_bytes(), (addr, port))
            .expect("failed to send");
        peer_socket
            .send_to(format!("CCCCC").as_bytes(), (addr, port))
            .expect("failed to send");

        // wait for all messages to be received
        loop {
            sleep(Duration::from_millis(10));
            let rcvd = received.lock().expect("failed to get lock on received vec");
            if rcvd.len() >= 3 {
                // then stop listener
                listening_handle.stop().expect("listener failed to stop");
                break;
            }
        }

        // compare to expected messages
        let act = received.lock().expect("received should be unlocked");
        println!("recieved messages: {act:#?}");
        let exp = vec![
            "AAAAA".to_string(),
            "BBBBB".to_string(),
            "CCCCC".to_string(),
        ];

        for ((len, buf), x) in act.iter().zip(exp) {
            assert_eq!(len, &5);

            let trimmed = &buf[..*len];
            let msg = String::from_utf8(trimmed.to_vec()).expect("failed to decode message");
            assert_eq!(msg, x);
        }
    }
}
