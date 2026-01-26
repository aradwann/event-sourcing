use std::sync::Arc;
use tokio::sync::broadcast;

mod craft_projector;
mod file_injector;
mod flight_notifier;
mod message_broadcaster;
mod models;

use craft_projector::CraftProjector;
use file_injector::FileInjector;
use flight_notifier::FlightNotifier;
use message_broadcaster::MessageBroadcaster;
use models::BoxError;

#[tokio::main]
async fn main() -> Result<(), BoxError> {
    env_logger::init();

    // Create broadcast channel
    let (tx, _) = broadcast::channel(1000);
    let broadcaster = Arc::new(MessageBroadcaster::new(tx));

    // Start components
    let file_injector =
        FileInjector::new("./sample_cloudevents.json".to_string(), broadcaster.clone());

    let craft_projector = CraftProjector::new(broadcaster.clone());
    let flight_notifier = FlightNotifier::new(
        "AMC421".to_string(),
        broadcaster.clone(),
        craft_projector.clone(),
    );

    // Spawn all tasks
    let file_handle = tokio::spawn(async move { file_injector.run().await });

    let projector_handle = tokio::spawn(async move { craft_projector.run().await });

    let notifier_handle = tokio::spawn(async move { flight_notifier.run().await });

    // Wait for all tasks
    let file_res = file_handle.await.map_err(|e| Box::new(e) as BoxError)?;
    file_res?;
    let projector_res = projector_handle
        .await
        .map_err(|e| Box::new(e) as BoxError)?;
    projector_res?;
    let notifier_res = notifier_handle.await.map_err(|e| Box::new(e) as BoxError)?;
    notifier_res?;

    Ok(())
}
