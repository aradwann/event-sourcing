use crate::message_broadcaster::MessageBroadcaster;
use crate::models::{AircraftState, BoxError, CloudEvent, EventType};
use dashmap::DashMap;
use log::error;
use std::sync::Arc;

#[derive(Clone)]
pub struct CraftProjector {
    aircraft_table: Arc<DashMap<String, AircraftState>>,
    broadcaster: Arc<MessageBroadcaster>,
}

impl CraftProjector {
    pub fn new(broadcaster: Arc<MessageBroadcaster>) -> Self {
        Self {
            aircraft_table: Arc::new(DashMap::new()),
            broadcaster,
        }
    }

    pub async fn run(self) -> Result<(), BoxError> {
        let mut rx = self.broadcaster.subscribe();

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
        let event_type = EventType::from_str(&event.event_type);

        match event_type {
            Some(EventType::AircraftIdentified) => {
                self.handle_aircraft_identified(event);
            }
            Some(EventType::VelocityReported) => {
                self.handle_velocity_reported(event);
            }
            Some(EventType::PositionReported) => {
                self.handle_position_reported(event);
            }
            _ => {
                // Ignore other event types
            }
        }
    }

    fn handle_aircraft_identified(&self, event: CloudEvent) {
        if let Some(icao) = event.get_icao_address() {
            let callsign = event
                .data
                .get("callsign")
                .and_then(|v| v.as_str())
                .map(|s| s.to_string());

            self.aircraft_table
                .entry(icao.clone())
                .and_modify(|state| {
                    state.callsign = callsign.clone();
                })
                .or_insert_with(|| AircraftState {
                    icao_address: icao,
                    callsign,
                    ..Default::default()
                });
        }
    }

    fn handle_velocity_reported(&self, event: CloudEvent) {
        if let Some(icao) = event.get_icao_address() {
            let heading = event.data.get("heading").and_then(|v| v.as_f64());
            let ground_speed = event.data.get("ground_speed").and_then(|v| v.as_f64());
            let vertical_rate = event
                .data
                .get("vertical_rate")
                .and_then(|v| v.as_i64())
                .map(|v| v as i32);

            self.aircraft_table
                .entry(icao.clone())
                .and_modify(|state| {
                    state.heading = heading;
                    state.ground_speed = ground_speed;
                    state.vertical_rate = vertical_rate;
                })
                .or_insert_with(|| AircraftState {
                    icao_address: icao,
                    heading,
                    ground_speed,
                    vertical_rate,
                    ..Default::default()
                });
        }
    }

    fn handle_position_reported(&self, event: CloudEvent) {
        if let Some(icao) = event.get_icao_address() {
            let longitude = event.data.get("longitude").and_then(|v| v.as_f64());
            let latitude = event.data.get("latitude").and_then(|v| v.as_f64());
            let altitude = event
                .data
                .get("altitude")
                .and_then(|v| v.as_i64())
                .map(|v| v as i32);

            self.aircraft_table
                .entry(icao.clone())
                .and_modify(|state| {
                    state.longitude = longitude;
                    state.latitude = latitude;
                    state.altitude = altitude;
                })
                .or_insert_with(|| AircraftState {
                    icao_address: icao,
                    longitude,
                    latitude,
                    altitude,
                    ..Default::default()
                });
        }
    }

    pub fn get_state_by_icao(&self, icao: &str) -> Option<AircraftState> {
        self.aircraft_table
            .get(icao)
            .map(|entry| entry.value().clone())
    }

    pub fn aircraft_by_callsign(&self, callsign: &str) -> Option<AircraftState> {
        self.aircraft_table
            .iter()
            .find(|entry| {
                entry
                    .value()
                    .callsign
                    .as_ref()
                    .map(|cs| cs == callsign)
                    .unwrap_or(false)
            })
            .map(|entry| entry.value().clone())
    }
}
