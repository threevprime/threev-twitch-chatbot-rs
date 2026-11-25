pub mod types;

use tokio::sync::broadcast;

pub use types::*;

#[derive(Clone, Debug)]
pub struct EventBus {
    tx: broadcast::Sender<EventType>,
}

impl EventBus {
    pub fn new(capacity: usize) -> Self {
        let (tx, _) = broadcast::channel(capacity);
        Self { tx }
    }

    pub fn send(&self, event: EventType) -> usize {
        self.tx.send(event).unwrap_or(0)
    }

    pub fn subscribe(&self) -> broadcast::Receiver<EventType> {
        self.tx.subscribe()
    }
}
