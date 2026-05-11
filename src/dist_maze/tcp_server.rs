use std::io;
use std::net::{TcpListener, TcpStream};

#[derive(Debug)]
pub struct TcpServer {
    listener: TcpListener,
}

impl TcpServer {
    pub fn new(listener: TcpListener) -> Self {
        Self { listener }
    }

    pub fn start(&self, mut handler: impl FnMut(TcpStream)) -> Result<(), io::Error> {
        for stream in self.listener.incoming() {
            handler(stream?);
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::mpsc;
    use std::thread;
    use std::time::Duration;

    #[test]
    fn new_stores_listener() {
        let listener = TcpListener::bind("127.0.0.1:0").unwrap();
        let expected = listener.local_addr().unwrap();
        let server = TcpServer::new(listener);
        assert_eq!(server.listener.local_addr().unwrap(), expected);
    }

    #[test]
    fn start_calls_handler_on_connection() {
        let listener = TcpListener::bind("127.0.0.1:0").unwrap();
        let addr = listener.local_addr().unwrap();
        let server = TcpServer::new(listener);
        let (tx, rx) = mpsc::channel();

        thread::spawn(move || {
            server
                .start(move |_stream| {
                    tx.send(()).ok();
                })
                .ok();
        });

        TcpStream::connect(addr).unwrap();
        rx.recv_timeout(Duration::from_secs(1))
            .expect("handler not called within 1 second");
    }

    #[test]
    fn start_calls_handler_for_each_connection() {
        let listener = TcpListener::bind("127.0.0.1:0").unwrap();
        let addr = listener.local_addr().unwrap();
        let server = TcpServer::new(listener);
        let (tx, rx) = mpsc::channel();

        thread::spawn(move || {
            server
                .start(move |_stream| {
                    tx.send(()).ok();
                })
                .ok();
        });

        for _ in 0..3 {
            TcpStream::connect(addr).unwrap();
            rx.recv_timeout(Duration::from_secs(1))
                .expect("handler not called within 1 second");
        }
    }

    #[test]
    fn start_returns_error_on_accept_failure() {
        let listener = TcpListener::bind("127.0.0.1:0").unwrap();
        listener.set_nonblocking(true).unwrap();
        let server = TcpServer::new(listener);
        let result = server.start(|_| {});
        assert!(result.is_err());
        assert_eq!(result.unwrap_err().kind(), io::ErrorKind::WouldBlock);
    }
}
