use crate::message_broadcaster::MessageBroadcaster;
use crate::models::BoxError;
use log::info;
use std::sync::Arc;
use tokio::fs::File;
use tokio::io::{AsyncBufReadExt, BufReader};
use tokio::time::{Duration, sleep};

pub struct FileInjector {
    file_path: String,
    broadcaster: Arc<MessageBroadcaster>,
}

impl FileInjector {
    pub fn new(file_path: String, broadcaster: Arc<MessageBroadcaster>) -> Self {
        Self {
            file_path,
            broadcaster,
        }
    }

    pub async fn run(self) -> Result<(), BoxError> {
        // Wait 2 seconds before reading file (similar to Elixir version)
        sleep(Duration::from_secs(2)).await;

        info!("Reading file: {}", self.file_path);

        let file = File::open(&self.file_path).await?;
        let reader = BufReader::new(file);
        let mut lines = reader.lines();

        while let Some(line) = lines.next_line().await? {
            let trimmed = line.trim();
            if !trimmed.is_empty() {
                if let Err(e) = self.broadcaster.broadcast_event(trimmed.to_string()) {
                    log::error!("Failed to broadcast event: {}", e);
                }
            }
        }

        info!("File injection complete");
        Ok(())
    }
}
