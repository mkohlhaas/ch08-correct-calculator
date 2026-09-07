// calculator.rs - CorrectCalculator combining all design patterns

use crate::bridge::ConsoleDisplay;
use crate::chain::{create_input_chain, InputHandler};
use crate::command::{ClearVariablesCommand, CommandProcessor, SetVariableCommand};
use crate::memento::{
    CalculatorMemento, CalculatorStateManager, MementoOriginator, create_state_from_memento,
    get_angle_mode, get_calculator_state_type, get_number_base,
};
use crate::observer::{
    CalculatorEvent, DisplayObserver, LoggerObserver, ObservableCalculator, Observer, Subject,
    VariableProvider,
};
use crate::parser::ExpressionParser;
use crate::state::{CalculatorState, ProgrammerMode, ScientificMode, StandardMode};
use crate::visitor::{optimize_expression, validate_expression};
use std::collections::HashMap;
use std::io::{self, Write};
use std::sync::{Arc, Mutex};

// Complete calculator that combines all patterns
pub struct CorrectCalculator {
    // Chapter 7 patterns
    command_processor: CommandProcessor,
    input_chain: Box<dyn InputHandler>,

    // Chapter 8 patterns
    state: Box<dyn CalculatorState>,
    state_manager: CalculatorStateManager,
    observable: ObservableCalculator,

    // Core data
    variables: HashMap<String, f64>,
    parser: ExpressionParser,
    next_observer_id: usize,
}

impl CorrectCalculator {
    pub fn new() -> Self {
        let parser = ExpressionParser::new();
        let command_processor = CommandProcessor::new();
        let input_chain = create_input_chain(parser.clone());

        let mut calculator = Self {
            command_processor,
            input_chain,
            state: Box::new(StandardMode::new()),
            state_manager: CalculatorStateManager::new(),
            observable: ObservableCalculator::new(),
            variables: HashMap::new(),
            parser,
            next_observer_id: 0,
        };

        // Add standard observers
        let display = Arc::new(Mutex::new(ConsoleDisplay));
        calculator.attach_observer(Box::new(DisplayObserver::new(display)));
        calculator.attach_observer(Box::new(LoggerObserver));

        calculator
    }

    fn attach_observer(&mut self, observer: Box<dyn Observer>) -> usize {
        self.observable.attach(observer)
    }

    fn detach_observer(&mut self, observer_id: usize) {
        self.observable.detach(observer_id);
    }

    fn notify(&self, event: &CalculatorEvent) {
        self.observable.notify(event);
    }

    fn process_input(&mut self, input: &str) -> Result<Option<f64>, String> {
        if let Some(stripped) = input.strip_prefix("/") {
            self.process_command(stripped)
        } else if let Some((name, value_str)) = input.split_once('=') {
            // Variable assignment
            let name = name.trim();
            let value_str = value_str.trim();

            // Parse and evaluate the expression
            let expr = self.parser.parse(value_str)?;
            let value = expr.evaluate(&self.variables)?;

            // Set the variable
            self.set_variable(name, value);

            Ok(Some(value))
        } else {
            // Expression evaluation
            let expr = self.parser.parse(input)?;

            // Optimize and validate the expression
            let optimized = optimize_expression(&*expr, &self.variables)?;
            validate_expression(&*optimized)?;

            // Evaluate the optimized expression
            let result = optimized.evaluate(&self.variables)?;

            // Store the result
            self.command_processor
                .get_calculator_mut()
                .store_calculation(input.to_string(), result);

            // Notify observers
            self.notify(&CalculatorEvent::ResultCalculated(
                result,
                input.to_string(),
            ));

            Ok(Some(result))
        }
    }

