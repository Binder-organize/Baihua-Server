use std::collections::{HashMap, HashSet};
use std::sync::RwLock;
use std::time::{Duration, Instant};
use tokio::sync::{broadcast, watch};
use tracing::{debug, trace, warn};
use uuid::Uuid;

type RoomSubKey = (Uuid, Uuid);

// A subscription entry tracks the cancel channel and a generation counter.
// The generation prevents stale disconnect cleanup from canceling a fresh
// re-subscription: each new register_subscription bumps the generation, and
// cancel_subscription only acts when the stored generation matches.
struct SubEntry {
    generation: u64,
    sender: watch::Sender<bool>,
}

// Manages all active WebSocket connections:
// - Per-room broadcast channels for real-time message delivery.
// - Per-user connection counters for online/offline presence.
// - Per-(user, room) cancel signals so leaving a room auto-unsubscribes WS.
pub struct ConnectionManager {
    rooms: RwLock<HashMap<Uuid, broadcast::Sender<String>>>,
    user_connections: RwLock<HashMap<Uuid, usize>>,
    subs: RwLock<HashMap<RoomSubKey, SubEntry>>,

    // Encrypted session state
    active_sessions: RwLock<HashMap<Uuid, Instant>>,
    pending_states: RwLock<HashSet<Uuid>>,
    ready_states: RwLock<HashMap<Uuid, (bool, bool)>>,
    grace_periods: RwLock<HashMap<Uuid, (Uuid, Instant)>>,

    shutdown: broadcast::Sender<()>,
}

impl ConnectionManager {
    pub fn new() -> Self {
        let (shutdown, _) = broadcast::channel(1);
        Self {
            rooms: RwLock::new(HashMap::new()),
            user_connections: RwLock::new(HashMap::new()),
            subs: RwLock::new(HashMap::new()),
            active_sessions: RwLock::new(HashMap::new()),
            pending_states: RwLock::new(HashSet::new()),
            ready_states: RwLock::new(HashMap::new()),
            grace_periods: RwLock::new(HashMap::new()),
            shutdown,
        }
    }

    // Subscribe to a room's broadcast channel.
    // Creates the channel on first subscription.
    pub fn subscribe(&self, room_id: Uuid) -> broadcast::Receiver<String> {
        let mut rooms = self
            .rooms
            .write()
            .expect("ConnectionManager rooms lock poisoned");
        let tx = rooms.entry(room_id).or_insert_with(|| {
            let (tx, _) = broadcast::channel(1024);
            tx
        });
        tx.subscribe()
    }

    // Broadcast a message to all subscribers of a room.
    // Logs the number of receivers that got the message (trace level).
    pub fn broadcast(&self, room_id: Uuid, message: &str) {
        let rooms = self
            .rooms
            .read()
            .expect("ConnectionManager rooms lock poisoned");
        if let Some(tx) = rooms.get(&room_id) {
            match tx.send(message.to_string()) {
                Ok(0) => {
                    debug!("broadcast to 0 receiver(s) in room {room_id}");
                }
                Ok(n) => {
                    trace!("broadcast to {n} receiver(s) in room {room_id}");
                }
                Err(_) => {
                    warn!("broadcast to room {room_id} failed: send error");
                }
            }
        }
    }

    // Register that a user has subscribed to a room.
    // Returns (Receiver, generation). The generation token must be passed to
    // cancel_subscription to avoid stale disconnect cleanup killing a fresh
    // re-subscription.
    pub fn register_subscription(
        &self,
        user_id: Uuid,
        room_id: Uuid,
    ) -> (watch::Receiver<bool>, u64) {
        let mut subs = self
            .subs
            .write()
            .expect("ConnectionManager subs lock poisoned");
        let entry = subs.entry((user_id, room_id)).or_insert_with(|| {
            let (tx, _) = watch::channel(false);
            SubEntry {
                generation: 0,
                sender: tx,
            }
        });
        entry.generation += 1;
        let generation = entry.generation;
        (entry.sender.subscribe(), generation)
    }

    // Unconditionally cancel a user's subscription to a room.
    // Used for explicit leave/kick where the cancel must always take effect.
    pub fn cancel_subscription(&self, user_id: Uuid, room_id: Uuid) {
        if let Some(entry) = self
            .subs
            .write()
            .expect("ConnectionManager subs lock poisoned")
            .remove(&(user_id, room_id))
        {
            let _ = entry.sender.send(true);
        }
    }

    // Cancel a subscription only if the generation matches.
    // Used during disconnect cleanup to avoid killing a fresh re-subscription
    // that raced with the stale disconnect handler.
    pub fn cancel_stale_subscription(&self, user_id: Uuid, room_id: Uuid, generation: u64) {
        let mut subs = self
            .subs
            .write()
            .expect("ConnectionManager subs lock poisoned");
        if let Some(entry) = subs.get(&(user_id, room_id))
            && entry.generation == generation
        {
            let entry = subs.remove(&(user_id, room_id)).unwrap();
            let _ = entry.sender.send(true);
        }
    }

    // Register a user connection. Returns `true` if this is the user's first connection
    // (i.e. the user just came online).
    pub fn user_connected(&self, user_id: Uuid) -> bool {
        let mut map = self
            .user_connections
            .write()
            .expect("ConnectionManager user_connections lock poisoned");
        let count = map.entry(user_id).or_insert(0);
        *count += 1;
        *count == 1
    }

