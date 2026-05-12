//! Bot node in a swarm that solves a shared maze cooperatively.
//!
//! Connects to a [`maze_robot::dist_maze::DistMazeServer`] via TCP and joins a UDP-broadcast
//! swarm of peer bots. Drives its own DFS through the maze, broadcasting what it has seen
//! (`look_dir` results) and visited (`move_dir` destinations); ingests the same kind of messages
//! from peers and updates per-peer records. When this bot — or any peer — finds the finish cell,
//! every bot in the swarm halts.
//!
//! ## Coordinate frames
//!
//! Each bot's `(0, 0)` is wherever the maze server placed *it*, so peer messages live in the
//! sender's local frame and aren't directly comparable to local keys. The bot still stores them,
//! keyed by sender id, so they can be inspected after the run. The single piece of cross-frame
//! coordination that works without translation is the `FinishFound` halt signal.

use std::{
    cell::RefCell,
    cmp::{max, min},
    collections::{HashMap, HashSet},
    fmt::Display,
    net::SocketAddr,
    sync::{
        Arc,
        atomic::{AtomicBool, Ordering},
    },
    time::{SystemTime, UNIX_EPOCH},
};

use anyhow::{Context, anyhow};
use clap::Parser;

use maze_robot::{Cell, DIR_ARR, Direction, dist_maze::DistRobot, traits::Robot};

#[derive(Parser)]
#[command(version, about, long_about = None)]
struct App {
    /// Address of the DistMazeServer to connect to (e.g. 127.0.0.1:5000).
    #[arg(long)]
    maze: SocketAddr,
    /// UDP port for the swarm broadcast. All peers must use the same port.
    #[arg(long)]
    port: u16,
    /// Use loopback-subnet broadcast (127.255.255.255) for same-host testing instead of the
    /// wire-facing limited broadcast.
    #[arg(long)]
    local: bool,
}

fn main() -> anyhow::Result<()> {
    let app = App::parse();
    let robot = DistRobot::try_build(app.maze)
        .with_context(|| format!("connect to maze at {}", app.maze))?;
    let robot = if app.local {
        robot.join_swarm_local(app.port)?
    } else {
        robot.join_swarm(app.port)?
    };

    let bot_id = rand_bot_id();
    let halt = Arc::new(AtomicBool::new(false));
    let state = SwarmState::new(bot_id, halt.clone());

    eprintln!("[bot {bot_id:#x}] connected to maze {}; swarm port {}", app.maze, app.port);

    let result = dfs_path(&robot, &state);

    // If we found the finish, tell everyone else so they can halt too.
    if let Ok(sol) = &result {
        if let Some(finish_key) = sol.winner.first() {
            let _ = robot.send(SwarmMsg::FinishFound {
                bot_id,
                key: *finish_key,
            });
        }
    }

    println!("Bot {bot_id:#x} finished.");
    if state.halted_by_peer() {
        println!("Halted because a peer found the finish.");
    }
    match result {
        Ok(sol) => println!(
            "Solution path (own frame): {:?}\n\nLocal map:\n{}",
            sol.winner, sol.seen
        ),
        Err(e) => println!("No solution from this bot: {e:#}"),
    }
    println!("Peer records observed: {}", state.peer_summary());
    Ok(())
}

// --- swarm coordination state ---------------------------------------------------------------

/// Shared mutable state for swarm-side coordination. `peers` and `halted_by_peer` are wrapped in
/// `RefCell` because the DFS holds an `&SwarmState` across the recursion and needs to mutate
/// these from inside it; the DFS is single-threaded so this is safe. `halt` is the cross-task
/// flag (Arc<AtomicBool>) so it can also be observed from places like signal handlers later.
struct SwarmState {
    bot_id: u64,
    halt: Arc<AtomicBool>,
    halted_by_peer: RefCell<bool>,
    peers: RefCell<HashMap<u64, PeerRecord>>,
}

#[derive(Default)]
struct PeerRecord {
    seen: HashMap<Key, Cell>,
    visited: HashSet<Key>,
}

impl SwarmState {
    fn new(bot_id: u64, halt: Arc<AtomicBool>) -> Self {
        Self {
            bot_id,
            halt,
            halted_by_peer: RefCell::new(false),
            peers: RefCell::new(HashMap::new()),
        }
    }

    fn halted(&self) -> bool {
        self.halt.load(Ordering::SeqCst)
    }

    fn signal_halt(&self) {
        self.halt.store(true, Ordering::SeqCst);
    }

