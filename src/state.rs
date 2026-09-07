// state.rs - State pattern implementation for calculator modes

// NOTE: Is this the state pattern! No states are changed! What even are the states?

// 1. State interface
//   - A. Standard calculator mode
//   - B. Scientific calculator mode
//   - C. Programmer calculator mode
//
// 2. Calculator context for state pattern
//
// Helper functions to avoid borrowing conflicts
//   - match_standard_input
//   - match_scientific_input
//   - match_programmer_input

use crate::adapter::ScientificOperations;
use crate::config::AngleMode;
use crate::parser::ExpressionParser;
use std::collections::HashMap;

// ================== //
// 1. State interface //
// ================== //

pub trait CalculatorState: Send + Sync {
    fn name(&self) -> &str;
    fn handle_input(
        &self,
        input: &str,
        calculator: &mut StateCalculator,
    ) -> Result<Option<f64>, String>;
    fn available_operations(&self) -> Vec<&'static str>;
    fn display_prompt(&self) -> String;
    fn as_any(&self) -> &dyn std::any::Any;
}

// =========================== //
// A. Standard calculator mode //
// =========================== //

pub struct StandardMode {
    pub sci_ops: Box<dyn ScientificOperations>,
}

impl StandardMode {
    pub fn new() -> Self {
        Self {
            sci_ops: Box::new(crate::adapter::StandardScientificOperations {
                angle_mode: AngleMode::Radians,
            }),
        }
    }
}

impl CalculatorState for StandardMode {
    fn name(&self) -> &str {
        "Standard"
    }

    fn handle_input(
        &self,
        input: &str,
        calculator: &mut StateCalculator,
    ) -> Result<Option<f64>, String> {
        // Handle basic arithmetic expressions
        if input.starts_with("mode") {
            // Change mode based on command
            let mode = input.trim_start_matches("mode").trim();
            match mode {
                "scientific" => {
                    calculator.change_state(Box::new(ScientificMode::new()));
                    Ok(None)
                }
                "programmer" => {
                    calculator.change_state(Box::new(ProgrammerMode::new()));
                    Ok(None)
                }
                _ => Err(format!("Unknown mode: {}", mode)),
            }
        } else if input.starts_with("help") {
            println!(
                "Available operations: {}",
                self.available_operations().join(", ")
            );
            println!("Type 'mode scientific' or 'mode programmer' to switch modes");
            Ok(None)
        } else if let Some((var_name, expression)) = input.split_once('=') {
            let var_name = var_name.trim();
            let expression = expression.trim();

            // Evaluate the expression and set the variable
            let expr = calculator.parser.parse(expression)?;
            let result = expr.evaluate(&calculator.variables)?;
            calculator.variables.insert(var_name.to_string(), result);
            calculator.store_result(format!("{} = {}", var_name, expression), result);
            Ok(Some(result))
        } else {
            // Normal expression evaluation
            let expr = calculator.parser.parse(input)?;
            let result = expr.evaluate(&calculator.variables)?;
            calculator.store_result(input.to_string(), result);
            Ok(Some(result))
        }
    }

    fn available_operations(&self) -> Vec<&'static str> {
        vec!["+", "-", "*", "/", "^"]
    }

    fn display_prompt(&self) -> String {
        "[Standard] >".to_string()
    }

    fn as_any(&self) -> &dyn std::any::Any {
        self
    }
}

// ============================= //
// B. Scientific calculator mode //
// ============================= //

pub struct ScientificMode {
    pub sci_ops: Box<dyn ScientificOperations>,
    pub angle_mode: AngleMode,
}

impl ScientificMode {
    pub fn new() -> Self {
        Self {
            sci_ops: Box::new(crate::adapter::StandardScientificOperations {
                angle_mode: AngleMode::Radians,
            }),
            angle_mode: AngleMode::Radians,
        }
    }
}

impl CalculatorState for ScientificMode {
    fn name(&self) -> &str {
        "Scientific"
    }

