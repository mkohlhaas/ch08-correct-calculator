// observer.rs - Observer pattern implementation for reactive updates

// - 0. Events that can be observed
// - 1. Define the Observer trait
// - 2. Define the Subject (Observable)
// - 3. Implement Concrete Observers
//   - A. Display observer that updates UI when calculator state changes
//   - B. Observer for dependent variables
//   - C. Logger observer that logs all events
//   - D. History observer that tracks calculation history
// - 4. Implementation of Subject for a calculator

// Notes:
//
// 1. Observer trait:
//   - update(...)
// 2. Subject trait:
//   - registration and notification: attach(...), detach(...)
//   - notify(...), will call update(...)
// 4. Subject implementation:
//   - keeps a list of observers

use crate::bridge::Display;
use crate::command::Calculation;
use std::collections::HashMap;
use std::sync::{Arc, Mutex};

// ============================== //
// 0. Events that can be observed //
// ============================== //

#[derive(Clone, Debug)]
pub enum CalculatorEvent {
    VariableChanged(String, f64),
    ResultCalculated(f64, String), // Result and expression
    ModeChanged(String),
    HistoryAdded(Calculation),
    StateRestored,
    Error(String),
}

// ============================ //
// 1. Define the Observer trait //
// ============================ //

pub trait Observer: Send + Sync {
    fn update(&self, event: &CalculatorEvent); // will be called from `notify` in the Subject
}

// ================================== //
// 2. Define the Subject (Observable) //
// ================================== //

// Could simply be a struct. Here we use a trait and define a Subject struct later.

// Subject that maintains a list of observers
pub trait Subject {
    fn attach(&mut self, observer: Box<dyn Observer>) -> usize;
    fn detach(&mut self, observer_id: usize);
    fn notify(&self, event: &CalculatorEvent); // calls update in the Observer
}

// =============================== //
// 3. Implement Concrete Observers //
// =============================== //

// ================================================================= //
// A. Display observer that updates UI when calculator state changes //
// ================================================================= //

pub struct DisplayObserver {
    // The Arc<Mutex<dyn Display>> enables thread-safe sharing of the display, which is important for
    // responsive user interfaces where the display might be accessed from both user interface and
    // calculation threads.
    display: Arc<Mutex<dyn Display>>,
}

impl DisplayObserver {
    pub fn new(display: Arc<Mutex<dyn Display>>) -> Self {
        Self { display }
    }
}

impl Observer for DisplayObserver {
    fn update(&self, event: &CalculatorEvent) {
        use CalculatorEvent::*;

        let display = self.display.lock().unwrap();
        match event {
            ResultCalculated(result, expr) => {
                display.show_result(*result);
                display.show_message(&format!("Evaluated: {}", expr));
            }
            VariableChanged(name, value) => {
                display.show_message(&format!("Variable {} = {}", name, value));
            }
            ModeChanged(mode) => {
                display.show_message(&format!("Switched to {} mode", mode));
            }
            Error(message) => {
                display.show_error(message);
            }
            StateRestored => {
                display.show_message("Calculator state restored");
            }
            HistoryAdded(_) => {
                // Do nothing for history additions
            }
        }
    }
}

// =================================== //
// B. Observer for dependent variables //
// =================================== //

// For more advanced reactive behavior, observers can implement dependent variables, where
// sum = x + y automatically recalculates when x or y changes. A dependent variable observer watches
// for VariableChanged events, checks whether the changed variable is a dependency, and triggers
// recalculation. The full implementation is in the code repository.

pub struct DependentVariableObserver {
    calculator: Arc<Mutex<dyn VariableProvider>>,
    dependencies: HashMap<String, Vec<(String, String)>>, // Map of variable to tuples of dependent var name and expression
}

// Interface for calculator to provide variable evaluation
pub trait VariableProvider: Send + Sync {
    fn get_variable(&self, name: &str) -> Option<f64>;
    fn set_variable(&mut self, name: &str, value: f64);
    fn evaluate_expression(&mut self, expr: &str) -> Result<f64, String>;
}

impl DependentVariableObserver {
    pub fn new(calculator: Arc<Mutex<dyn VariableProvider>>) -> Self {
        Self {
            calculator,
            dependencies: HashMap::new(),
        }
    }

    pub fn add_dependency(&mut self, variable: &str, dependent: &str, expression: &str) {
        let dependencies = self.dependencies.entry(variable.to_string()).or_default();

        dependencies.push((dependent.to_string(), expression.to_string()));
    }

