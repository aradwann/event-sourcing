use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::{RwLock, mpsc, oneshot};
use tracing::debug;

#[derive(Debug, Clone)]
enum EventType {
    AmountWithdrawn,
    AmountDeposited,
    FeeApplied,
}

#[derive(Debug, Clone)]
struct Event {
    pub account_number: String,
    pub event_type: EventType,
    pub value: f64,
}

#[derive(Debug, Clone)]
struct ProjectorState {
    balance: f64,
    account_number: String,
}

enum ProjectorMessage {
    HandleEvent(Event),
    GetBalance(oneshot::Sender<f64>),
}

struct Projector {
    tx: mpsc::UnboundedSender<ProjectorMessage>,
}

impl Projector {
    pub fn new(account_number: String) -> Self {
        let (tx, rx) = mpsc::unbounded_channel();

        let state = ProjectorState {
            balance: 0.0,
            account_number: account_number.clone(),
        };

        tokio::spawn(Self::run(state, rx));

        Self { tx }
    }

    async fn run(mut state: ProjectorState, mut rx: mpsc::UnboundedReceiver<ProjectorMessage>) {
        while let Some(msg) = rx.recv().await {
            match msg {
                ProjectorMessage::HandleEvent(event) => {
                    state = Self::handle_event(state, event);
                }
                ProjectorMessage::GetBalance(reply) => {
                    let _ = reply.send(state.balance);
                }
            }
        }
    }

    fn handle_event(mut state: ProjectorState, event: Event) -> ProjectorState {
        match event.event_type {
            EventType::AmountWithdrawn => {
                state.balance -= event.value;
            }
            EventType::AmountDeposited => {
                state.balance += event.value;
            }
            EventType::FeeApplied => {
                state.balance -= event.value;
            }
        }
        state
    }

    pub fn apply_event(&self, event: Event) {
        let _ = self.tx.send(ProjectorMessage::HandleEvent(event));
    }

    pub async fn get_balance(&self) -> Option<f64> {
        let (tx, rx) = oneshot::channel();
        self.tx.send(ProjectorMessage::GetBalance(tx)).ok()?;
        rx.await.ok()
    }
}

pub struct ProjectorRegistry {
    projectors: Arc<RwLock<HashMap<String, Projector>>>,
}

impl ProjectorRegistry {
    pub fn new() -> Self {
        Self {
            projectors: Arc::new(RwLock::new(HashMap::new())),
        }
    }

    pub async fn apply_event(&self, event: Event) {
        let account_number = event.account_number.clone();

        // Check if projector exists
        {
            let projectors = self.projectors.read().await;
            if let Some(projector) = projectors.get(&account_number) {
                projector.apply_event(event);
                return;
            }
        }

        // Projector doesn't exist, create it
        debug!("Attempt to apply event to non-existent account, starting projector");

        let projector = Projector::new(account_number.clone());
        projector.apply_event(event);

        let mut projectors = self.projectors.write().await;
        projectors.insert(account_number, projector);
    }

    pub async fn lookup_balance(&self, account_number: &str) -> Result<f64, String> {
        let projectors = self.projectors.read().await;

        match projectors.get(account_number) {
            Some(projector) => projector
                .get_balance()
                .await
                .ok_or_else(|| "Failed to get balance".to_string()),
            None => Err("unknown_account".to_string()),
        }
    }
}

// Example usage:
#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_projector() {
        let registry = ProjectorRegistry::new();

        // Apply some events
        registry
            .apply_event(Event {
                account_number: "ACC123".to_string(),
                event_type: EventType::AmountDeposited,
                value: 100.0,
            })
            .await;

        registry
            .apply_event(Event {
                account_number: "ACC123".to_string(),
                event_type: EventType::AmountWithdrawn,
                value: 30.0,
            })
            .await;

        registry
            .apply_event(Event {
                account_number: "ACC123".to_string(),
                event_type: EventType::FeeApplied,
                value: 5.0,
            })
            .await;

        // Small delay to ensure events are processed
        tokio::time::sleep(tokio::time::Duration::from_millis(100)).await;

        let balance = registry.lookup_balance("ACC123").await.unwrap();
        assert_eq!(balance, 65.0);
    }
}
