// memento.rs - Memento pattern implementation for saving/restoring calculator state

// We'll implement this pattern in three parts:
// 1. Memento:    Defining the memento that captures calculator state,
// 2. Originator: Modifying the calculator to create and restore from mementos,
// 3. Caretaker:  Building a caretaker that manages memento history.

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

#[cfg(test)]
mod tests {
    use super::*;
    use crate::state::{NumberBase, ProgrammerMode, ScientificMode, StandardMode};

    fn sample_memento() -> CalculatorMemento {
        CalculatorMemento {
            variables: HashMap::from([("x".to_string(), 42.0)]),
            history: vec![],
            mode: CalculatorStateType::Standard,
            angle_mode: AngleMode::Radians,
            number_base: None,
        }
    }

    #[test]
    fn state_manager_saves_and_restores() {
        let mut manager = CalculatorStateManager::new();
        manager.save_state("test", sample_memento());
        assert!(manager.has_state("test"));

        let restored = manager.restore_state("test").unwrap();
        assert_eq!(restored.variables["x"], 42.0);
        assert_eq!(restored.mode, CalculatorStateType::Standard);
    }

    #[test]
    fn state_manager_restore_missing_state_errors() {
        let manager = CalculatorStateManager::new();
        match manager.restore_state("missing") {
            Err(message) => assert!(message.contains("missing")),
            Ok(_) => panic!("expected an error for a missing state"),
        }
    }

    #[test]
    fn state_manager_lists_saved_states() {
        let mut manager = CalculatorStateManager::new();
        manager.save_state("a", sample_memento());
        manager.save_state("b", sample_memento());

        let mut names = manager.list_saved_states();
        names.sort();
        assert_eq!(names, vec!["a".to_string(), "b".to_string()]);
    }

    #[test]
    fn state_manager_list_is_empty_when_nothing_saved() {
        let manager = CalculatorStateManager::new();
        assert!(manager.list_saved_states().is_empty());
    }

    #[test]
    fn state_manager_deletes_saved_state() {
        let mut manager = CalculatorStateManager::new();
        manager.save_state("a", sample_memento());
        manager.delete_state("a").unwrap();
        assert!(!manager.has_state("a"));
    }

    #[test]
    fn state_manager_delete_missing_state_errors() {
        let mut manager = CalculatorStateManager::new();
        assert!(manager.delete_state("missing").is_err());
    }

    #[test]
    fn save_state_overwrites_existing() {
        let mut manager = CalculatorStateManager::new();
        manager.save_state("a", sample_memento());
        manager.save_state("a", sample_memento());
        assert_eq!(manager.list_saved_states().len(), 1);
    }

    #[test]
    fn create_state_from_memento_standard() {
        let state = create_state_from_memento(&sample_memento());
        assert_eq!(state.name(), "Standard");
    }

    #[test]
    fn create_state_from_memento_scientific() {
        let memento = CalculatorMemento {
            mode: CalculatorStateType::Scientific,
            angle_mode: AngleMode::Degrees,
            ..sample_memento()
        };
        let state = create_state_from_memento(&memento);
        assert_eq!(state.name(), "Scientific");
        assert_eq!(get_angle_mode(&*state), AngleMode::Degrees);
    }

    #[test]
    fn create_state_from_memento_programmer() {
        let memento = CalculatorMemento {
            mode: CalculatorStateType::Programmer,
            number_base: Some(NumberBase::Hexadecimal),
            ..sample_memento()
        };
        let state = create_state_from_memento(&memento);
        assert_eq!(state.name(), "Programmer");
        assert_eq!(get_number_base(&*state), Some(NumberBase::Hexadecimal));
    }

    #[test]
    fn get_calculator_state_type_classifies_modes() {
        assert_eq!(
            get_calculator_state_type(&*Box::new(StandardMode::new())),
            CalculatorStateType::Standard
        );
        assert_eq!(
            get_calculator_state_type(&*Box::new(ScientificMode::new())),
            CalculatorStateType::Scientific
        );
        assert_eq!(
            get_calculator_state_type(&*Box::new(ProgrammerMode::new())),
            CalculatorStateType::Programmer
        );
    }

    #[test]
    fn get_angle_mode_defaults_to_radians_for_non_scientific() {
        let state = Box::new(StandardMode::new());
        assert_eq!(get_angle_mode(&*state), AngleMode::Radians);
    }

    #[test]
    fn get_angle_mode_detects_degree_mode() {
        let state = Box::new(ScientificMode {
            sci_ops: Box::new(crate::adapter::StandardScientificOperations {
                angle_mode: AngleMode::Degrees,
            }),
            angle_mode: AngleMode::Degrees,
        });
        assert_eq!(get_angle_mode(&*state), AngleMode::Degrees);
    }

    #[test]
    fn get_number_base_is_none_for_non_programmer() {
        let state = Box::new(StandardMode::new());
        assert_eq!(get_number_base(&*state), None);
    }

    #[test]
    fn get_number_base_detects_programmer_base() {
        let state = Box::new(ProgrammerMode {
            base: NumberBase::Binary,
        });
        assert_eq!(get_number_base(&*state), Some(NumberBase::Binary));
    }
}