    fn halted_by_peer(&self) -> bool {
        *self.halted_by_peer.borrow()
    }

    /// Pull every pending swarm message off the socket and fold it into peer state. Non-blocking.
    fn drain_incoming(&self, robot: &DistRobot) {
        loop {
            match robot.try_recv::<SwarmMsg>() {
                Ok(Some(msg)) => self.apply(msg),
                Ok(None) => return,
                Err(e) => {
                    eprintln!("[bot {:#x}] try_recv error: {e}", self.bot_id);
                    return;
                }
            }
        }
    }

    fn apply(&self, msg: SwarmMsg) {
        match msg {
            SwarmMsg::Seen { bot_id, key, cell } if bot_id != self.bot_id => {
                self.peers
                    .borrow_mut()
                    .entry(bot_id)
                    .or_default()
                    .seen
                    .insert(key, cell);
            }
            SwarmMsg::Visited { bot_id, key } if bot_id != self.bot_id => {
                self.peers
                    .borrow_mut()
                    .entry(bot_id)
                    .or_default()
                    .visited
                    .insert(key);
            }
            SwarmMsg::FinishFound { bot_id, .. } if bot_id != self.bot_id => {
                *self.halted_by_peer.borrow_mut() = true;
                self.signal_halt();
            }
            _ => {}
        }
    }

    fn peer_summary(&self) -> String {
        let peers = self.peers.borrow();
        if peers.is_empty() {
            return "(none)".to_string();
        }
        let mut parts: Vec<String> = peers
            .iter()
            .map(|(id, r)| format!("{id:#x}: seen={} visited={}", r.seen.len(), r.visited.len()))
            .collect();
        parts.sort();
        parts.join("; ")
    }
}

// --- swarm message codec --------------------------------------------------------------------

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum SwarmMsg {
    Seen { bot_id: u64, key: Key, cell: Cell },
    Visited { bot_id: u64, key: Key },
    FinishFound { bot_id: u64, key: Key },
}

const TAG_SEEN: u8 = 0;
const TAG_VISITED: u8 = 1;
const TAG_FINISH: u8 = 2;

const CELL_FINISH: u8 = 0;
const CELL_OCCUPIED: u8 = 1;
const CELL_OPEN: u8 = 2;
const CELL_WALL: u8 = 3;

fn encode_cell(c: Cell) -> u8 {
    match c {
        Cell::Finish => CELL_FINISH,
        Cell::Occupied => CELL_OCCUPIED,
        Cell::Open => CELL_OPEN,
        Cell::Wall => CELL_WALL,
    }
}

fn decode_cell(b: u8) -> Result<Cell, MsgError> {
    match b {
        CELL_FINISH => Ok(Cell::Finish),
        CELL_OCCUPIED => Ok(Cell::Occupied),
        CELL_OPEN => Ok(Cell::Open),
        CELL_WALL => Ok(Cell::Wall),
        b => Err(MsgError(format!("unknown cell byte {b:#x}"))),
    }
}

impl TryFrom<SwarmMsg> for Vec<u8> {
    type Error = std::convert::Infallible;
    fn try_from(m: SwarmMsg) -> Result<Self, Self::Error> {
        let mut buf = Vec::with_capacity(32);
        match m {
            SwarmMsg::Seen { bot_id, key, cell } => {
                buf.push(TAG_SEEN);
                buf.extend_from_slice(&bot_id.to_le_bytes());
                buf.extend_from_slice(&(key.0 as i64).to_le_bytes());
                buf.extend_from_slice(&(key.1 as i64).to_le_bytes());
                buf.push(encode_cell(cell));
            }
            SwarmMsg::Visited { bot_id, key } => {
                buf.push(TAG_VISITED);
                buf.extend_from_slice(&bot_id.to_le_bytes());
                buf.extend_from_slice(&(key.0 as i64).to_le_bytes());
                buf.extend_from_slice(&(key.1 as i64).to_le_bytes());
            }
            SwarmMsg::FinishFound { bot_id, key } => {
                buf.push(TAG_FINISH);
                buf.extend_from_slice(&bot_id.to_le_bytes());
                buf.extend_from_slice(&(key.0 as i64).to_le_bytes());
                buf.extend_from_slice(&(key.1 as i64).to_le_bytes());
            }
        }
        Ok(buf)
    }
}

#[derive(Debug)]
struct MsgError(String);
impl Display for MsgError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "MsgError: {}", self.0)
    }
}
impl std::error::Error for MsgError {}

