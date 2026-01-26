use crate::models::{BoxError, CloudEvent};
use tokio::sync::broadcast;

#[derive(Clone)]
pub struct MessageBroadcaster {
    tx: broadcast::Sender<String>,
}

impl MessageBroadcaster {
    pub fn new(tx: broadcast::Sender<String>) -> Self {
        Self { tx }
    }

    pub fn subscribe(&self) -> broadcast::Receiver<String> {
        self.tx.subscribe()
    }

    pub fn broadcast_event(&self, event_json: String) -> Result<(), BoxError> {
        self.tx.send(event_json)?;
        Ok(())
    }

    pub fn broadcast_cloudevent(&self, event: CloudEvent) -> Result<(), BoxError> {
        let json = serde_json::to_string(&event)?;
        self.broadcast_event(json)
    }
}
