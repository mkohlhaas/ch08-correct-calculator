// memento.rs - Memento pattern implementation for saving/restoring calculator state

use crate::command::Calculation;
use crate::config::AngleMode;
use crate::state::{CalculatorState, NumberBase, ProgrammerMode, ScientificMode, StandardMode};
use std::collections::HashMap;

// ==================================== //
// 1. Memento to store calculator state //
// ==================================== //

// The clone cost is the price of independence; for very large states, you might consider copy-on-write techniques.

#[derive(Clone)]
pub struct CalculatorMemento {
    pub variables: HashMap<String, f64>,
    pub history: Vec<Calculation>,
    pub mode: CalculatorStateType,
    pub angle_mode: AngleMode,
    pub number_base: Option<NumberBase>, // Only used for ProgrammerMode
}

// Enum to represent calculator state type for memento
#[derive(Clone, Debug, PartialEq)]
pub enum CalculatorStateType {
    Standard,
    Scientific,
    Programmer,
}

// ====================================================== //
// 2. Originator trait for creating and applying mementos //
// ====================================================== //

pub trait MementoOriginator {
    fn create_memento(&self) -> CalculatorMemento;
    fn restore_from_memento(&mut self, memento: &CalculatorMemento) -> Result<(), String>;
}

// Command for saving state
// pub struct SaveStateCommand {
//     pub name: String,
// }

// impl SaveStateCommand {
//     pub fn new(name: impl Into<String>) -> Self {
//         Self { name: name.into() }
//     }
// }

// Command for restoring state
// pub struct RestoreStateCommand {
//     pub name: String,
// }

// impl RestoreStateCommand {
//     pub fn new(name: impl Into<String>) -> Self {
//         Self { name: name.into() }
//     }
// }

// ================================== //
// 3. Caretaker that manages mementos //
// ================================== //

pub struct CalculatorStateManager {
    saved_states: HashMap<String, CalculatorMemento>,
}

impl CalculatorStateManager {
    pub fn new() -> Self {
        Self {
            saved_states: HashMap::new(),
        }
    }

    pub fn save_state(&mut self, name: &str, memento: CalculatorMemento) {
        self.saved_states.insert(name.to_string(), memento);
        println!("State saved as '{}'", name);
    }

    pub fn restore_state(&self, name: &str) -> Result<CalculatorMemento, String> {
        if let Some(memento) = self.saved_states.get(name) {
            println!("State '{}' restored", name);
            Ok(memento.clone())
        } else {
            Err(format!("No saved state named '{}'", name))
        }
    }

    pub fn list_saved_states(&self) -> Vec<String> {
        self.saved_states.keys().cloned().collect()
    }

    pub fn has_state(&self, name: &str) -> bool {
        self.saved_states.contains_key(name)
    }

    pub fn delete_state(&mut self, name: &str) -> Result<(), String> {
        if self.saved_states.remove(name).is_some() {
            println!("State '{}' deleted", name);
            Ok(())
        } else {
            Err(format!("No saved state named '{}'", name))
        }
    }
}

// Utility function to create a calculator state from memento
pub fn create_state_from_memento(memento: &CalculatorMemento) -> Box<dyn CalculatorState> {
    match memento.mode {
        CalculatorStateType::Standard => Box::new(StandardMode::new()),
        CalculatorStateType::Scientific => {
            let mode = ScientificMode::new();
            Box::new(ScientificMode {
                sci_ops: mode.sci_ops,
                angle_mode: memento.angle_mode,
            })
        }
        CalculatorStateType::Programmer => Box::new(ProgrammerMode {
            base: memento.number_base.unwrap_or(NumberBase::Decimal),
        }),
    }
}

// Helper to determine calculator state type
pub fn get_calculator_state_type(state: &dyn CalculatorState) -> CalculatorStateType {
    if state.name() == "Standard" {
        CalculatorStateType::Standard
    } else if state.name() == "Scientific" {
        CalculatorStateType::Scientific
    } else if state.name() == "Programmer" {
        CalculatorStateType::Programmer
    } else {
        panic!("Unknown calculator state type: {}", state.name())
    }
}

// Helper to determine angle mode from scientific mode state
pub fn get_angle_mode(state: &dyn CalculatorState) -> AngleMode {
    if state.name() == "Scientific" {
        if state.display_prompt().contains("(DEG)") {
            AngleMode::Degrees
        } else {
            AngleMode::Radians
        }
    } else {
        AngleMode::Radians // Default for non-scientific modes
    }
}

// Helper to determine number base from programmer mode state
pub fn get_number_base(state: &dyn CalculatorState) -> Option<NumberBase> {
    if state.name() == "Programmer" {
        let prompt = state.display_prompt();
        if prompt.contains("(BIN)") {
            Some(NumberBase::Binary)
        } else if prompt.contains("(OCT)") {
            Some(NumberBase::Octal)
        } else if prompt.contains("(HEX)") {
            Some(NumberBase::Hexadecimal)
        } else {
            Some(NumberBase::Decimal)
        }
    } else {
        None
    }
}