impl TryFrom<Vec<u8>> for SwarmMsg {
    type Error = MsgError;
    fn try_from(v: Vec<u8>) -> Result<Self, Self::Error> {
        let tag = *v.first().ok_or_else(|| MsgError("empty msg".into()))?;
        if v.len() < 25 {
            return Err(MsgError(format!("msg too short ({} bytes)", v.len())));
        }
        let bot_id = u64::from_le_bytes(v[1..9].try_into().unwrap());
        let x = i64::from_le_bytes(v[9..17].try_into().unwrap()) as isize;
        let y = i64::from_le_bytes(v[17..25].try_into().unwrap()) as isize;
        let key = Key(x, y);
        match tag {
            TAG_SEEN => {
                if v.len() < 26 {
                    return Err(MsgError("Seen msg missing cell byte".into()));
                }
                let cell = decode_cell(v[25])?;
                Ok(SwarmMsg::Seen { bot_id, key, cell })
            }
            TAG_VISITED => Ok(SwarmMsg::Visited { bot_id, key }),
            TAG_FINISH => Ok(SwarmMsg::FinishFound { bot_id, key }),
            t => Err(MsgError(format!("unknown tag {t:#x}"))),
        }
    }
}

fn rand_bot_id() -> u64 {
    let nanos = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default()
        .as_nanos() as u64;
    let stack = &nanos as *const _ as u64;
    nanos ^ stack
}

// --- DFS adapted from text_dfs --------------------------------------------------------------

#[derive(Eq, Debug, PartialEq)]
struct Solution {
    /// Path from start to finish in this bot's local frame. `first()` is the finish cell, the
    /// rest unwinds backwards toward the bot's start at `Key(0, 0)`.
    winner: Vec<Key>,
    seen: Seen,
}

impl Display for Solution {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.seen)
    }
}

#[derive(Eq, Debug, Default, PartialEq)]
struct Seen {
    key_ord: Vec<Key>,
    key_map: HashMap<Key, Cell>,
    min_x: isize,
    max_x: isize,
    min_y: isize,
    max_y: isize,
}

impl Seen {
    fn new() -> Self {
        Self::default()
    }

    fn push(&mut self, key: Key, cell: Cell) {
        self.key_ord.push(key);
        self.key_map.insert(key, cell);
        self.max_x = max(self.max_x, key.0);
        self.min_x = min(self.min_x, key.0);
        self.max_y = max(self.max_y, key.1);
        self.min_y = min(self.min_y, key.1);
    }

    fn get_by_coords(&self, x: isize, y: isize) -> Option<&Cell> {
        self.key_map.get(&Key(x, y))
    }
}

impl Display for Seen {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let mut out = String::new();
        for y in (self.min_y..=self.max_y).rev() {
            for x in self.min_x..=self.max_x {
                match self.get_by_coords(x, y) {
                    Some(Cell::Open) => out.push('·'),
                    Some(Cell::Wall) => out.push('░'),
                    Some(Cell::Finish) => out.push('F'),
                    Some(Cell::Occupied) => out.push('O'),
                    None => out.push(' '),
                }
            }
            out.push('\n');
        }
        write!(f, "{out}")
    }
}

#[derive(Debug, Clone, Copy)]
struct Node {
    key: Key,
    cell: Cell,
    direction: Option<Direction>,
}

impl Default for Node {
    fn default() -> Self {
        Self {
            key: Key(0, 0),
            cell: Cell::Open,
            direction: None,
        }
    }
}

#[derive(Debug, Eq, PartialEq, Hash, Copy, Clone)]
struct Key(isize, isize);

impl Key {
    fn compute_in_dir(&self, direction: &Direction) -> Self {
        match direction {
            Direction::North => Self(self.0, self.1 + 1),
            Direction::South => Self(self.0, self.1 - 1),
            Direction::East => Self(self.0 + 1, self.1),
            Direction::West => Self(self.0 - 1, self.1),
        }
    }
}

impl Display for Key {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "({},{})", self.0, self.1)
    }
}

enum MaybePath {
    Done(Vec<Key>),
    Error(anyhow::Error),
}

