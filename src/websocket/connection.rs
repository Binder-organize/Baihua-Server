use std::collections::HashMap;
use std::sync::RwLock;
use tokio::sync::{broadcast, watch};
use tracing::{debug, trace, warn};
use uuid::Uuid;

type RoomSubKey = (Uuid, Uuid);

// Manages all active WebSocket connections:
// - Per-room broadcast channels for real-time message delivery.
// - Per-user connection counters for online/offline presence.
// - Per-(user, room) cancel signals so leaving a room auto-unsubscribes WS.
pub struct ConnectionManager {
    rooms: RwLock<HashMap<Uuid, broadcast::Sender<String>>>,
    user_connections: RwLock<HashMap<Uuid, usize>>,
    subs: RwLock<HashMap<RoomSubKey, watch::Sender<bool>>>,
}

impl ConnectionManager {
    pub fn new() -> Self {
        Self {
            rooms: RwLock::new(HashMap::new()),
            user_connections: RwLock::new(HashMap::new()),
            subs: RwLock::new(HashMap::new()),
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
    // Returns a Receiver that fires when the subscription should be canceled.
    // (user left/kicked from the room).
    pub fn register_subscription(&self, user_id: Uuid, room_id: Uuid) -> watch::Receiver<bool> {
        let mut subs = self
            .subs
            .write()
            .expect("ConnectionManager subs lock poisoned");
        let tx = subs
            .entry((user_id, room_id))
            .or_insert_with(|| {
                let (tx, _) = watch::channel(false);
                tx
            })
            .clone();
        tx.subscribe()
    }

    // Cancel a user's subscription to a room (user left or was kicked).
    // All forward tasks for this (user, room) will exit.
    pub fn cancel_subscription(&self, user_id: Uuid, room_id: Uuid) {
        if let Some(tx) = self
            .subs
            .write()
            .expect("ConnectionManager subs lock poisoned")
            .remove(&(user_id, room_id))
        {
            let _ = tx.send(true);
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
    #[allow(dead_code)]
    pub fn is_user_online(&self, user_id: Uuid) -> bool {
        self.user_connections
            .read()
            .expect("ConnectionManager user_connections lock poisoned")
            .contains_key(&user_id)
    }
}

impl Default for ConnectionManager {
    fn default() -> Self {
        Self::new()
    }
}
