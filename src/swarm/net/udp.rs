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
    /// socket used to send messages
    snd_sock: UdpSocket,
}

impl Broadcaster {
    /// send
    pub fn send<'a>(&self, msg: impl Into<String>) -> Result<(), CommsErr> {
        self.snd_sock
            .send_to(msg.into().as_bytes(), self.tgt_addr)?;

        Ok(())
    }
}

pub trait SocketProvider {
    /// get a reference to a UdpSocket
    fn get_sock_ref(&self) -> &UdpSocket;
    /// get socket wrapped in Arc for threadsafe borrowing
    fn get_sock_arc(&self) -> Arc<UdpSocket>;
}

/// Behaviour to listen for messages broadcasted on the local network over UDP. offers methods to
/// get one message or to listen in a loop & call a handler on each message received. requires
/// ability to borrow a UdpSocket
pub trait Listener: SocketProvider {
    /// receive a message & return it. will block until message is received unless Some(Duration)
    /// is given as timeout.
    fn rcv_once(&self, timeout: Option<Duration>) -> Result<(usize, [u8; 64]), CommsErr> {
        let sock = self.get_sock_ref();
        sock.set_read_timeout(timeout)?;

        // listen for message
        let mut buf = [0u8; 64];
        let (len, _) = sock.recv_from(&mut buf)?;

        // clear timeout if one was set
        if timeout.is_some() {
            sock.set_read_timeout(None)?;
        }

        Ok((len, buf))
    }

    /// start listener loop in new thread, executing given handler on each received message
    ///
    /// returns [`StopListenHandle`] if listener start successfully, used to kill listener
    /// thread if needed.
    fn start(
        &self,
        on_msg: impl Fn(usize, [u8; 64], SocketAddr) -> Result<(), CommsErr> + Send + 'static,
    ) -> Result<StopListenHandle, CommsErr> {
        // get copy of socket for passing to thread
        let sock = self.get_sock_arc();
        // set read timeout to keep from blocking kill signal in loop
        sock.set_read_timeout(Some(Duration::from_millis(10)))?;
        // create kill signal & return listener channels
        let (kill_sndr, kill_rcvr) = oneshot::channel();

        thread::spawn(move || {
            let mut rcvr_copy = kill_rcvr;

            loop {
                // check if received kill command
                match rcvr_copy.try_recv() {
                    // end loop if so
                    Ok(_) | Err(oneshot::TryRecvError::Disconnected) => break,
                    // otherwise continue
                    Err(oneshot::TryRecvError::Empty(rx)) => {
                        // restore kill signal receiver
                        rcvr_copy = rx;
                    }
                };
                // attempt to read a message
                let mut buf = [0u8; 64];
                match sock.recv_from(&mut buf) {
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
                    Err(_) => return Err(CommsErr::HandlerError),
                };
            }

            Ok(())
        });

        Ok(StopListenHandle(kill_sndr))
    }
}

pub struct StopListenHandle(oneshot::Sender<()>);

impl StopListenHandle {
    pub fn stop(self) -> Result<(), CommsErr> {
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
    /// address at which to open socket for broadcasting from
    pub snd_addr: Option<SocketAddr>,
}

impl BroadcastBuilder {
    pub fn new() -> Self {
        Self {
            tgt_port: None,
            snd_addr: None,
        }
    }

    pub fn port(self, port: u16) -> Self {
        Self {
            tgt_port: Some(port),
            ..self
        }
    }

    pub fn try_init(self) -> Result<Broadcaster, CommsErr> {
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

        Ok(Broadcaster { tgt_addr, snd_sock })
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

    pub fn try_init_sock(self) -> Result<UdpSocket, CommsErr> {
        // use given address
        let addr = SocketAddr::new(
            self.addr.unwrap_or(IpAddr::V4(Ipv4Addr::LOCALHOST)),
            self.port.ok_or(BuildError::MissingPort)?,
        );
        // then bind socket to this addr
        UdpSocket::bind(addr).map_err(|e| e.into())
    }
}

#[cfg(test)]
mod tests {
    use std::{
        sync::{Arc, Mutex},
        thread::{self, sleep},
    };

    use super::*;

    #[test]
    fn a_scream_into_void_can_be_heard() {
        let msg = "AAAAAAAAA";

        // broadcast port to target
        let port = 6002;
        // configure Broadcaster to be tested
        let broadcaster = BroadcastBuilder::new()
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
        assert_eq!(act, msg.as_bytes());
    }

    struct TestListener(Arc<UdpSocket>);

    impl SocketProvider for TestListener {
        fn get_sock_ref(&self) -> &UdpSocket {
            &self.0
        }

        fn get_sock_arc(&self) -> Arc<UdpSocket> {
            Arc::clone(&self.0)
        }
    }

    impl Listener for TestListener {}

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
        let sock = ListenBuilder::new()
            .addr(addr)
            .port(port)
            .try_init_sock()
            .expect("failed to initialize Listener");
        let listener = TestListener(Arc::new(sock));
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
        let sock = ListenBuilder::new()
            .addr(addr)
            .port(port)
            .try_init_sock()
            .expect("failed to initialize Listener");
        let listener = TestListener(Arc::new(sock));

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