    fn process_command(&mut self, command: &str) -> Result<Option<f64>, String> {
        let parts: Vec<&str> = command.split_whitespace().collect();
        if parts.is_empty() {
            return Err("Empty command".to_string());
        }

        match parts[0] {
            "help" => {
                println!("Available commands:");
                println!("  /help                - Show this help");
                println!("  /exit                - Exit the calculator");
                println!("  /mode [mode]         - Switch mode (standard, scientific, programmer)");
                println!("  /save [name]         - Save current calculator state");
                println!("  /restore [name]      - Restore saved calculator state");
                println!("  /list                - List saved states");
                println!("  /delete [name]       - Delete a saved state");
                println!("  /vars                - List all variables");
                println!("  /clear               - Clear all variables");
                println!("  /history             - Show calculation history");
                println!("  /optimize [expr]     - Show optimized version of expression");
                println!("  /validate [expr]     - Validate an expression");
                Ok(None)
            }
            "mode" => {
                if parts.len() < 2 {
                    return Err(
                        "Missing mode argument. Use /mode [standard|scientific|programmer]"
                            .to_string(),
                    );
                }

                match parts[1] {
                    "standard" => {
                        self.state = Box::new(StandardMode::new());
                        self.notify(&CalculatorEvent::ModeChanged("Standard".to_string()));
                    }
                    "scientific" => {
                        self.state = Box::new(ScientificMode::new());
                        self.notify(&CalculatorEvent::ModeChanged("Scientific".to_string()));
                    }
                    "programmer" => {
                        self.state = Box::new(ProgrammerMode::new());
                        self.notify(&CalculatorEvent::ModeChanged("Programmer".to_string()));
                    }
                    _ => return Err(format!("Unknown mode: {}", parts[1])),
                }

                println!("Switched to {} mode", self.state.name());
                Ok(None)
            }
            "save" => {
                if parts.len() < 2 {
                    return Err("Missing name argument. Use /save [name]".to_string());
                }
                let name = parts[1];

                // Create memento
                let memento = self.create_memento();

                // Save the state
                self.state_manager.save_state(name, memento);
                Ok(None)
            }
            "restore" => {
                if parts.len() < 2 {
                    return Err("Missing name argument. Use /restore [name]".to_string());
                }
                let name = parts[1];

                // Restore the state
                let memento = self.state_manager.restore_state(name)?;
                self.restore_from_memento(&memento)?;

                // Notify observers
                self.notify(&CalculatorEvent::StateRestored);

                Ok(None)
            }
            "list" => {
                let states = self.state_manager.list_saved_states();
                if states.is_empty() {
                    println!("No saved states");
                } else {
                    println!("Saved states:");
                    for state in states {
                        println!("  {}", state);
                    }
                }
                Ok(None)
            }
            "delete" => {
                if parts.len() < 2 {
                    return Err("Missing name argument. Use /delete [name]".to_string());
                }
                let name = parts[1];

                self.state_manager.delete_state(name)?;
                Ok(None)
            }
            "vars" => {
                if self.variables.is_empty() {
                    println!("No variables defined");
                } else {
                    println!("Variables:");
                    for (name, value) in &self.variables {
                        println!("  {} = {}", name, value);
                    }
                }
                Ok(None)
            }
            "clear" => {
                let command = Box::new(ClearVariablesCommand::new());
                self.command_processor.execute(command)?;
                self.variables.clear();
                println!("All variables cleared");
                Ok(None)
            }
            "history" => {
                let history = self.command_processor.get_calculator().history.clone();
                if history.is_empty() {
                    println!("No calculation history");
                } else {
                    println!("Calculation history:");
                    for (i, calc) in history.iter().enumerate() {
                        println!("  {}. {} = {}", i + 1, calc.expression, calc.result);
                    }
                }
                Ok(None)
            }
            "optimize" => {
                if parts.len() < 2 {
                    return Err("Missing expression. Use /optimize [expression]".to_string());
                }

                let expr_str = &command[parts[0].len()..].trim();
                let expr = self.parser.parse(expr_str)?;
                let optimized = optimize_expression(&*expr, &self.variables)?;

                println!("Original: {}", expr.to_string());
                println!("Optimized: {}", optimized.to_string());

                Ok(None)
            }
            "validate" => {
                if parts.len() < 2 {
                    return Err("Missing expression. Use /validate [expression]".to_string());
                }

                let expr_str = &command[parts[0].len()..].trim();
                let expr = self.parser.parse(expr_str)?;

                match validate_expression(&*expr) {
                    Ok(_) => println!("Expression is valid"),
                    Err(e) => println!("Validation errors: {}", e),
                }

                Ok(None)
            }
            _ => Err(format!("Unknown command: {}", parts[0])),
        }
    }

    fn set_variable(&mut self, name: &str, value: f64) {
        self.variables.insert(name.to_string(), value);

        // Execute SetVariableCommand to enable undo/redo
        let command = Box::new(SetVariableCommand::new(name.to_string(), value));
        let _ = self.command_processor.execute(command);

        // Notify observers
        self.notify(&CalculatorEvent::VariableChanged(name.to_string(), value));
    }

    pub fn run(&mut self) {
        println!("Correct Calculator - Chapter 8");
        println!("Incorporating patterns from Chapters 5-8");
        println!("Type expressions to evaluate, variables to set (x = 5),");
        println!("commands (/help, /mode, /save, /restore), or /exit to quit");

        loop {
            print!("{} ", self.state.display_prompt());
            io::stdout().flush().unwrap();

            // Use explicit import for stdin to ensure it works properly
            let stdin = io::stdin();
            let mut input = String::new();

            // Read from stdin and check length
            if let Ok(n) = stdin.read_line(&mut input) {
                if n == 0 {
                    // End of file or broken pipe
                    println!("End of input, exiting...");
                    break;
                }
            } else {
                println!("Error reading input, please try again");
                continue;
            }

            let input = input.trim();
            if input == "/exit" {
                break;
            }

            if input.is_empty() {
                // Skip empty input
                continue;
            }

            match self.process_input(input) {
                Ok(Some(result)) => println!("= {}", result),
                Ok(None) => {} // Command executed with no result to display
                Err(error) => {
                    println!("Error: {}", error);
                    self.notify(&CalculatorEvent::Error(error));
                }
            }
        }

        println!("Goodbye!");
    }
}

// Implement MementoOriginator for CorrectCalculator
impl MementoOriginator for CorrectCalculator {
    fn create_memento(&self) -> CalculatorMemento {
        CalculatorMemento {
            variables: self.variables.clone(),
            history: self.command_processor.get_calculator().history.clone(),
            mode: get_calculator_state_type(&*self.state),
            angle_mode: get_angle_mode(&*self.state),
            number_base: get_number_base(&*self.state),
        }
    }

    fn restore_from_memento(&mut self, memento: &CalculatorMemento) -> Result<(), String> {
        // Restore variables
        self.variables = memento.variables.clone();

        // Restore history
        self.command_processor.get_calculator_mut().history = memento.history.clone();

        // Restore state
        self.state = create_state_from_memento(memento);

        Ok(())
    }
}

// Implement VariableProvider for Arc<Mutex<CorrectCalculator>>
impl VariableProvider for CorrectCalculator {
    fn get_variable(&self, name: &str) -> Option<f64> {
        self.variables.get(name).copied()
    }

    fn set_variable(&mut self, name: &str, value: f64) {
        self.set_variable(name, value);
    }

    fn evaluate_expression(&mut self, expr: &str) -> Result<f64, String> {
        let expr_tree = self.parser.parse(expr)?;
        expr_tree.evaluate(&self.variables)
    }
}