    fn handle_input(
        &self,
        input: &str,
        calculator: &mut StateCalculator,
    ) -> Result<Option<f64>, String> {
        // Handle scientific expressions and functions
        if input.starts_with("mode") {
            // Handle mode change
            let mode = input.trim_start_matches("mode").trim();
            match mode {
                "standard" => {
                    calculator.change_state(Box::new(StandardMode::new()));
                    Ok(None)
                }
                "programmer" => {
                    calculator.change_state(Box::new(ProgrammerMode::new()));
                    Ok(None)
                }
                _ => Err(format!("Unknown mode: {}", mode)),
            }
        } else if input == "angle deg" {
            // Change angle mode to degrees
            calculator.change_state(Box::new(ScientificMode {
                sci_ops: Box::new(crate::adapter::StandardScientificOperations {
                    angle_mode: AngleMode::Degrees,
                }),
                angle_mode: AngleMode::Degrees,
            }));
            println!("Angle mode set to degrees");
            Ok(None)
        } else if input == "angle rad" {
            // Change angle mode to radians
            calculator.change_state(Box::new(ScientificMode {
                sci_ops: Box::new(crate::adapter::StandardScientificOperations {
                    angle_mode: AngleMode::Radians,
                }),
                angle_mode: AngleMode::Radians,
            }));
            println!("Angle mode set to radians");
            Ok(None)
        } else if input.starts_with("help") {
            println!(
                "Available operations: {}",
                self.available_operations().join(", ")
            );
            println!("Type 'mode standard' or 'mode programmer' to switch modes");
            println!("Type 'angle deg' or 'angle rad' to change angle mode");
            Ok(None)
        } else if input.starts_with("sin ")
            || input.starts_with("cos ")
            || input.starts_with("tan ")
        {
            // Handle trigonometric functions
            let (func, arg_str) = input.split_once(' ').unwrap();

            // Parse and evaluate the argument
            let expr = calculator.parser.parse(arg_str)?;
            let arg = expr.evaluate(&calculator.variables)?;

            let result = match func {
                "sin" => self.sci_ops.sin(arg),
                "cos" => self.sci_ops.cos(arg),
                "tan" => self.sci_ops.tan(arg),
                _ => unreachable!(),
            };

            calculator.store_result(input.to_string(), result);
            Ok(Some(result))
        } else if input.starts_with("log") {
            // Handle logarithm with base
            let parts: Vec<&str> = input.splitn(3, ' ').collect();
            if parts.len() != 3 {
                return Err("Usage: log <base> <value>".to_string());
            }

            let base_expr = calculator.parser.parse(parts[1])?;
            let value_expr = calculator.parser.parse(parts[2])?;

            let base = base_expr.evaluate(&calculator.variables)?;
            let value = value_expr.evaluate(&calculator.variables)?;

            let result = self.sci_ops.log(value, base)?;
            calculator.store_result(input.to_string(), result);
            Ok(Some(result))
        } else if let Some((var_name, expression)) = input.split_once('=') {
            // Handle variable assignment
            let var_name = var_name.trim();
            let expression = expression.trim();

            let expr = calculator.parser.parse(expression)?;
            let result = expr.evaluate(&calculator.variables)?;
            calculator.variables.insert(var_name.to_string(), result);
            calculator.store_result(format!("{} = {}", var_name, expression), result);
            Ok(Some(result))
        } else {
            // Handle normal expressions with scientific operations
            let expr = calculator.parser.parse(input)?;
            let result = expr.evaluate(&calculator.variables)?;
            calculator.store_result(input.to_string(), result);
            Ok(Some(result))
        }
    }

    fn available_operations(&self) -> Vec<&'static str> {
        vec![
            "+", "-", "*", "/", "^", "sin", "cos", "tan", "log", "ln", "sqrt",
        ]
    }

    fn display_prompt(&self) -> String {
        match self.angle_mode {
            AngleMode::Radians => "[Scientific (RAD)] >".to_string(),
            AngleMode::Degrees => "[Scientific (DEG)] >".to_string(),
        }
    }

    fn as_any(&self) -> &dyn std::any::Any {
        self
    }
}

// ============================= //
// C. Programmer calculator mode //
// ============================= //

pub struct ProgrammerMode {
    pub base: NumberBase,
}

impl ProgrammerMode {
    pub fn new() -> Self {
        Self {
            base: NumberBase::Decimal,
        }
    }