fn dfs_path(robot: &DistRobot, state: &SwarmState) -> anyhow::Result<Solution> {
    let mut visited = HashSet::new();
    let seen = RefCell::new(Seen::new());

    match dfs_helper(robot, state, Node::default(), &mut visited, &seen) {
        Ok(()) => {
            if state.halted_by_peer() {
                Err(anyhow!("halted: peer found the finish first"))
            } else {
                Err(anyhow!("exhausted: no path to finish from this bot's start"))
            }
        }
        Err(MaybePath::Error(e)) => Err(e.context("error encountered while searching for finish")),
        Err(MaybePath::Done(path)) => {
            // Tell peers to stop searching.
            state.signal_halt();
            Ok(Solution {
                winner: path.into_iter().rev().collect(),
                seen: seen.take(),
            })
        }
    }
}

fn dfs_helper(
    robot: &DistRobot,
    state: &SwarmState,
    node: Node,
    visited: &mut HashSet<Key>,
    seen: &RefCell<Seen>,
) -> Result<(), MaybePath> {
    // Process incoming swarm traffic at every recursion entry so peer Finish announcements halt
    // us as promptly as possible.
    state.drain_incoming(robot);
    if state.halted() {
        return Ok(());
    }

    let Node {
        key,
        cell,
        direction,
    } = node;

    // Move robot to this cell if a direction was supplied (otherwise we're at the start). A
    // MoveError here is fatal — DFS told the robot to walk into a cell its peek said was open,
    // and the only way that fails is a concurrent peer move. Bubble it up so the caller can fall
    // through to the next neighbor.
    if let Some(dir) = direction {
        robot
            .go(dir)
            .map_err(|e| MaybePath::Error(anyhow::Error::new(e)))?;
        // Broadcast that we've entered this cell.
        let _ = robot.send(SwarmMsg::Visited {
            bot_id: state.bot_id,
            key,
        });
    }

    if let Cell::Finish = cell {
        return Err(MaybePath::Done(vec![key]));
    }

    visited.insert(key);

    DIR_ARR
        .iter()
        .map(|&dir| {
            // Peek and broadcast. peek can fail on transport error; surface that as a DFS error.
            let cell_res = robot
                .peek(dir)
                .map_err(|e| MaybePath::Error(anyhow::Error::new(e)));
            (dir, cell_res)
        })
        .map(|(dir, cell_res)| {
            let neighbor_key = key.compute_in_dir(&dir);
            if let Ok(c) = cell_res {
                seen.borrow_mut().push(neighbor_key, c);
                let _ = robot.send(SwarmMsg::Seen {
                    bot_id: state.bot_id,
                    key: neighbor_key,
                    cell: c,
                });
            }
            (dir, neighbor_key, cell_res)
        })
        // Walls and occupied cells aren't walkable — skip without recursing.
        .filter_map(|(dir, neighbor_key, cell_res)| match cell_res {
            Err(e) => Some(Err(e)),
            Ok(Cell::Wall) | Ok(Cell::Occupied) => None,
            Ok(c) => Some(Ok(Node {
                key: neighbor_key,
                cell: c,
                direction: Some(dir),
            })),
        })
        .try_fold((), |_, node_res| {
            if state.halted() {
                return Ok(());
            }
            let node = match node_res {
                Ok(n) => n,
                Err(e) => return Err(e),
            };
            if visited.contains(&node.key) {
                return Ok(());
            }
            let node_direction = node.direction;
            match dfs_helper(robot, state, node, visited, seen) {
                Err(MaybePath::Done(mut path)) => {
                    path.push(key);
                    Err(MaybePath::Done(path))
                }
                Ok(()) => {
                    // Step back to the current cell so the next neighbor recursion starts here.
                    // Skip the back-step if we're halting — exiting the maze cleanly isn't a
                    // requirement.
                    if state.halted() {
                        return Ok(());
                    }
                    if let Some(dir) = node_direction {
                        // A back-step into our own previous cell can fail if another bot is now
                        // there. Treat that as a fatal error for this branch; the maze server
                        // gave us the cell originally and we have no recovery for "I can't go
                        // home".
                        robot
                            .go(dir.reverse())
                            .map_err(|e| MaybePath::Error(anyhow::Error::new(e)))?;
                    }
                    Ok(())
                }
                err => err,
            }
        })
}

