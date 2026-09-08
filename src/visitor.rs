// visitor.rs - Visitor pattern implementation for traversing and transforming expressions

use crate::expression::{
    BinaryOperation, Expression, ExpressionVisitor, FunctionCall, NumberExpression,
    VariableExpression,
};
use crate::token::{Function, Operator};
use std::collections::HashMap;

// ================================ //
// 4. Implement a Concrete Visitors //
// ================================ //

// ---------------------- //
// A. OptimizationVisitor //
// ---------------------- //

// Concrete visitor for optimizing expressions
pub struct OptimizationVisitor {
    variables: HashMap<String, f64>,
}

impl OptimizationVisitor {
    pub fn new(variables: HashMap<String, f64>) -> Self {
        Self { variables }
    }

    pub fn optimize(&mut self, expr: &dyn Expression) -> Result<Box<dyn Expression>, String> {
        expr.accept(self)
    }

    fn get_constant_value(&self, expr: &dyn Expression) -> Option<f64> {
        expr.as_any()
            .downcast_ref::<NumberExpression>()
            .map(|num_expr| num_expr.value)
    }
}

impl ExpressionVisitor for OptimizationVisitor {
    fn visit_number(&mut self, expr: &NumberExpression) -> Result<Box<dyn Expression>, String> {
        // Numbers are already optimized
        Ok(expr.clone_box())
    }

    fn visit_variable(
        &mut self,
        expr: &VariableExpression,
    ) -> Result<Box<dyn Expression>, String> {
        // If the variable has a known constant value, replace with a number
        if let Some(value) = self.variables.get(&expr.name) {
            Ok(Box::new(NumberExpression::new(*value)))
        } else {
            Ok(expr.clone_box())
        }
    }

    fn visit_binary_op(
        &mut self,
        expr: &BinaryOperation,
    ) -> Result<Box<dyn Expression>, String> {
        // Optimize left and right subexpressions
        let left_opt = expr.left.accept(self)?;
        let right_opt = expr.right.accept(self)?;

        // If both operands are constants, evaluate them
        if let (Some(left_val), Some(right_val)) = (
            self.get_constant_value(&*left_opt),
            self.get_constant_value(&*right_opt),
        ) {
            let result = match expr.operator {
                Operator::Add => left_val + right_val,
                Operator::Subtract => left_val - right_val,
                Operator::Multiply => left_val * right_val,
                Operator::Divide => {
                    if right_val == 0.0 {
                        return Err("Division by zero in optimization".to_string());
                    }
                    left_val / right_val
                }
                Operator::Power => left_val.powf(right_val),
            };

            Ok(Box::new(NumberExpression::new(result)))
        } else {
            // Some special cases for further optimization
            match expr.operator {
                Operator::Multiply => {
                    // Multiply by 0 = 0
                    if let Some(0.0) = self.get_constant_value(&*left_opt) {
                        return Ok(Box::new(NumberExpression::new(0.0)));
                    }
                    if let Some(0.0) = self.get_constant_value(&*right_opt) {
                        return Ok(Box::new(NumberExpression::new(0.0)));
                    }
                    // Multiply by 1 = other operand
                    if let Some(1.0) = self.get_constant_value(&*left_opt) {
                        return Ok(right_opt);
                    }
                    if let Some(1.0) = self.get_constant_value(&*right_opt) {
                        return Ok(left_opt);
                    }
                }
                Operator::Add => {
                    // Add 0 = other operand
                    if let Some(0.0) = self.get_constant_value(&*left_opt) {
                        return Ok(right_opt);
                    }
                    if let Some(0.0) = self.get_constant_value(&*right_opt) {
                        return Ok(left_opt);
                    }
                }
                Operator::Subtract => {
                    // Subtract 0 = left operand
                    if let Some(0.0) = self.get_constant_value(&*right_opt) {
                        return Ok(left_opt);
                    }
                }
                Operator::Divide => {
                    // Divide by 1 = left operand
                    if let Some(1.0) = self.get_constant_value(&*right_opt) {
                        return Ok(left_opt);
                    }
                    // Divide 0 by anything = 0
                    if let Some(0.0) = self.get_constant_value(&*left_opt) {
                        return Ok(Box::new(NumberExpression::new(0.0)));
                    }
                }
                Operator::Power => {
                    // Anything^0 = 1
                    if let Some(0.0) = self.get_constant_value(&*right_opt) {
                        return Ok(Box::new(NumberExpression::new(1.0)));
                    }
                    // Anything^1 = itself
                    if let Some(1.0) = self.get_constant_value(&*right_opt) {
                        return Ok(left_opt);
                    }
                    // 1^anything = 1
                    if let Some(1.0) = self.get_constant_value(&*left_opt) {
                        return Ok(Box::new(NumberExpression::new(1.0)));
                    }
                }
            }

            // Cannot fully optimize, create a new operation with optimized operands
            Ok(Box::new(BinaryOperation::new(
                left_opt,
                right_opt,
                expr.operator.clone(),
            )))
        }
    }