    // Unregister a user connection. Returns `true` if the user is now fully offline
    // (no remaining connections).
    pub fn user_disconnected(&self, user_id: Uuid) -> bool {
        let mut map = self
            .user_connections
            .write()
            .expect("ConnectionManager user_connections lock poisoned");
        if let Some(count) = map.get_mut(&user_id) {
            *count -= 1;
            if *count == 0 {
                map.remove(&user_id);
                return true;
            }
        }
        false
    }

    // Check if a user has any active WebSocket connection.
    pub fn is_user_online(&self, user_id: Uuid) -> bool {
        self.user_connections
            .read()
            .expect("ConnectionManager user_connections lock poisoned")
            .contains_key(&user_id)
    }

    // ── Encrypted session helpers ─────────────────────────────────

    // Mark a room as having an active encrypted session.
    pub fn set_session_active(&self, room_id: Uuid) {
        let mut map = self
            .active_sessions
            .write()
            .expect("ConnectionManager active_sessions lock poisoned");
        map.insert(room_id, Instant::now());
    }

    // Check whether a room has an active encrypted session.
    pub fn is_session_active(&self, room_id: Uuid) -> bool {
        let map = self
            .active_sessions
            .read()
            .expect("ConnectionManager active_sessions lock poisoned");
        map.contains_key(&room_id)
    }

    // Mark a room as pending (encrypt_request sent, awaiting accept).
    pub fn mark_pending(&self, room_id: Uuid) {
        let mut map = self
            .pending_states
            .write()
            .expect("ConnectionManager pending_states lock poisoned");
        map.insert(room_id);
    }

    // Check whether a room has a pending encrypt_request.
    pub fn is_pending(&self, room_id: Uuid) -> bool {
        let map = self
            .pending_states
            .read()
            .expect("ConnectionManager pending_states lock poisoned");
        map.contains(&room_id)
    }

    // Clear pending state (after accept received or on cleanup).
    pub fn clear_pending(&self, room_id: Uuid) {
        let mut map = self
            .pending_states
            .write()
            .expect("ConnectionManager pending_states lock poisoned");
        map.remove(&room_id);
    }

    // Remove a room from the active sessions set.
    pub fn remove_session(&self, room_id: Uuid) {
        let mut map = self
            .active_sessions
            .write()
            .expect("ConnectionManager active_sessions lock poisoned");
        map.remove(&room_id);
    }

    // Mark one side (by index 0 or 1) as ready for a room.
    // Returns true when both sides have signalled ready.
    pub fn set_ready(&self, room_id: Uuid, member_index: usize) -> bool {
        let mut map = self
            .ready_states
            .write()
            .expect("ConnectionManager ready_states lock poisoned");
        let state = map.entry(room_id).or_insert((false, false));
        match member_index {
            0 => state.0 = true,
            1 => state.1 = true,
            _ => {}
        }
        let both_ready = state.0 && state.1;
        if both_ready {
            map.remove(&room_id);
        }
        both_ready
    }

    // Clean up ready state for a room (on failure / abort).
    pub fn remove_ready_state(&self, room_id: Uuid) {
        let mut map = self
            .ready_states
            .write()
            .expect("ConnectionManager ready_states lock poisoned");
        map.remove(&room_id);
    }

    // Start a 30-second grace period for a room after a user disconnects.
    pub fn start_grace_period(&self, room_id: Uuid, offline_user_id: Uuid) {
        let mut map = self
            .grace_periods
            .write()
            .expect("ConnectionManager grace_periods lock poisoned");
        map.insert(
            room_id,
            (offline_user_id, Instant::now() + Duration::from_secs(30)),
        );
    }

    // Check the current grace period for a room, if any.
    pub fn get_grace_period(&self, room_id: Uuid) -> Option<(Uuid, Instant)> {
        let map = self
            .grace_periods
            .read()
            .expect("ConnectionManager grace_periods lock poisoned");
        map.get(&room_id).copied()
    }

    // Cancel a grace period (user reconnected in time).
    pub fn cancel_grace_period(&self, room_id: Uuid) {
        let mut map = self
            .grace_periods
            .write()
            .expect("ConnectionManager grace_periods lock poisoned");
        map.remove(&room_id);
    }

    // Cancel all grace periods for a specific user (on reconnect).
    pub fn cancel_grace_periods_for_user(&self, user_id: Uuid) {
        let mut map = self
            .grace_periods
            .write()
            .expect("ConnectionManager grace_periods lock poisoned");
        map.retain(|_, (offline, _)| *offline != user_id);
    }

    // Return all active session room IDs.
    pub fn active_session_rooms(&self) -> Vec<Uuid> {
        let map = self
            .active_sessions
            .read()
            .expect("ConnectionManager active_sessions lock poisoned");
        map.keys().copied().collect()
    }

    // Signal every live connection to close so graceful shutdown does not
    // stall waiting for clients to disconnect on their own.
    pub fn initiate_shutdown(&self) {
        let _ = self.shutdown.send(());
    }

    // Subscribe to the global shutdown signal.
    pub fn shutdown_notification(&self) -> broadcast::Receiver<()> {
        self.shutdown.subscribe()
    }
}

impl Default for ConnectionManager {
    fn default() -> Self {
        Self::new()
    }
}