    pub fn remove_dependency(&mut self, variable: &str, dependent: &str) {
        if let Some(dependencies) = self.dependencies.get_mut(variable) {
            dependencies.retain(|(dep, _)| dep != dependent);

            if dependencies.is_empty() {
                self.dependencies.remove(variable);
            }
        }
    }
}

impl Observer for DependentVariableObserver {
    fn update(&self, event: &CalculatorEvent) {
        if let CalculatorEvent::VariableChanged(name, _) = event {
            // Check if any variables depend on this one
            if let Some(dependents) = self.dependencies.get(name) {
                let mut calc = self.calculator.lock().unwrap();
                for (dependent, expr) in dependents {
                    // Re-evaluate the dependent variable
                    if let Ok(value) = calc.evaluate_expression(expr) {
                        calc.set_variable(dependent, value);
                    }
                }
            }
        } else if let CalculatorEvent::StateRestored = event {
            // Re-evaluate all dependent variables
            let mut calc = self.calculator.lock().unwrap();
            for dependents in self.dependencies.values() {
                for (dependent, expr) in dependents {
                    if let Ok(value) = calc.evaluate_expression(expr) {
                        calc.set_variable(dependent, value);
                    }
                }
            }
        }
    }
}

// ======================================= //
// C. Logger observer that logs all events //
// ======================================= //

pub struct LoggerObserver;

impl Observer for LoggerObserver {
    fn update(&self, event: &CalculatorEvent) {
        use CalculatorEvent::*;

        match event {
            VariableChanged(name, value) => {
                println!("[LOG] Variable changed: {} = {}", name, value);
            }
            ResultCalculated(result, expr) => {
                println!("[LOG] Calculation: {} = {}", expr, result);
            }
            ModeChanged(mode) => {
                println!("[LOG] Mode changed to: {}", mode);
            }
            HistoryAdded(calc) => {
                println!("[LOG] History added: {} = {}", calc.expression, calc.result);
            }
            StateRestored => {
                println!("[LOG] State restored");
            }
            Error(message) => {
                println!("[LOG] Error: {}", message);
            }
        }
    }
}

// =================================================== //
// D. History observer that tracks calculation history //
// =================================================== //

pub struct HistoryObserver {
    max_entries: usize,
    history: Arc<Mutex<Vec<Calculation>>>,
}

impl HistoryObserver {
    pub fn new(max_entries: usize) -> Self {
        Self {
            max_entries,
            history: Arc::new(Mutex::new(Vec::new())),
        }
    }

    pub fn get_history(&self) -> Arc<Mutex<Vec<Calculation>>> {
        Arc::clone(&self.history)
    }
}

impl Observer for HistoryObserver {
    fn update(&self, event: &CalculatorEvent) {
        if let CalculatorEvent::HistoryAdded(calc) = event {
            let mut history = self.history.lock().unwrap();
            history.push(calc.clone());

            // Trim if exceeds max entries
            if history.len() > self.max_entries {
                history.remove(0);
            }
        } else if let CalculatorEvent::StateRestored = event {
            // Clear history when state is restored
            let mut history = self.history.lock().unwrap();
            history.clear();
        }
    }
}

// ============================================= //
// 4. Implementation of Subject for a calculator //
// ============================================= //

// Subject keeps a list of observers

// IDs avoid the need for PartialEq on trait objects, which Rust doesn't support.

pub struct ObservableCalculator {
    // every observer has an ID (sequence number)
    observers: HashMap<usize, Box<dyn Observer>>,
    next_observer_id: usize,
}

impl ObservableCalculator {
    pub fn new() -> Self {
        Self {
            observers: HashMap::new(),
            next_observer_id: 0,
        }
    }
}

impl Subject for ObservableCalculator {
    fn attach(&mut self, observer: Box<dyn Observer>) -> usize {
        let id = self.next_observer_id;
        self.observers.insert(id, observer);
        self.next_observer_id += 1;
        id
    }

    fn detach(&mut self, observer_id: usize) {
        self.observers.remove(&observer_id);
    }

    fn notify(&self, event: &CalculatorEvent) {
        for observer in self.observers.values() {
            observer.update(event);
        }
    }
}

// ===== //
// Tests //
// ===== //

#[cfg(test)]
mod tests {
    use super::*;
    use crate::command::Calculation;

