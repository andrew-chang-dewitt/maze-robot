//! abstractions for simpler networked communications.

/// provides traits for bidirectional messaging w/ guaranteed delivery over TCP (e.g. certified mail).
pub mod tcp;
/// provides traits for communication by broadcasting over UDP (e.g. screaming into the void).
pub mod udp;