    // Helper for bitwise operations
    fn execute_bitwise_op(&self, a: f64, b: f64, op: fn(i64, i64) -> i64) -> f64 {
        let a_int = a as i64;
        let b_int = b as i64;
        op(a_int, b_int) as f64
    }
}

impl CalculatorState for ProgrammerMode {
    fn name(&self) -> &str {
        "Programmer"
    }

    fn handle_input(
        &self,
        input: &str,
        calculator: &mut StateCalculator,
    ) -> Result<Option<f64>, String> {
        // Handle programmer mode commands and operations
        if input.starts_with("mode") {
            // Handle mode change
            let mode = input.trim_start_matches("mode").trim();
            match mode {
                "standard" => {
                    calculator.change_state(Box::new(StandardMode::new()));
                    Ok(None)
                }
                "scientific" => {
                    calculator.change_state(Box::new(ScientificMode::new()));
                    Ok(None)
                }
                _ => Err(format!("Unknown mode: {}", mode)),
            }
        } else if input.starts_with("base") {
            // Change number base
            let base = input.trim_start_matches("base").trim();
            let new_base = match base {
                "bin" | "binary" => NumberBase::Binary,
                "oct" | "octal" => NumberBase::Octal,
                "dec" | "decimal" => NumberBase::Decimal,
                "hex" | "hexadecimal" => NumberBase::Hexadecimal,
                _ => return Err(format!("Unknown base: {}", base)),
            };

            calculator.change_state(Box::new(ProgrammerMode { base: new_base }));
            println!("Base set to {:?}", new_base);
            Ok(None)
        } else if input.starts_with("help") {
            println!(
                "Available operations: {}",
                self.available_operations().join(", ")
            );
            println!("Type 'mode standard' or 'mode scientific' to switch modes");
            println!("Type 'base bin', 'base oct', 'base dec', or 'base hex' to change base");
            println!("Bitwise operations: AND, OR, XOR, NOT, SHL, SHR");
            Ok(None)
        } else if input.starts_with("AND ") || input.starts_with("OR ") || input.starts_with("XOR ")
        {
            // Handle bitwise binary operations
            let parts: Vec<&str> = input.splitn(3, ' ').collect();
            if parts.len() != 3 {
                return Err(format!("Usage: {} <operand1> <operand2>", parts[0]));
            }

            let op = parts[0];
            let a_expr = calculator.parser.parse(parts[1])?;
            let b_expr = calculator.parser.parse(parts[2])?;

            let a = a_expr.evaluate(&calculator.variables)?;
            let b = b_expr.evaluate(&calculator.variables)?;

            let result = match op {
                "AND" => self.execute_bitwise_op(a, b, |a, b| a & b),
                "OR" => self.execute_bitwise_op(a, b, |a, b| a | b),
                "XOR" => self.execute_bitwise_op(a, b, |a, b| a ^ b),
                _ => unreachable!(),
            };

            calculator.store_result(input.to_string(), result);
            println!("{} = {}", input, self.base.format(result));
            Ok(Some(result))
        } else if input.starts_with("NOT ") {
            // Handle bitwise NOT operation
            let expr_str = input.trim_start_matches("NOT ").trim();
            let expr = calculator.parser.parse(expr_str)?;
            let value = expr.evaluate(&calculator.variables)?;

            let result = self.execute_bitwise_op(value, 0.0, |a, _| !a);
            calculator.store_result(input.to_string(), result);
            println!("{} = {}", input, self.base.format(result));
            Ok(Some(result))
        } else if input.starts_with("SHL ") || input.starts_with("SHR ") {
            // Handle shift operations
            let parts: Vec<&str> = input.splitn(3, ' ').collect();
            if parts.len() != 3 {
                return Err(format!("Usage: {} <value> <bits>", parts[0]));
            }

            let op = parts[0];
            let value_expr = calculator.parser.parse(parts[1])?;
            let bits_expr = calculator.parser.parse(parts[2])?;

            let value = value_expr.evaluate(&calculator.variables)?;
            let bits = bits_expr.evaluate(&calculator.variables)? as u32;

            let result = match op {
                "SHL" => self.execute_bitwise_op(value, bits as f64, |a, b| a << b as u32),
                "SHR" => self.execute_bitwise_op(value, bits as f64, |a, b| a >> b as u32),
                _ => unreachable!(),
            };

            calculator.store_result(input.to_string(), result);
            println!("{} = {}", input, self.base.format(result));
            Ok(Some(result))
        } else if let Some((var_name, expression)) = input.split_once('=') {
            // Handle variable assignment
            let var_name = var_name.trim();
            let expression = expression.trim();

            // Try to parse according to current base
            let result =
                if !expression.contains(|c: char| c.is_alphabetic() || "+-*/()^".contains(c)) {
                    match self.base.parse(expression) {
                        Ok(value) => value,
                        Err(_) => {
                            // Fall back to regular parser if base-specific parsing fails
                            let expr = calculator.parser.parse(expression)?;
                            expr.evaluate(&calculator.variables)?
                        }
                    }
                } else {
                    // For expressions, use the regular parser
                    let expr = calculator.parser.parse(expression)?;
                    expr.evaluate(&calculator.variables)?
                };

            calculator.variables.insert(var_name.to_string(), result);
            calculator.store_result(format!("{} = {}", var_name, expression), result);
            println!("{} = {}", var_name, self.base.format(result));
            Ok(Some(result))
        } else {
            // Normal expression evaluation
            let expr = calculator.parser.parse(input)?;
            let result = expr.evaluate(&calculator.variables)?;
            calculator.store_result(input.to_string(), result);
            println!("= {}", self.base.format(result));
            Ok(Some(result))
        }
    }

