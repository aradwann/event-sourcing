# Flight Tracker (Rust)

A Rust implementation of the flight tracking system that processes CloudEvents for aircraft tracking.

## Project Structure

```
flight_tracker/
├── Cargo.toml
├── src/
│   ├── main.rs
│   ├── models.rs
│   ├── message_broadcaster.rs
│   ├── file_injector.rs
│   ├── craft_projector.rs
│   └── flight_notifier.rs
└── sample_cloudevents.json
```

## Components

- **MessageBroadcaster**: Event distribution using Tokio broadcast channels (replaces GenStage producer)
- **FileInjector**: Reads CloudEvents from JSON file and broadcasts them
- **CraftProjector**: Maintains aircraft state in a concurrent hash map (replaces ETS table)
- **FlightNotifier**: Monitors specific flight positions

## Key Differences from Elixir Version

1. **Concurrency Model**: Uses Tokio async runtime instead of OTP processes
2. **Event Distribution**: Tokio broadcast channels instead of GenStage
3. **State Storage**: DashMap (concurrent HashMap) instead of ETS
4. **Error Handling**: Result types instead of pattern matching
5. **Type Safety**: Strongly typed structures with Serde for JSON serialization

## Building

```bash
cargo build --release
```

## Running

```bash
# Set log level
export RUST_LOG=info

# Run the application
cargo run
```

## Dependencies

- **tokio**: Async runtime
- **serde/serde_json**: JSON serialization
- **dashmap**: Concurrent hash map
- **chrono**: Date/time handling
- **uuid**: Unique identifier generation
- **log/env_logger**: Logging

## Configuration

The flight callsign to track is hardcoded in `main.rs` as "AMC421". To track a different flight, modify:

```rust
let flight_notifier = FlightNotifier::new(
    "YOUR_CALLSIGN".to_string(),
    broadcaster.clone(),
    craft_projector.clone(),
);
```

## Example Output

```
[INFO] Reading file: ./sample_cloudevents.json
[INFO] Flight notifier started for callsign: AMC421
[INFO] AMC421's position: 10743, 99723
[INFO] AMC421's position: 24126, 104789
[INFO] AMC421's position: 24064, 104815
...
```

## Testing

The project processes events from `sample_cloudevents.json` containing CloudEvents for flight AMC421.

## Module Overview

### models.rs
Defines core data structures:
- `CloudEvent`: CloudEvents specification
- `AircraftState`: Aircraft state information
- `EventType`: Event type enumeration

### message_broadcaster.rs
Manages event distribution using broadcast channels, allowing multiple consumers to receive the same events.

### file_injector.rs
Reads CloudEvents from a file and publishes them to the broadcaster after a 2-second delay.

### craft_projector.rs
Maintains aircraft state by:
- Processing aircraft identification events
- Updating velocity information
- Tracking position data
- Providing query methods for aircraft lookup

### flight_notifier.rs
Monitors position updates for a specific flight callsign and logs position changes.
