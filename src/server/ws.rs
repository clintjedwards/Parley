// WebSocket hub. Broadcasts real-time events to connected TUI clients.
//
// Pattern: a tokio::sync::broadcast channel owned by ApiState.
// Each connected client subscribes and streams events until disconnect.

use crate::models::WsEvent;
use tokio::sync::broadcast;

pub struct WsHub {
    tx: broadcast::Sender<WsEvent>,
}

impl WsHub {
    pub fn new() -> Self {
        let (tx, _) = broadcast::channel(256);
        Self { tx }
    }

    pub fn subscribe(&self) -> broadcast::Receiver<WsEvent> {
        self.tx.subscribe()
    }

    pub fn broadcast(&self, event: WsEvent) {
        // Ignore send errors — no connected clients is fine
        let _ = self.tx.send(event);
    }
}