    fn available_operations(&self) -> Vec<&'static str> {
        vec!["+", "-", "*", "/", "AND", "OR", "XOR", "NOT", "SHL", "SHR"]
    }

    fn display_prompt(&self) -> String {
        match self.base {
            NumberBase::Binary => "[Programmer (BIN)] >".to_string(),
            NumberBase::Octal => "[Programmer (OCT)] >".to_string(),
            NumberBase::Decimal => "[Programmer (DEC)] >".to_string(),
            NumberBase::Hexadecimal => "[Programmer (HEX)] >".to_string(),
        }
    }

    fn as_any(&self) -> &dyn std::any::Any {
        self
    }
}

// Enum to represent different number bases for programmer mode
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum NumberBase {
    Binary,
    Octal,
    Decimal,
    Hexadecimal,
}

impl NumberBase {
    pub fn format(&self, value: f64) -> String {
        let value = value as i64; // Convert to integer for non-decimal bases
        match self {
            NumberBase::Binary => format!("0b{:b}", value),
            NumberBase::Octal => format!("0o{:o}", value),
            NumberBase::Decimal => format!("{}", value),
            NumberBase::Hexadecimal => format!("0x{:X}", value),
        }
    }

    pub fn parse(&self, text: &str) -> Result<f64, String> {
        match self {
            NumberBase::Binary => {
                if let Some(value) = text.strip_prefix("0b") {
                    i64::from_str_radix(value, 2)
                        .map(|v| v as f64)
                        .map_err(|_| format!("Invalid binary number: {}", text))
                } else {
                    Err(format!("Binary numbers must start with 0b: {}", text))
                }
            }
            NumberBase::Octal => {
                if let Some(value) = text.strip_prefix("0o") {
                    i64::from_str_radix(value, 8)
                        .map(|v| v as f64)
                        .map_err(|_| format!("Invalid octal number: {}", text))
                } else {
                    Err(format!("Octal numbers must start with 0o: {}", text))
                }
            }
            NumberBase::Decimal => text
                .parse::<f64>()
                .map_err(|_| format!("Invalid decimal number: {}", text)),
            NumberBase::Hexadecimal => {
                if let Some(value) = text.strip_prefix("0x") {
                    i64::from_str_radix(value, 16)
                        .map(|v| v as f64)
                        .map_err(|_| format!("Invalid hexadecimal number: {}", text))
                } else {
                    Err(format!("Hexadecimal numbers must start with 0x: {}", text))
                }
            }
        }
    }
}

// ======================================= //
// 2. Calculator context for state pattern //
// ======================================= //

