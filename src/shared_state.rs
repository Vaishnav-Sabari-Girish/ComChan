use std::collections::HashMap;
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::{Arc, RwLock};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Message {
    pub id: u64,
    pub content: String,
    pub read: bool,
    pub acked: bool,
}

impl Message {
    fn new(id: u64, content: String) -> Self {
        Self {
            id,
            content,
            read: false,
            acked: false,
        }
    }
}

/// Thread-safe channel that holds messages and allows atomic updates.
#[derive(Clone, Debug)]
pub struct Channel {
    inner: Arc<RwLock<HashMap<u64, Message>>>,
    next_id: Arc<AtomicU64>,
}

impl Channel {
    pub fn new() -> Self {
        Channel {
            inner: Arc::new(RwLock::new(HashMap::new())),
            next_id: Arc::new(AtomicU64::new(1)),
        }
    }

    /// Add a message and return its id.
    pub fn add_message(&self, content: impl Into<String>) -> u64 {
        let id = self.next_id.fetch_add(1, Ordering::SeqCst);
        let msg = Message::new(id, content.into());
        let mut w = self.inner.write().unwrap();
        w.insert(id, msg);
        id
    }

    /// Mark a message read. Returns true if the message existed and state changed.
    pub fn mark_read(&self, id: u64) -> bool {
        let mut w = self.inner.write().unwrap();
        if let Some(msg) = w.get_mut(&id) {
            if !msg.read {
                msg.read = true;
                return true;
            }
        }
        false
    }

    /// Acknowledge a message delivery. Returns true if acked now.
    pub fn ack_message(&self, id: u64) -> bool {
        let mut w = self.inner.write().unwrap();
        if let Some(msg) = w.get_mut(&id) {
            if !msg.acked {
                msg.acked = true;
                return true;
            }
        }
        false
    }

    /// Get a cloned snapshot of a message.
    pub fn get_message(&self, id: u64) -> Option<Message> {
        let r = self.inner.read().unwrap();
        r.get(&id).cloned()
    }

    /// Count messages currently stored.
    pub fn count(&self) -> usize {
        let r = self.inner.read().unwrap();
        r.len()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::Arc;
    use std::thread;

    #[test]
    fn concurrent_add_and_mark() {
        let ch = Arc::new(Channel::new());

        // Spawn producers that add messages concurrently
        let mut producers = Vec::new();
        for i in 0..4 {
            let c = ch.clone();
            producers.push(thread::spawn(move || {
                for j in 0..250 {
                    let _ = c.add_message(format!("msg-{}-{}", i, j));
                }
            }));
        }

        for p in producers {
            p.join().expect("producer failed");
        }

        // Ensure all messages added
        let total = ch.count();
        assert_eq!(total, 4 * 250);

        // Spawn concurrent workers to mark read and ack messages
        let mut workers = Vec::new();
        let ids: Vec<u64> = {
            let r = ch.inner.read().unwrap();
            r.keys().cloned().collect()
        };

        for _ in 0..8 {
            let c = ch.clone();
            let ids = ids.clone();
            workers.push(thread::spawn(move || {
                for id in ids.iter() {
                    // mark read and ack in arbitrary order
                    let _ = c.mark_read(*id);
                    let _ = c.ack_message(*id);
                }
            }));
        }

        for w in workers {
            w.join().expect("worker failed");
        }

        // Validate all messages are read and acked exactly once
        let r = ch.inner.read().unwrap();
        for (_id, msg) in r.iter() {
            assert!(msg.read, "message should be marked read");
            assert!(msg.acked, "message should be acked");
        }
    }
}
