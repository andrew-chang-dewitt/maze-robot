/// establish as "server" for bidirectional communication over long-lasting TCP socket.
///
/// implementors must provide a constant specifying the port to listen for client connections on &
/// a method specifying logic for determining if a client is allowed to establish a connection.
pub trait Server {}

/// establish as "client" for bidirectional communication over long-lasting TCP socket.
///
/// implementors must provide a constant specifying the [`Server`] address to connect to.
pub trait Client {}