pub struct StateCalculator {
    pub state: Box<dyn CalculatorState>,
    pub variables: HashMap<String, f64>,
    pub parser: ExpressionParser,
    pub results_history: Vec<(String, f64)>,
}

impl StateCalculator {
    pub fn new() -> Self {
        Self {
            state: Box::new(StandardMode::new()),
            variables: HashMap::new(),
            parser: ExpressionParser::new(),
            results_history: Vec::new(),
        }
    }

    pub fn change_state(&mut self, new_state: Box<dyn CalculatorState>) {
        println!("Switching to {} mode", new_state.name());
        self.state = new_state;
    }

    pub fn process_input(&mut self, input: &str) -> Result<Option<f64>, String> {
        // Create a cloned input to avoid lifetime issues
        let input_owned = input.to_string();

        // Get a reference to the current state implementation
        let state_type = self.state.name().to_string();

        // Use helper functions to process the input based on state type
        match state_type.as_str() {
            "Standard" => match_standard_input(&input_owned, self),
            "Scientific" => match_scientific_input(&input_owned, self),
            "Programmer" => match_programmer_input(&input_owned, self),
            _ => Err(format!("Unknown state type: {}", state_type)),
        }
    }

    pub fn store_result(&mut self, input: String, result: f64) {
        self.results_history.push((input, result));
        self.variables.insert("ans".to_string(), result);
    }

    pub fn display_prompt(&self) -> String {
        self.state.display_prompt()
    }
}

// ============================================= //
// Helper functions to avoid borrowing conflicts //
// ============================================= //

fn match_standard_input(
    input: &str,
    calculator: &mut StateCalculator,
) -> Result<Option<f64>, String> {
    // Replicate the standard mode logic to avoid borrowing issues
    if input.starts_with("mode") {
        // Change mode based on command
        let mode = input.trim_start_matches("mode").trim();
        match mode {
            "scientific" => {
                calculator.change_state(Box::new(ScientificMode::new()));
                Ok(None)
            }
            "programmer" => {
                calculator.change_state(Box::new(ProgrammerMode::new()));
                Ok(None)
            }
            _ => Err(format!("Unknown mode: {}", mode)),
        }
    } else if input.starts_with("help") {
        println!("Available operations: +, -, *, /, ^");
        println!("Type 'mode scientific' or 'mode programmer' to switch modes");
        Ok(None)
    } else if let Some((var_name, expression)) = input.split_once('=') {
        let var_name = var_name.trim();
        let expression = expression.trim();

        // Evaluate the expression and set the variable
        let expr = calculator.parser.parse(expression)?;
        let result = expr.evaluate(&calculator.variables)?;
        calculator.variables.insert(var_name.to_string(), result);
        calculator.store_result(format!("{} = {}", var_name, expression), result);
        Ok(Some(result))
    } else {
        // Normal expression evaluation
        let expr = calculator.parser.parse(input)?;
        let result = expr.evaluate(&calculator.variables)?;
        calculator.store_result(input.to_string(), result);
        Ok(Some(result))
    }
}

fn match_scientific_input(
    input: &str,
    calculator: &mut StateCalculator,
) -> Result<Option<f64>, String> {
    // Replicate the scientific mode logic
    if input.starts_with("mode") {
        // Handle mode change
        let mode = input.trim_start_matches("mode").trim();
        match mode {
            "standard" => {
                calculator.change_state(Box::new(StandardMode::new()));
                Ok(None)
            }
            "programmer" => {
                calculator.change_state(Box::new(ProgrammerMode::new()));
                Ok(None)
            }
            _ => Err(format!("Unknown mode: {}", mode)),
        }
    } else if input == "angle deg" {
        // Change angle mode to degrees
        calculator.change_state(Box::new(ScientificMode {
            sci_ops: Box::new(crate::adapter::StandardScientificOperations {
                angle_mode: AngleMode::Degrees,
            }),
            angle_mode: AngleMode::Degrees,
        }));
        println!("Angle mode set to degrees");
        Ok(None)
    } else if input == "angle rad" {
        // Change angle mode to radians
        calculator.change_state(Box::new(ScientificMode {
            sci_ops: Box::new(crate::adapter::StandardScientificOperations {
                angle_mode: AngleMode::Radians,
            }),
            angle_mode: AngleMode::Radians,
        }));
        println!("Angle mode set to radians");
        Ok(None)
    } else if input.starts_with("help") {
        println!("Available operations: +, -, *, /, ^, sin, cos, tan, log, ln, sqrt");
        println!("Type 'mode standard' or 'mode programmer' to switch modes");
        println!("Type 'angle deg' or 'angle rad' to change angle mode");
        Ok(None)
    } else {
        // For other scientific operations, use a generic approach
        // that doesn't depend on the ScientificMode specifics
        let expr = calculator.parser.parse(input)?;
        let result = expr.evaluate(&calculator.variables)?;
        calculator.store_result(input.to_string(), result);
        Ok(Some(result))
    }
}

