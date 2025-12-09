const MAX_VALUE: i32 = 10_000;
const MIN_VALUE: i32 = 0;

#[derive(Debug, Clone)]
struct State {
    value: i32,
}

#[derive(Debug, Clone)]
enum Command {
    Add { value: i32 },
    Min { value: i32 },
    Mul { value: i32 },
    Div { value: i32 },
}

#[derive(Debug, Clone)]
enum EventType {
    ValueAdded { value: i32 },
    ValueSubtracted { value: i32 },
    ValueMultiplied { value: i32 },
    ValueDivided { value: i32 },
}

#[derive(Debug, Clone)]
enum CommandResult {
    Event(EventType),
    Error(String),
}

struct Calculator;

impl Calculator {
    fn handle_command(state: &State, command: Command) -> CommandResult {
        match command {
            Command::Add { value: v } => {
                let value = (MAX_VALUE - state.value).min(v);
                CommandResult::Event(EventType::ValueAdded { value })
            }
            Command::Min { value: v } => {
                let value = MIN_VALUE.max(state.value - v);
                CommandResult::Event(EventType::ValueSubtracted { value })
            }
            Command::Mul { value: v } => {
                if state.value.saturating_mul(v) > MAX_VALUE
                    || (state.value != 0 && v != 0 && state.value * v > MAX_VALUE)
                {
                    CommandResult::Error("multiply_failed".to_string())
                } else {
                    CommandResult::Event(EventType::ValueMultiplied { value: v })
                }
            }
            Command::Div { value: 0 } => CommandResult::Error("divide_failed".to_string()),
            Command::Div { value: v } => CommandResult::Event(EventType::ValueDivided { value: v }),
        }
    }

    fn handle_event(state: &State, event: EventType) -> State {
        match event {
            EventType::ValueAdded { value: v } => State {
                value: state.value + v,
            },
            EventType::ValueSubtracted { value: v } => State {
                value: state.value - v,
            },
            EventType::ValueMultiplied { value: v } => State {
                value: state.value * v,
            },
            EventType::ValueDivided { value: v } => State {
                value: state.value / v,
            },
        }
    }
}

fn main() {
    let state = State { value: 100 };

    let command = Command::Add { value: 50 };
    match Calculator::handle_command(&state, command) {
        CommandResult::Event(event) => {
            let new_state = Calculator::handle_event(&state, event);
            println!("New state: {:?}", new_state);
        }
        CommandResult::Error(err) => {
            println!("Error: {}", err);
        }
    }
}