    fn visit_function_call(
        &mut self,
        expr: &FunctionCall,
    ) -> Result<Box<dyn Expression>, String> {
        // Optimize the argument
        let arg_opt = expr.argument.accept(self)?;

        // If the argument is a constant, evaluate the function
        if let Some(arg_val) = self.get_constant_value(&*arg_opt) {
            let result = match expr.function {
                Function::Sin => arg_val.sin(),
                Function::Cos => arg_val.cos(),
                Function::Tan => {
                    if (arg_val - std::f64::consts::PI / 2.0).abs() % std::f64::consts::PI < 1e-10 {
                        return Err("Tangent undefined at this value".to_string());
                    }
                    arg_val.tan()
                }
                Function::Sqrt => {
                    if arg_val < 0.0 {
                        return Err("Cannot take square root of negative number".to_string());
                    }
                    arg_val.sqrt()
                }
            };

            Ok(Box::new(NumberExpression::new(result)))
        } else {
            // Cannot optimize, create a new function call with optimized argument
            Ok(Box::new(FunctionCall::new(expr.function.clone(), arg_opt)))
        }
    }
}

// -------------------- //
// B. ValidationVisitor //
// -------------------- //

// Concrete visitor for validating expressions
pub struct ValidationVisitor {
    pub errors: Vec<String>,
}

impl ValidationVisitor {
    pub fn new() -> Self {
        Self { errors: Vec::new() }
    }

    pub fn validate(&mut self, expr: &dyn Expression) -> Result<(), String> {
        expr.accept(self)?;

        if self.errors.is_empty() {
            Ok(())
        } else {
            Err(self.errors.join("; "))
        }
    }
}

impl ExpressionVisitor for ValidationVisitor {
    fn visit_number(&mut self, expr: &NumberExpression) -> Result<Box<dyn Expression>, String> {
        // Numbers are always valid
        Ok(expr.clone_box())
    }

    fn visit_variable(
        &mut self,
        expr: &VariableExpression,
    ) -> Result<Box<dyn Expression>, String> {
        // Variables are assumed to be valid (could add name validation here)
        Ok(expr.clone_box())
    }

    fn visit_binary_op(
        &mut self,
        expr: &BinaryOperation,
    ) -> Result<Box<dyn Expression>, String> {
        // Check for division by zero in constant expressions
        if let Operator::Divide = expr.operator
            && let Some(right) = expr.right.as_any().downcast_ref::<NumberExpression>()
            && right.value == 0.0
        {
            self.errors.push("Division by zero".to_string());
        }

        // Validate operands
        expr.left.accept(self)?;
        expr.right.accept(self)?;

        Ok(expr.clone_box())
    }

    fn visit_function_call(
        &mut self,
        expr: &FunctionCall,
    ) -> Result<Box<dyn Expression>, String> {
        // Validate function arguments
        match expr.function {
            Function::Sqrt => {
                if let Some(arg) = expr.argument.as_any().downcast_ref::<NumberExpression>()
                    && arg.value < 0.0
                {
                    self.errors
                        .push("Cannot take square root of negative number".to_string());
                }
            }
            Function::Tan => {
                if let Some(arg) = expr.argument.as_any().downcast_ref::<NumberExpression>() {
                    let value = arg.value;
                    if (value - std::f64::consts::PI / 2.0).abs() % std::f64::consts::PI < 1e-10 {
                        self.errors
                            .push("Tangent undefined at this value".to_string());
                    }
                }
            }
            _ => {}
        }

        // Validate argument
        expr.argument.accept(self)?;

        Ok(expr.clone_box())
    }
}

// ---------------- //
// Helper Functions //
// ---------------- //

// Function to optimize an expression
pub fn optimize_expression(
    expr: &dyn Expression,
    variables: &HashMap<String, f64>,
) -> Result<Box<dyn Expression>, String> {
    let mut visitor = OptimizationVisitor::new(variables.clone());
    visitor.optimize(expr)
}

