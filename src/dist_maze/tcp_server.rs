use std::io::{self, Read, Write};
use std::net::{SocketAddr, TcpListener, TcpStream};
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{Arc, Mutex};
use std::thread;

use super::thread_pool::ThreadPool;

#[derive(Debug)]
pub struct TcpServer {
    listener: TcpListener,
    running: Arc<AtomicBool>,
    pool: ThreadPool,
}

impl TcpServer {
    pub fn new(listener: TcpListener) -> Self {
        Self {
            listener,
            running: Arc::new(AtomicBool::new(false)),
            pool: ThreadPool::new(
                thread::available_parallelism()
                    .map(|i| i.into())
                    .unwrap_or(2),
            ),
        }
    }

    pub fn local_addr(&self) -> Result<SocketAddr, io::Error> {
        self.listener.local_addr()
    }

    /// Returns a [`StopSignal`] that can be used to shut down the server from another thread.
    ///
    /// Must be called before [`Self::start`] (since the server is typically moved into the thread
    /// that runs `start`, callers lose access after that point).
    pub fn stop_signal(&self) -> Result<StopSignal, io::Error> {
        Ok(StopSignal {
            running: Arc::clone(&self.running),
            addr: self.listener.local_addr()?,
        })
    }

    /// Start listener loop; passing each accepted connection to a worker in the ThreadPool.
    pub fn start<'a, T, R, E, F, const N: usize>(&self, handler: F) -> Result<(), io::Error>
    where
        E: Into<io::Error>,
        R: Into<&'a [u8]>,
        F: FnMut(SocketAddr, T) -> Result<R, E> + Send + Sync + 'static,
        T: TryFrom<[u8; N], Error = io::Error> + Into<[u8; N]>,
    {
        let handler_arc = Arc::new(Mutex::new(handler));
        self.running.store(true, Ordering::SeqCst);

        for stream in self.listener.incoming() {
            // Check flag before doing anything with the new stream so a wake-up self-connect
            // (sent by StopSignal::stop) exits cleanly without entering the read loop.
            if !self.running.load(Ordering::SeqCst) {
                break;
            }

            let mut stream_ok = stream?;
            let from = stream_ok.peer_addr()?;

            let handler_clone = Arc::clone(&handler_arc);

            self.pool.execute(move || {
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

                    match handler_clone.lock().expect("handler to be available")(from, body) {
                        Ok(res) => stream_ok.write(res.into()).expect("response to write"),
                        Err(_) => stream_ok.write("Error".as_bytes()).expect("error to write"),
                    };
                }
            })
        }

        Ok(())
    }
}

/// External handle that signals a running [`TcpServer`] to stop accepting new connections.
///
/// Created by [`TcpServer::stop_signal`] before [`TcpServer::start`] is called. Calling
/// [`StopSignal::stop`] flips the server's running flag and self-connects to unblock the
/// pending `accept()`, allowing the server's outer accept loop to observe the flag and exit.
///
/// Cloneable so the same shutdown trigger can be wired up to multiple sources (e.g. a Ctrl-C
/// handler and a test-teardown step).
#[derive(Debug, Clone)]
pub struct StopSignal {
    running: Arc<AtomicBool>,
    addr: SocketAddr,
}

impl StopSignal {
    /// Request the server to stop accepting new connections.
    ///
    /// Sets the running flag to `false`, then opens a one-shot TCP connection to the server's
    /// bound address to unblock any in-flight `accept()` call. Best-effort — if the self-connect
    /// fails (e.g. listener already torn down), the flag is still set.
    pub fn stop(&self) {
        self.running.store(false, Ordering::SeqCst);
        // Wake up a blocked accept() so the loop body runs once more and sees the flag.
        let _ = TcpStream::connect(self.addr);
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
        let result =
            server.start(|_addr: SocketAddr, _msg: Msg| Ok::<&'static [u8], io::Error>(b""));
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
        let received = rx
            .recv_timeout(Duration::from_secs(1))
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
            let received = rx
                .recv_timeout(Duration::from_secs(1))
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

        let received_peer = rx
            .recv_timeout(Duration::from_secs(1))
            .expect("handler not called within 1 second");
        assert_eq!(received_peer, client_addr);
    }

    // --- shutdown tests ---

    #[test]
    fn stop_signal_unblocks_idle_accept_loop() {
        // Server is parked inside accept() with no clients. stop() must unblock it.
        let listener = TcpListener::bind("127.0.0.1:0").unwrap();
        let server = TcpServer::new(listener);
        let stop = server.stop_signal().unwrap();

        let th = thread::spawn(move || {
            server
                .start(|_addr, _msg: Msg| Ok::<&'static [u8], io::Error>(b"."))
                .ok();
        });

        // Give the server thread a moment to enter accept().
        thread::sleep(Duration::from_millis(50));
        stop.stop();

        // Without graceful shutdown, this join would hang forever.
        let (tx, rx) = mpsc::channel();
        thread::spawn(move || {
            th.join().ok();
            tx.send(()).ok();
        });
        rx.recv_timeout(Duration::from_secs(1))
            .expect("server thread did not exit within 1 second of stop()");
    }

    #[test]
    fn stop_signal_after_client_disconnect_lets_thread_exit() {
        // Verify a client connection completing cleanly + stop() lets the server thread exit.
        let listener = TcpListener::bind("127.0.0.1:0").unwrap();
        let addr = listener.local_addr().unwrap();
        let server = TcpServer::new(listener);
        let stop = server.stop_signal().unwrap();

        let th = thread::spawn(move || {
            server
                .start(|_addr, _msg: Msg| Ok::<&'static [u8], io::Error>(b"."))
                .ok();
        });

        // Connect, send one message, disconnect.
        send_recv(addr, &[1u8], 1);

        stop.stop();

        let (tx, rx) = mpsc::channel();
        thread::spawn(move || {
            th.join().ok();
            tx.send(()).ok();
        });
        rx.recv_timeout(Duration::from_secs(1))
            .expect("server thread did not exit within 1 second of stop()");
    }

    #[test]
    fn stop_signal_is_cloneable() {
        // Two clones should both trigger the same shutdown.
        let listener = TcpListener::bind("127.0.0.1:0").unwrap();
        let server = TcpServer::new(listener);
        let stop_a = server.stop_signal().unwrap();
        let stop_b = stop_a.clone();

        let th = thread::spawn(move || {
            server
                .start(|_addr, _msg: Msg| Ok::<&'static [u8], io::Error>(b"."))
                .ok();
        });

        thread::sleep(Duration::from_millis(50));
        // Using the clone is sufficient — both share the same flag and addr.
        stop_b.stop();
        drop(stop_a);

        let (tx, rx) = mpsc::channel();
        thread::spawn(move || {
            th.join().ok();
            tx.send(()).ok();
        });
        rx.recv_timeout(Duration::from_secs(1))
            .expect("server thread did not exit within 1 second of stop()");
    }
}
