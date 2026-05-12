use std::io::{self, Read, Write};
use std::net::{SocketAddr, TcpListener, TcpStream};

#[derive(Debug)]
pub struct TcpServer {
    listener: TcpListener,
}

impl TcpServer {
    pub fn new(listener: TcpListener) -> Self {
        Self { listener }
    }

    pub fn local_addr(&self) -> Result<SocketAddr, io::Error> {
        self.listener.local_addr()
    }

    // FIXME: serial connection handling — the outer `for stream in incoming()` plus the inner
    // per-stream `loop` means only one client is serviced at a time; subsequent accept()s block
    // until the active stream EOFs. Acceptable for single-bot dev, not for a swarm. Revisit with
    // per-connection threads (Arc<Mutex<...>> on shared state) or async I/O.
    pub fn start<'a, T, R, E, F, const N: usize>(&self, mut handler: F) -> Result<(), io::Error>
    where
        E: Into<io::Error>,
        R: Into<&'a [u8]>,
        F: FnMut(SocketAddr, T) -> Result<R, E>,
        T: TryFrom<[u8; N], Error = io::Error> + Into<[u8; N]>,
    {
        for stream in self.listener.incoming() {
            let mut stream_ok = stream?;
            let from = stream_ok.peer_addr()?;

            loop {
                let mut message = [0u8; N];
                if let Err(e) = stream_ok.read_exact(&mut message) {
                    if e.kind() != io::ErrorKind::UnexpectedEof {
                        let _ = stream_ok.write(e.to_string().as_bytes());
                    }
                    break;
                }

                let body: T = match message.try_into() {
                    Ok(b) => b,
                    Err(e) => {
                        let _ = stream_ok.write(e.to_string().as_bytes());
                        break;
                    }
                };

                match handler(from, body) {
                    Ok(res) => stream_ok.write(res.into())?,
                    Err(_) => stream_ok.write("Error".as_bytes())?,
                };
            }
        }

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::{Read, Write};
    use std::sync::mpsc;
    use std::thread;
    use std::time::Duration;

    // Minimal test message type: wraps a single byte.
    // TryFrom fails on 0xFF to exercise parse-error path.
    #[derive(Debug, Clone, Copy, PartialEq)]
    struct Msg(u8);

    impl TryFrom<[u8; 1]> for Msg {
        type Error = io::Error;
        fn try_from(b: [u8; 1]) -> Result<Self, Self::Error> {
            if b[0] == 0xFF {
                Err(io::Error::new(io::ErrorKind::InvalidData, "bad byte"))
            } else {
                Ok(Msg(b[0]))
            }
        }
    }

    impl Into<[u8; 1]> for Msg {
        fn into(self) -> [u8; 1] {
            [self.0]
        }
    }

    fn send_recv(addr: SocketAddr, send: &[u8], recv_len: usize) -> Vec<u8> {
        let mut client = TcpStream::connect(addr).unwrap();
        client.write_all(send).unwrap();
        // signal done writing so server sees EOF on its end if it reads more
        client.shutdown(std::net::Shutdown::Write).unwrap();
        let mut buf = vec![0u8; recv_len];
        client.read_exact(&mut buf).unwrap();
        buf
    }

    #[test]
    fn new_stores_listener() {
        let listener = TcpListener::bind("127.0.0.1:0").unwrap();
        let expected = listener.local_addr().unwrap();
        let server = TcpServer::new(listener);
        assert_eq!(server.listener.local_addr().unwrap(), expected);
    }

    #[test]
    fn start_returns_error_on_accept_failure() {
        let listener = TcpListener::bind("127.0.0.1:0").unwrap();
        listener.set_nonblocking(true).unwrap();
        let server = TcpServer::new(listener);
        let result = server.start(|_addr: SocketAddr, _msg: Msg| Ok::<&'static [u8], io::Error>(b""));
        assert!(result.is_err());
        assert_eq!(result.unwrap_err().kind(), io::ErrorKind::WouldBlock);
    }

    #[test]
    fn start_parses_message_and_passes_to_handler() {
        let listener = TcpListener::bind("127.0.0.1:0").unwrap();
        let addr = listener.local_addr().unwrap();
        let server = TcpServer::new(listener);
        let (tx, rx) = mpsc::channel();

        thread::spawn(move || {
            server
                .start(move |_addr, msg: Msg| {
                    tx.send(msg).ok();
                    Ok::<&'static [u8], io::Error>(b".")
                })
                .ok();
        });

        send_recv(addr, &[42u8], 1);
        let received = rx.recv_timeout(Duration::from_secs(1))
            .expect("handler not called within 1 second");
        assert_eq!(received, Msg(42));
    }

    #[test]
    fn start_sends_handler_response_to_client() {
        let listener = TcpListener::bind("127.0.0.1:0").unwrap();
        let addr = listener.local_addr().unwrap();
        let server = TcpServer::new(listener);

        thread::spawn(move || {
            server
                .start(|_addr, _msg: Msg| Ok::<&'static [u8], io::Error>(b"OK"))
                .ok();
        });

        let response = send_recv(addr, &[1u8], 2);
        assert_eq!(response, b"OK");
    }

    #[test]
    fn start_sends_error_bytes_on_handler_error() {
        let listener = TcpListener::bind("127.0.0.1:0").unwrap();
        let addr = listener.local_addr().unwrap();
        let server = TcpServer::new(listener);

        thread::spawn(move || {
            server
                .start(|_addr, _msg: Msg| {
                    Err::<&'static [u8], io::Error>(io::Error::new(
                        io::ErrorKind::Other,
                        "handler failed",
                    ))
                })
                .ok();
        });

        let response = send_recv(addr, &[1u8], 5);
        assert_eq!(response, b"Error");
    }

    #[test]
    fn start_survives_parse_failure_and_continues() {
        let listener = TcpListener::bind("127.0.0.1:0").unwrap();
        let addr = listener.local_addr().unwrap();
        let server = TcpServer::new(listener);
        let (tx, rx) = mpsc::channel();

        thread::spawn(move || {
            server
                .start(move |_addr, _msg: Msg| {
                    tx.send(()).ok();
                    Ok::<&'static [u8], io::Error>(b".")
                })
                .ok();
        });

        // 0xFF triggers parse error — server should recover, not exit
        let mut bad = TcpStream::connect(addr).unwrap();
        bad.write_all(&[0xFFu8]).unwrap();
        drop(bad);

        // valid connection after bad one — handler must still be called
        send_recv(addr, &[1u8], 1);
        rx.recv_timeout(Duration::from_secs(1))
            .expect("server died on parse failure — handler not called on next connection");
    }

    #[test]
    fn start_sends_parse_error_response_to_client() {
        let listener = TcpListener::bind("127.0.0.1:0").unwrap();
        let addr = listener.local_addr().unwrap();
        let server = TcpServer::new(listener);

        thread::spawn(move || {
            server
                .start(|_addr, _msg: Msg| Ok::<&'static [u8], io::Error>(b"."))
                .ok();
        });

        let mut client = TcpStream::connect(addr).unwrap();
        client.write_all(&[0xFFu8]).unwrap();
        client.shutdown(std::net::Shutdown::Write).unwrap();

        let mut response = Vec::new();
        client.read_to_end(&mut response).unwrap();

        assert!(!response.is_empty(), "expected error response, got nothing");
        let msg = String::from_utf8_lossy(&response);
        // response should describe the parse failure ("bad byte" is the error from Msg::try_from)
        assert!(
            msg.contains("bad byte"),
            "expected parse error description in response, got: {msg:?}"
        );
    }

    #[test]
    fn start_calls_handler_for_each_connection() {
        let listener = TcpListener::bind("127.0.0.1:0").unwrap();
        let addr = listener.local_addr().unwrap();
        let server = TcpServer::new(listener);
        let (tx, rx) = mpsc::channel();

        thread::spawn(move || {
            server
                .start(move |_addr, msg: Msg| {
                    tx.send(msg).ok();
                    Ok::<&'static [u8], io::Error>(b".")
                })
                .ok();
        });

        for i in 0u8..3 {
            send_recv(addr, &[i], 1);
            let received = rx.recv_timeout(Duration::from_secs(1))
                .expect("handler not called within 1 second");
            assert_eq!(received, Msg(i));
        }
    }

    #[test]
    fn start_passes_peer_addr_to_handler() {
        let listener = TcpListener::bind("127.0.0.1:0").unwrap();
        let addr = listener.local_addr().unwrap();
        let server = TcpServer::new(listener);
        let (tx, rx) = mpsc::channel();

        thread::spawn(move || {
            server
                .start(move |peer, _msg: Msg| {
                    tx.send(peer).ok();
                    Ok::<&'static [u8], io::Error>(b".")
                })
                .ok();
        });

        let mut client = TcpStream::connect(addr).unwrap();
        let client_addr = client.local_addr().unwrap();
        // must send a byte so server can read_exact and reach the handler
        client.write_all(&[1u8]).unwrap();
        client.shutdown(std::net::Shutdown::Write).unwrap();

        let received_peer = rx.recv_timeout(Duration::from_secs(1))
            .expect("handler not called within 1 second");
        assert_eq!(received_peer, client_addr);
    }
}