    struct SpyObserver {
        events: Arc<Mutex<Vec<CalculatorEvent>>>,
    }

    impl Observer for SpyObserver {
        fn update(&self, event: &CalculatorEvent) {
            self.events.lock().unwrap().push(event.clone());
        }
    }

    fn calc(result: f64) -> Calculation {
        Calculation {
            expression: format!("{}", result),
            result,
            timestamp: std::time::SystemTime::now(),
        }
    }

    #[test]
    fn observable_attaches_and_notifies() {
        let mut observable = ObservableCalculator::new();
        let events = Arc::new(Mutex::new(Vec::new()));
        observable.attach(Box::new(SpyObserver {
            events: events.clone(),
        }));

        observable.notify(&CalculatorEvent::ResultCalculated(5.0, "2 + 3".to_string()));
        observable.notify(&CalculatorEvent::ModeChanged("Standard".to_string()));

        assert_eq!(events.lock().unwrap().len(), 2);
    }

    #[test]
    fn observable_detach_stops_notifications() {
        let mut observable = ObservableCalculator::new();
        let events = Arc::new(Mutex::new(Vec::new()));
        let id = observable.attach(Box::new(SpyObserver {
            events: events.clone(),
        }));
        observable.detach(id);

        observable.notify(&CalculatorEvent::Error("boom".to_string()));
        assert!(events.lock().unwrap().is_empty());
    }

    #[test]
    fn observable_assigns_unique_observer_ids() {
        let mut observable = ObservableCalculator::new();
        let events = Arc::new(Mutex::new(Vec::new()));
        let id1 = observable.attach(Box::new(SpyObserver {
            events: events.clone(),
        }));
        let id2 = observable.attach(Box::new(SpyObserver {
            events: events.clone(),
        }));
        assert_ne!(id1, id2);
    }

    // ---------- HistoryObserver ----------

    #[test]
    fn history_observer_records_events() {
        let observer = HistoryObserver::new(10);
        let history = observer.get_history();

        observer.update(&CalculatorEvent::HistoryAdded(calc(1.0)));
        observer.update(&CalculatorEvent::HistoryAdded(calc(2.0)));

        let history = history.lock().unwrap();
        assert_eq!(history.len(), 2);
        assert_eq!(history[0].result, 1.0);
        assert_eq!(history[1].result, 2.0);
    }

    #[test]
    fn history_observer_trims_to_max_entries() {
        let observer = HistoryObserver::new(2);
        let history = observer.get_history();

        observer.update(&CalculatorEvent::HistoryAdded(calc(1.0)));
        observer.update(&CalculatorEvent::HistoryAdded(calc(2.0)));
        observer.update(&CalculatorEvent::HistoryAdded(calc(3.0)));

        let history = history.lock().unwrap();
        assert_eq!(history.len(), 2);
        assert_eq!(history[0].result, 2.0);
        assert_eq!(history[1].result, 3.0);
    }

    #[test]
    fn history_observer_clears_on_state_restore() {
        let observer = HistoryObserver::new(10);
        let history = observer.get_history();

        observer.update(&CalculatorEvent::HistoryAdded(calc(1.0)));
        observer.update(&CalculatorEvent::StateRestored);

        assert!(history.lock().unwrap().is_empty());
    }

    // ---------- DisplayObserver ----------

    struct MockDisplay {
        messages: Arc<Mutex<Vec<String>>>,
    }

    impl crate::bridge::Display for MockDisplay {
        fn show_result(&self, result: f64) {
            self.messages
                .lock()
                .unwrap()
                .push(format!("result:{}", result));
        }

        fn show_error(&self, error: &str) {
            self.messages
                .lock()
                .unwrap()
                .push(format!("error:{}", error));
        }

        fn show_expression(&self, expression: &dyn crate::expression::Expression) {
            self.messages
                .lock()
                .unwrap()
                .push(format!("expr:{}", expression.to_string()));
        }

        fn show_message(&self, message: &str) {
            self.messages.lock().unwrap().push(message.to_string());
        }
    }

    #[test]
    fn display_observer_reacts_to_results() {
        let messages = Arc::new(Mutex::new(Vec::new()));
        let display = Arc::new(Mutex::new(MockDisplay {
            messages: messages.clone(),
        }));
        let observer = DisplayObserver::new(display);

        observer.update(&CalculatorEvent::ResultCalculated(5.0, "2 + 3".to_string()));

        let messages = messages.lock().unwrap();
        assert_eq!(messages[0], "result:5");
        assert!(messages[1].contains("2 + 3"));
    }

