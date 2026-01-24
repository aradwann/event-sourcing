use std::collections::HashMap;
use std::sync::{Arc, Mutex};
use tokio::sync::{mpsc, oneshot};

#[derive(Debug, Clone)]
pub enum EventType {
    ZombieKilled { attacker: String },
}

#[derive(Debug)]
pub enum ProjectorMessage {
    HandleEvent(EventType),
    GetTop10(oneshot::Sender<Vec<(String, u64)>>),
    GetScore {
        attacker: String,
        reply: oneshot::Sender<u64>,
    },
    WeekCompleted,
}

#[derive(Debug, Clone)]
pub struct ProjectorState {
    scores: HashMap<String, u64>,
    top10: Vec<(String, u64)>,
}

impl ProjectorState {
    fn new() -> Self {
        Self {
            scores: HashMap::new(),
            top10: Vec::new(),
        }
    }

    fn rerank(&mut self) {
        let mut sorted: Vec<_> = self.scores.iter().map(|(k, v)| (k.clone(), *v)).collect();

        sorted.sort_by(|a, b| b.1.cmp(&a.1)); // Sort descending by score
        self.top10 = sorted.into_iter().take(10).collect();
    }

    fn reset_scores(&mut self) {
        self.scores.clear();
    }
}

pub struct Projector {
    sender: mpsc::UnboundedSender<ProjectorMessage>,
}

impl Projector {
    pub fn start_link() -> Self {
        let (tx, mut rx) = mpsc::unbounded_channel::<ProjectorMessage>();

        tokio::spawn(async move {
            let mut state = ProjectorState::new();

            while let Some(msg) = rx.recv().await {
                match msg {
                    ProjectorMessage::HandleEvent(EventType::ZombieKilled { attacker }) => {
                        let score = state.scores.entry(attacker).or_insert(0);
                        *score += 1;
                        state.rerank();
                    }
                    ProjectorMessage::GetTop10(reply) => {
                        let _ = reply.send(state.top10.clone());
                    }
                    ProjectorMessage::GetScore { attacker, reply } => {
                        let score = state.scores.get(&attacker).copied().unwrap_or(0);
                        let _ = reply.send(score);
                    }
                    ProjectorMessage::WeekCompleted => {
                        state.reset_scores();
                        state.rerank();
                    }
                }
            }
        });

        Self { sender: tx }
    }

    pub fn apply_event(&self, event: EventType) {
        let _ = self.sender.send(ProjectorMessage::HandleEvent(event));
    }

    pub async fn get_top10(&self) -> Vec<(String, u64)> {
        let (tx, rx) = oneshot::channel();
        let _ = self.sender.send(ProjectorMessage::GetTop10(tx));
        rx.await.unwrap_or_default()
    }

    pub async fn get_score(&self, attacker: String) -> u64 {
        let (tx, rx) = oneshot::channel();
        let _ = self.sender.send(ProjectorMessage::GetScore {
            attacker,
            reply: tx,
        });
        rx.await.unwrap_or(0)
    }
}

// Example usage:
#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_projector() {
        let projector = Projector::start_link();

        projector.apply_event(EventType::ZombieKilled {
            attacker: "player1".to_string(),
        });
        projector.apply_event(EventType::ZombieKilled {
            attacker: "player1".to_string(),
        });
        projector.apply_event(EventType::ZombieKilled {
            attacker: "player2".to_string(),
        });

        // Give time for events to process
        tokio::time::sleep(tokio::time::Duration::from_millis(10)).await;

        let score = projector.get_score("player1".to_string()).await;
        assert_eq!(score, 2);

        let top10 = projector.get_top10().await;
        assert_eq!(top10.len(), 2);
        assert_eq!(top10[0], ("player1".to_string(), 2));
    }
}
