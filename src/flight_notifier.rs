use crate::craft_projector::CraftProjector;
use crate::message_broadcaster::MessageBroadcaster;
use crate::models::{BoxError, CloudEvent, EventType};
use log::{error, info};
use std::sync::Arc;

pub struct FlightNotifier {
    flight_callsign: String,
    broadcaster: Arc<MessageBroadcaster>,
    craft_projector: CraftProjector,
}

impl FlightNotifier {
    pub fn new(
        flight_callsign: String,
        broadcaster: Arc<MessageBroadcaster>,
        craft_projector: CraftProjector,
    ) -> Self {
        Self {
            flight_callsign,
            broadcaster,
            craft_projector,
        }
    }

    pub async fn run(self) -> Result<(), BoxError> {
        let mut rx = self.broadcaster.subscribe();

        info!(
            "Flight notifier started for callsign: {}",
            self.flight_callsign
        );

        loop {
            match rx.recv().await {
                Ok(event_json) => {
                    if let Err(e) = self.handle_event_json(&event_json) {
                        error!("Error handling event: {}", e);
                    }
                }
                Err(e) => {
                    error!("Error receiving event: {}", e);
                    break;
                }
            }
        }

        Ok(())
    }

    fn handle_event_json(&self, event_json: &str) -> Result<(), BoxError> {
        let event: CloudEvent = serde_json::from_str(event_json)?;
        self.handle_event(event);
        Ok(())
    }

    fn handle_event(&self, event: CloudEvent) {
        if let Some(EventType::PositionReported) = EventType::from_str(&event.event_type) {
            if let Some(icao) = event.get_icao_address() {
                if let Some(aircraft) = self.craft_projector.get_state_by_icao(&icao) {
                    // Check if this is the flight we're interested in
                    if let Some(ref callsign) = aircraft.callsign {
                        if callsign.trim() == self.flight_callsign {
                            let latitude = event
                                .data
                                .get("latitude")
                                .and_then(|v| v.as_f64())
                                .unwrap_or(0.0);
                            let longitude = event
                                .data
                                .get("longitude")
                                .and_then(|v| v.as_f64())
                                .unwrap_or(0.0);

                            info!("{}'s position: {}, {}", callsign, latitude, longitude);
                        }
                    }
                }
            }
        }
    }
}
