use serde::{Deserialize, Serialize};
pub type BoxError = Box<dyn std::error::Error + Send + Sync>;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CloudEvent {
    #[serde(rename = "type")]
    pub event_type: String,
    pub specversion: String,
    pub source: String,
    pub id: String,
    pub time: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub contenttype: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub datacontenttype: Option<String>,
    pub data: serde_json::Value,
}

#[derive(Debug, Clone, Default)]
pub struct AircraftState {
    pub icao_address: String,
    pub callsign: Option<String>,
    pub latitude: Option<f64>,
    pub longitude: Option<f64>,
    pub altitude: Option<i32>,
    pub heading: Option<f64>,
    pub ground_speed: Option<f64>,
    pub vertical_rate: Option<i32>,
}

impl CloudEvent {
    pub fn new(event_type: &str, data: serde_json::Value) -> Self {
        Self {
            event_type: format!("org.book.flighttracker.{}", event_type),
            specversion: "1.0".to_string(),
            source: "radio_aggregator".to_string(),
            id: uuid::Uuid::new_v4().to_string(),
            time: chrono::Utc::now().to_rfc3339(),
            contenttype: None,
            datacontenttype: Some("application/json".to_string()),
            data,
        }
    }

    pub fn get_icao_address(&self) -> Option<String> {
        self.data
            .get("icao_address")
            .and_then(|v| v.as_str())
            .map(|s| s.to_string())
    }
}

#[derive(Debug, Clone)]
pub enum EventType {
    AircraftIdentified,
    PositionReported,
    VelocityReported,
    SquawkReceived,
}

impl EventType {
    pub fn from_str(s: &str) -> Option<Self> {
        match s {
            "org.book.flighttracker.aircraft_identified" => Some(Self::AircraftIdentified),
            "org.book.flighttracker.position_reported" => Some(Self::PositionReported),
            "org.book.flighttracker.velocity_reported" => Some(Self::VelocityReported),
            "org.book.flighttracker.squawk_received" => Some(Self::SquawkReceived),
            _ => None,
        }
    }
}