fn match_programmer_input(
    input: &str,
    calculator: &mut StateCalculator,
) -> Result<Option<f64>, String> {
    // Simplified programmer mode logic
    if input.starts_with("mode") {
        // Handle mode change
        let mode = input.trim_start_matches("mode").trim();
        match mode {
            "standard" => {
                calculator.change_state(Box::new(StandardMode::new()));
                Ok(None)
            }
            "scientific" => {
                calculator.change_state(Box::new(ScientificMode::new()));
                Ok(None)
            }
            _ => Err(format!("Unknown mode: {}", mode)),
        }
    } else if input.starts_with("base") {
        // Change number base
        let base = input.trim_start_matches("base").trim();
        let new_base = match base {
            "bin" | "binary" => NumberBase::Binary,
            "oct" | "octal" => NumberBase::Octal,
            "dec" | "decimal" => NumberBase::Decimal,
            "hex" | "hexadecimal" => NumberBase::Hexadecimal,
            _ => return Err(format!("Unknown base: {}", base)),
        };

        calculator.change_state(Box::new(ProgrammerMode { base: new_base }));
        println!("Base set to {:?}", new_base);
        Ok(None)
    } else if input.starts_with("help") {
        println!("Available operations: +, -, *, /, AND, OR, XOR, NOT, SHL, SHR");
        println!("Type 'mode standard' or 'mode scientific' to switch modes");
        println!("Type 'base bin', 'base oct', 'base dec', or 'base hex' to change base");
        println!("Bitwise operations: AND, OR, XOR, NOT, SHL, SHR");
        Ok(None)
    } else {
        // Generic evaluation for other operations
        let expr = calculator.parser.parse(input)?;
        let result = expr.evaluate(&calculator.variables)?;
        calculator.store_result(input.to_string(), result);
        Ok(Some(result))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::adapter::StandardScientificOperations;
    use crate::config::AngleMode;

    // ---------- StateCalculator context ----------

    #[test]
    fn new_calculator_starts_in_standard_mode() {
        let calculator = StateCalculator::new();
        assert_eq!(calculator.state.name(), "Standard");
        assert_eq!(calculator.display_prompt(), "[Standard] >");
    }

    #[test]
    fn standard_mode_evaluates_expressions() {
        let mut calculator = StateCalculator::new();
        let result = calculator.process_input("2 + 3").unwrap();
        assert_eq!(result, Some(5.0));
        assert_eq!(calculator.results_history.len(), 1);
        assert_eq!(calculator.variables["ans"], 5.0);
    }

    #[test]
    fn standard_mode_handles_variable_assignment() {
        let mut calculator = StateCalculator::new();
        assert_eq!(calculator.process_input("x = 5").unwrap(), Some(5.0));
        assert_eq!(calculator.variables["x"], 5.0);
        assert_eq!(calculator.process_input("x + 1").unwrap(), Some(6.0));
    }

    #[test]
    fn standard_mode_uses_variables_in_expressions() {
        let mut calculator = StateCalculator::new();
        calculator.variables.insert("x".to_string(), 10.0);
        assert_eq!(calculator.process_input("x * 2").unwrap(), Some(20.0));
    }

    #[test]
    fn standard_mode_returns_error_for_undefined_variable() {
        let mut calculator = StateCalculator::new();
        assert!(calculator.process_input("unknown + 1").is_err());
    }

    #[test]
    fn switch_to_scientific_mode() {
        let mut calculator = StateCalculator::new();
        calculator.process_input("mode scientific").unwrap();
        assert_eq!(calculator.state.name(), "Scientific");
        assert!(calculator.display_prompt().contains("(RAD)"));
    }

    #[test]
    fn scientific_mode_evaluates_expression() {
        let mut calculator = StateCalculator::new();
        calculator.process_input("mode scientific").unwrap();
        let result = calculator.process_input("sin ( 1 )").unwrap().unwrap();
        assert!((result - 1.0_f64.sin()).abs() < 1e-10);
    }

    #[test]
    fn switch_to_programmer_mode() {
        let mut calculator = StateCalculator::new();
        calculator.process_input("mode programmer").unwrap();
        assert_eq!(calculator.state.name(), "Programmer");
    }

    #[test]
    fn programmer_mode_sets_number_base() {
        let mut calculator = StateCalculator::new();
        calculator.process_input("mode programmer").unwrap();
        calculator.process_input("base hex").unwrap();
        assert!(calculator.display_prompt().contains("(HEX)"));
    }

    #[test]
    fn switching_between_modes() {
        let mut calculator = StateCalculator::new();
        calculator.process_input("mode scientific").unwrap();
        calculator.process_input("angle deg").unwrap();
        assert!(calculator.display_prompt().contains("(DEG)"));
        calculator.process_input("mode standard").unwrap();
        assert_eq!(calculator.display_prompt(), "[Standard] >");
        calculator.process_input("mode programmer").unwrap();
        assert!(calculator.display_prompt().contains("(DEC)"));
    }

    #[test]
    fn unknown_mode_returns_error() {
        let mut calculator = StateCalculator::new();
        assert!(calculator.process_input("mode quantum").is_err());
    }

    #[test]
    fn store_result_sets_ans_variable() {
        let mut calculator = StateCalculator::new();
        calculator.store_result("2 + 3".to_string(), 5.0);
        assert_eq!(calculator.variables["ans"], 5.0);
        assert_eq!(calculator.results_history.len(), 1);
    }

    // ---------- StandardMode ----------

    #[test]
    fn standard_mode_handle_input_direct() {
        let mode = StandardMode::new();
        let mut calculator = StateCalculator::new();
        assert_eq!(mode.handle_input("2 + 3", &mut calculator).unwrap(), Some(5.0));
    }

    #[test]
    fn standard_mode_helper_reports_operations() {
        let mode = StandardMode::new();
        assert_eq!(mode.available_operations(), vec!["+", "-", "*", "/", "^"]);
    }

    // ---------- ScientificMode ----------

    #[test]
    fn scientific_mode_trig_in_radians() {
        let mode = ScientificMode::new();
        let mut calculator = StateCalculator::new();
        let result = mode.handle_input("cos 0", &mut calculator).unwrap().unwrap();
        assert!((result - 1.0).abs() < 1e-10);
    }

    #[test]
    fn scientific_mode_trig_in_degrees() {
        let mode = ScientificMode {
            sci_ops: Box::new(StandardScientificOperations {
                angle_mode: AngleMode::Degrees,
            }),
            angle_mode: AngleMode::Degrees,
        };
        let mut calculator = StateCalculator::new();
        let result = mode.handle_input("sin 90", &mut calculator).unwrap().unwrap();
        assert!((result - 1.0).abs() < 1e-10);
    }

    #[test]
    fn scientific_mode_log() {
        let mode = ScientificMode::new();
        let mut calculator = StateCalculator::new();
        let result = mode.handle_input("log 10 100", &mut calculator).unwrap().unwrap();
        assert!((result - 2.0).abs() < 1e-10);
    }

    #[test]
    fn scientific_mode_log_rejects_bad_arguments() {
        let mode = ScientificMode::new();
        let mut calculator = StateCalculator::new();
        assert!(mode.handle_input("log 10 -5", &mut calculator).is_err());
        assert!(mode.handle_input("log 1 10", &mut calculator).is_err());
    }

    #[test]
    fn scientific_mode_change_angle_via_command() {
        let mode = ScientificMode::new();
        let mut calculator = StateCalculator::new();
        mode.handle_input("angle deg", &mut calculator).unwrap();
        assert!(calculator.display_prompt().contains("(DEG)"));
    }

    // ---------- ProgrammerMode ----------

    #[test]
    fn programmer_mode_bitwise_and() {
        let mode = ProgrammerMode::new();
        let mut calculator = StateCalculator::new();
        assert_eq!(mode.handle_input("AND 5 3", &mut calculator).unwrap(), Some(1.0));
    }

    #[test]
    fn programmer_mode_bitwise_or() {
        let mode = ProgrammerMode::new();
        let mut calculator = StateCalculator::new();
        assert_eq!(mode.handle_input("OR 5 3", &mut calculator).unwrap(), Some(7.0));
    }

    #[test]
    fn programmer_mode_bitwise_xor() {
        let mode = ProgrammerMode::new();
        let mut calculator = StateCalculator::new();
        assert_eq!(mode.handle_input("XOR 5 3", &mut calculator).unwrap(), Some(6.0));
    }

    #[test]
    fn programmer_mode_bitwise_not() {
        let mode = ProgrammerMode::new();
        let mut calculator = StateCalculator::new();
        assert_eq!(mode.handle_input("NOT 0", &mut calculator).unwrap(), Some(-1.0));
    }

    #[test]
    fn programmer_mode_shift_left() {
        let mode = ProgrammerMode::new();
        let mut calculator = StateCalculator::new();
        assert_eq!(mode.handle_input("SHL 1 3", &mut calculator).unwrap(), Some(8.0));
    }

    #[test]
    fn programmer_mode_shift_right() {
        let mode = ProgrammerMode::new();
        let mut calculator = StateCalculator::new();
        assert_eq!(mode.handle_input("SHR 8 2", &mut calculator).unwrap(), Some(2.0));
    }

    #[test]
    fn programmer_mode_change_base() {
        let mode = ProgrammerMode::new();
        let mut calculator = StateCalculator::new();
        mode.handle_input("base bin", &mut calculator).unwrap();
        assert_eq!(calculator.state.name(), "Programmer");
        assert!(calculator.display_prompt().contains("(BIN)"));
    }

    #[test]
    fn programmer_mode_available_operations() {
        let mode = ProgrammerMode::new();
        assert_eq!(
            mode.available_operations(),
            vec!["+", "-", "*", "/", "AND", "OR", "XOR", "NOT", "SHL", "SHR"]
        );
    }

    // ---------- NumberBase ----------

    #[test]
    fn number_base_formats_decimal() {
        assert_eq!(NumberBase::Decimal.format(42.0), "42");
    }

    #[test]
    fn number_base_parses() {
        assert_eq!(NumberBase::Binary.parse("0b101").unwrap(), 5.0);
        assert_eq!(NumberBase::Octal.parse("0o10").unwrap(), 8.0);
        assert_eq!(NumberBase::Decimal.parse("42").unwrap(), 42.0);
        assert_eq!(NumberBase::Hexadecimal.parse("0xFF").unwrap(), 255.0);
    }

    #[test]
    fn number_base_parsing_requires_prefix() {
        assert!(NumberBase::Binary.parse("101").is_err());
        assert!(NumberBase::Octal.parse("10").is_err());
        assert!(NumberBase::Hexadecimal.parse("FF").is_err());
    }

    #[test]
    fn number_base_parsing_rejects_invalid() {
        assert!(NumberBase::Binary.parse("0b").is_err());
        assert!(NumberBase::Hexadecimal.parse("0xG").is_err());
        assert!(NumberBase::Decimal.parse("abc").is_err());
    }

    #[test]
    fn number_base_formats_non_decimal() {
        assert_eq!(NumberBase::Binary.format(5.0), "0b101");
        assert_eq!(NumberBase::Octal.format(8.0), "0o10");
        assert_eq!(NumberBase::Hexadecimal.format(255.0), "0xFF");
    }

    #[test]
    fn number_base_truncates_fractional_values() {
        assert_eq!(NumberBase::Binary.format(5.9), "0b101");
    }
}
