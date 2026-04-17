//! bot controller primatives for creating/connecting to a swarm of 1+ bots & sharing data between
//! them via message passing. uses udp & a set port for discovering peer bots, then establishes TCP
//! socket connections for sharing state updates between peers?

use std::{
    net::SocketAddr,
    sync::{Arc, Mutex},
    time::Duration,
};

mod net;

// /// object for managing connections to peer robots on the local network via tcp sockets, & sending
// /// messages between them to share data
// pub struct Swarm {
//     peers: Peers,
// }
//
// /// collection of peers. thread-safe & growable as more peers are discovered.
// type Peers = Arc<Mutex<Vec<SocketAddr>>>;
//
// /// builder object for configuring connection then joining a Swarm w/ other robots on the local
// /// network listening at the given port.
// pub struct SwarmBuilder {
//     timeout: Duration,
//     port: u16,
// }
//
// impl SwarmBuilder {
//     pub fn new() -> Self {
//         Self {
//             timeout: Duration::from_secs(30),
//             port: 6000,
//         }
//     }
//
//     pub fn timeout(self, timeout: Duration) -> Self {
//         Self { timeout, ..self }
//     }
//
//     pub fn port(self, port: u16) -> Self {
//         Self { port, ..self }
//     }
//
//     pub fn connect(self) -> anyhow::Result<Swarm> {
//         let swarm = Swarm {
//             peers: discovery::start(self.timeout, self.port)?,
//         };
//
//         Ok(swarm)
//     }
// }