// Function to validate an expression
pub fn validate_expression(expr: &dyn Expression) -> Result<(), String> {
    let mut visitor = ValidationVisitor::new();
    visitor.validate(expr)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::expression::{Expression, NumberExpression, VariableExpression};
    use crate::parser::ExpressionParser;
    use std::collections::HashMap;

    fn parse(expression: &str) -> Box<dyn Expression> {
        ExpressionParser::new().parse(expression).unwrap()
    }

    // ---------- optimize_expression ----------

    #[test]
    fn optimize_folds_constant_expressions() {
        let expression = parse("2 + 3");
        let optimized = optimize_expression(&*expression, &HashMap::new()).unwrap();

        assert!(
            optimized
                .as_any()
                .downcast_ref::<NumberExpression>()
                .is_some()
        );
        assert_eq!(optimized.evaluate(&HashMap::new()).unwrap(), 5.0);
    }

    #[test]
    fn optimize_substitutes_known_variables() {
        let expression = parse("x + 1");
        let variables = HashMap::from([("x".to_string(), 10.0)]);
        let optimized = optimize_expression(&*expression, &variables).unwrap();

        assert_eq!(optimized.evaluate(&HashMap::new()).unwrap(), 11.0);
    }

    #[test]
    fn optimize_keeps_unknown_variables() {
        let expression = parse("x + y");
        let optimized = optimize_expression(&*expression, &HashMap::new()).unwrap();

        let variables = HashMap::from([("x".to_string(), 1.0), ("y".to_string(), 2.0)]);
        assert_eq!(optimized.evaluate(&variables).unwrap(), 3.0);
    }

    #[test]
    fn optimize_identity_add_zero() {
        let expression = parse("x + 0");
        let optimized = optimize_expression(&*expression, &HashMap::new()).unwrap();

        assert!(
            optimized
                .as_any()
                .downcast_ref::<VariableExpression>()
                .is_some()
        );
        let variables = HashMap::from([("x".to_string(), 7.0)]);
        assert_eq!(optimized.evaluate(&variables).unwrap(), 7.0);
    }

    #[test]
    fn optimize_multiply_by_zero() {
        let expression = parse("5 * 0");
        let optimized = optimize_expression(&*expression, &HashMap::new()).unwrap();
        assert_eq!(optimized.evaluate(&HashMap::new()).unwrap(), 0.0);
    }

    #[test]
    fn optimize_multiply_by_one() {
        let expression = parse("x * 1");
        let optimized = optimize_expression(&*expression, &HashMap::new()).unwrap();

        assert!(
            optimized
                .as_any()
                .downcast_ref::<VariableExpression>()
                .is_some()
        );
    }

    #[test]
    fn optimize_power_to_zero() {
        let expression = parse("x ^ 0");
        let optimized = optimize_expression(&*expression, &HashMap::new()).unwrap();
        assert_eq!(optimized.evaluate(&HashMap::new()).unwrap(), 1.0);
    }

    #[test]
    fn optimize_constant_function_call() {
        let expression = parse("sqrt ( 4 )");
        let optimized = optimize_expression(&*expression, &HashMap::new()).unwrap();
        assert_eq!(optimized.evaluate(&HashMap::new()).unwrap(), 2.0);
    }

    #[test]
    fn optimize_division_by_zero_errors() {
        let expression = parse("5 / 0");
        assert!(optimize_expression(&*expression, &HashMap::new()).is_err());
    }

    // ---------- validate_expression ----------

    #[test]
    fn validate_accepts_valid_expression() {
        let expression = parse("2 + 3");
        assert!(validate_expression(&*expression).is_ok());
    }

    #[test]
    fn validate_accepts_variables() {
        let expression = parse("x * 2");
        assert!(validate_expression(&*expression).is_ok());
    }

    #[test]
    fn validate_detects_division_by_zero() {
        let expression = parse("2 / 0");
        assert!(validate_expression(&*expression).is_err());
    }

    #[test]
    fn validate_detects_sqrt_of_negative_constant() {
        let expression = FunctionCall::new(
            Function::Sqrt,
            Box::new(NumberExpression::new(-4.0)),
        );
        match validate_expression(&expression as &dyn Expression) {
            Err(message) => assert!(message.contains("square root")),
            Ok(()) => panic!("expected validation error"),
        }
    }

    #[test]
    fn validate_detects_tangent_undefined() {
        let expression = FunctionCall::new(
            Function::Tan,
            Box::new(NumberExpression::new(std::f64::consts::PI / 2.0)),
        );
        assert!(validate_expression(&expression as &dyn Expression).is_err());
    }

    #[test]
    fn validate_accepts_valid_sqrt() {
        let expression = FunctionCall::new(
            Function::Sqrt,
            Box::new(NumberExpression::new(4.0)),
        );
        assert!(validate_expression(&expression as &dyn Expression).is_ok());
    }

    // ---------- accept() double dispatch ----------

    #[test]
    fn accept_dispatches_to_number_visit() {
        let number = NumberExpression::new(1.0);
        let mut visitor = ValidationVisitor::new();
        assert!(number.accept(&mut visitor).is_ok());
        assert!(visitor.errors.is_empty());
    }

    #[test]
    fn accept_dispatches_to_function_call_visit() {
        let function = FunctionCall::new(
            Function::Sqrt,
            Box::new(NumberExpression::new(-1.0)),
        );
        let mut visitor = ValidationVisitor::new();
        assert!(function.accept(&mut visitor).is_ok());
        assert_eq!(visitor.errors.len(), 1);
    }

    // ---------- OptimizationVisitor ----------

    #[test]
    fn optimization_visitor_is_reusable() {
        let mut visitor = OptimizationVisitor::new(HashMap::new());
        let a = visitor.optimize(&*parse("1 + 2")).unwrap();
        let b = visitor.optimize(&*parse("3 * 4")).unwrap();
        assert_eq!(a.evaluate(&HashMap::new()).unwrap(), 3.0);
        assert_eq!(b.evaluate(&HashMap::new()).unwrap(), 12.0);
    }
}