    #[test]
    fn display_observer_reacts_to_errors() {
        let messages = Arc::new(Mutex::new(Vec::new()));
        let display = Arc::new(Mutex::new(MockDisplay {
            messages: messages.clone(),
        }));
        let observer = DisplayObserver::new(display);

        observer.update(&CalculatorEvent::Error("bad input".to_string()));

        assert_eq!(messages.lock().unwrap()[0], "error:bad input");
    }

    #[test]
    fn display_observer_reacts_to_variable_changed() {
        let messages = Arc::new(Mutex::new(Vec::new()));
        let display = Arc::new(Mutex::new(MockDisplay {
            messages: messages.clone(),
        }));
        let observer = DisplayObserver::new(display);

        observer.update(&CalculatorEvent::VariableChanged("x".to_string(), 7.0));

        assert_eq!(messages.lock().unwrap()[0], "Variable x = 7");
    }

    // ---------- DependentVariableObserver ----------

    struct MockVariableProvider {
        variables: HashMap<String, f64>,
        parser: crate::parser::ExpressionParser,
    }

    impl VariableProvider for MockVariableProvider {
        fn get_variable(&self, name: &str) -> Option<f64> {
            self.variables.get(name).copied()
        }

        fn set_variable(&mut self, name: &str, value: f64) {
            self.variables.insert(name.to_string(), value);
        }

        fn evaluate_expression(&mut self, expression: &str) -> Result<f64, String> {
            let tree = self.parser.parse(expression)?;
            tree.evaluate(&self.variables)
        }
    }

    fn mock_provider() -> Arc<Mutex<MockVariableProvider>> {
        Arc::new(Mutex::new(MockVariableProvider {
            variables: HashMap::new(),
            parser: crate::parser::ExpressionParser::new(),
        }))
    }

    #[test]
    fn dependent_variable_observer_recalculates() {
        let provider = mock_provider();
        provider.lock().unwrap().set_variable("x", 5.0);
        let mut observer = DependentVariableObserver::new(provider.clone());
        observer.add_dependency("x", "y", "x + 1");

        observer.update(&CalculatorEvent::VariableChanged("x".to_string(), 5.0));

        assert_eq!(provider.lock().unwrap().get_variable("y"), Some(6.0));
    }

    #[test]
    fn dependent_variable_observer_removed_dependency_ignored() {
        let provider = mock_provider();
        let mut observer = DependentVariableObserver::new(provider.clone());
        observer.add_dependency("x", "y", "x + 1");
        observer.remove_dependency("x", "y");

        observer.update(&CalculatorEvent::VariableChanged("x".to_string(), 5.0));

        assert_eq!(provider.lock().unwrap().get_variable("y"), None);
    }

    #[test]
    fn dependent_variable_observer_ignores_unrelated_events() {
        let provider = mock_provider();
        let mut observer = DependentVariableObserver::new(provider.clone());
        observer.add_dependency("x", "y", "x + 1");

        observer.update(&CalculatorEvent::VariableChanged("z".to_string(), 5.0));

        assert_eq!(provider.lock().unwrap().get_variable("y"), None);
    }

    #[test]
    fn dependent_variable_observer_recalculates_on_restore() {
        let provider = mock_provider();
        provider.lock().unwrap().set_variable("x", 3.0);
        let mut observer = DependentVariableObserver::new(provider.clone());
        observer.add_dependency("x", "y", "x * 2");

        observer.update(&CalculatorEvent::StateRestored);

        assert_eq!(provider.lock().unwrap().get_variable("y"), Some(6.0));
    }

    // ---------- LoggerObserver ----------

    #[test]
    fn logger_observer_handles_all_event_kinds() {
        let observer = LoggerObserver;
        observer.update(&CalculatorEvent::ResultCalculated(1.0, "1".to_string()));
        observer.update(&CalculatorEvent::VariableChanged("x".to_string(), 2.0));
        observer.update(&CalculatorEvent::ModeChanged("Scientific".to_string()));
        observer.update(&CalculatorEvent::HistoryAdded(calc(3.0)));
        observer.update(&CalculatorEvent::StateRestored);
        observer.update(&CalculatorEvent::Error("error".to_string()));
    }
}