// --- tests ----------------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn codec_round_trip_seen() {
        let msg = SwarmMsg::Seen {
            bot_id: 0xDEAD_BEEF_CAFE_0001,
            key: Key(-3, 7),
            cell: Cell::Open,
        };
        let bytes: Vec<u8> = msg.try_into().unwrap();
        let back: SwarmMsg = bytes.try_into().unwrap();
        assert_eq!(back, msg);
    }

    #[test]
    fn codec_round_trip_visited() {
        let msg = SwarmMsg::Visited {
            bot_id: 0xABCD_0123_4567_89EF,
            key: Key(42, -100),
        };
        let bytes: Vec<u8> = msg.try_into().unwrap();
        let back: SwarmMsg = bytes.try_into().unwrap();
        assert_eq!(back, msg);
    }

    #[test]
    fn codec_round_trip_finish() {
        let msg = SwarmMsg::FinishFound {
            bot_id: 1,
            key: Key(0, 0),
        };
        let bytes: Vec<u8> = msg.try_into().unwrap();
        let back: SwarmMsg = bytes.try_into().unwrap();
        assert_eq!(back, msg);
    }

    #[test]
    fn codec_round_trip_every_cell_variant() {
        for cell in [Cell::Finish, Cell::Occupied, Cell::Open, Cell::Wall] {
            let msg = SwarmMsg::Seen {
                bot_id: 99,
                key: Key(1, 2),
                cell,
            };
            let bytes: Vec<u8> = msg.try_into().unwrap();
            let back: SwarmMsg = bytes.try_into().unwrap();
            assert_eq!(back, msg);
        }
    }

    #[test]
    fn decode_rejects_unknown_tag() {
        let bytes = vec![0xFF; 26];
        let err = SwarmMsg::try_from(bytes).unwrap_err();
        assert!(format!("{err}").contains("unknown tag"));
    }

    #[test]
    fn decode_rejects_short_msg() {
        let bytes = vec![TAG_SEEN, 1, 2, 3];
        let err = SwarmMsg::try_from(bytes).unwrap_err();
        assert!(format!("{err}").contains("too short"));
    }

    #[test]
    fn decode_rejects_short_seen_missing_cell() {
        // header (25 bytes) but no cell byte
        let mut bytes = vec![TAG_SEEN];
        bytes.extend_from_slice(&0u64.to_le_bytes());
        bytes.extend_from_slice(&0i64.to_le_bytes());
        bytes.extend_from_slice(&0i64.to_le_bytes());
        let err = SwarmMsg::try_from(bytes).unwrap_err();
        assert!(format!("{err}").contains("missing cell"));
    }

    #[test]
    fn applying_peer_seen_records_in_peer_state() {
        let state = SwarmState::new(1, Arc::new(AtomicBool::new(false)));
        state.apply(SwarmMsg::Seen {
            bot_id: 2,
            key: Key(3, 4),
            cell: Cell::Open,
        });
        let peers = state.peers.borrow();
        let rec = peers.get(&2).expect("peer 2 recorded");
        assert_eq!(rec.seen.get(&Key(3, 4)), Some(&Cell::Open));
    }

    #[test]
    fn applying_peer_visited_records_in_peer_state() {
        let state = SwarmState::new(1, Arc::new(AtomicBool::new(false)));
        state.apply(SwarmMsg::Visited {
            bot_id: 2,
            key: Key(5, 6),
        });
        let peers = state.peers.borrow();
        let rec = peers.get(&2).expect("peer 2 recorded");
        assert!(rec.visited.contains(&Key(5, 6)));
    }

    #[test]
    fn applying_own_msg_is_ignored() {
        // Swarm filters self by nonce, but defensively the bot also ignores msgs tagged with its
        // own bot_id — useful if a peer ever forges one.
        let state = SwarmState::new(42, Arc::new(AtomicBool::new(false)));
        state.apply(SwarmMsg::Seen {
            bot_id: 42,
            key: Key(0, 0),
            cell: Cell::Open,
        });
        assert!(state.peers.borrow().is_empty());
    }

    #[test]
    fn peer_finish_halts_state() {
        let state = SwarmState::new(1, Arc::new(AtomicBool::new(false)));
        assert!(!state.halted());
        state.apply(SwarmMsg::FinishFound {
            bot_id: 2,
            key: Key(7, 8),
        });
        assert!(state.halted());
        assert!(state.halted_by_peer());
    }

    #[test]
    fn own_finish_does_not_mark_halted_by_peer() {
        // signal_halt is the right path for own-finish; apply on own-id msg is a no-op.
        let state = SwarmState::new(1, Arc::new(AtomicBool::new(false)));
        state.apply(SwarmMsg::FinishFound {
            bot_id: 1,
            key: Key(0, 0),
        });
        assert!(!state.halted());
        assert!(!state.halted_by_peer());
    }
}
