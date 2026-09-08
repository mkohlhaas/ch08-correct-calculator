// expression.rs - Expression tree implementation (from Chapter 6)

use crate::token::{Function, Operator};
use std::any::Any;
use std::collections::HashMap;

// Expression trait defining common behavior
pub trait Expression: Send + Sync {
    fn evaluate(&self, variables: &HashMap<String, f64>) -> Result<f64, String>;
    fn to_string(&self) -> String;

    // For debugging and visualization
    fn precedence(&self) -> u8 {
        0 // Leaf nodes have lowest precedence by default
    }

    // Allow downcasting for visitor pattern
    fn as_any(&self) -> &dyn Any;

    // Typed downcast helpers built on as_any() so they dispatch correctly
    // through the dyn Expression vtable.
    fn as_number(&self) -> Option<&NumberExpression> {
        self.as_any().downcast_ref::<NumberExpression>()
    }
    fn as_variable(&self) -> Option<&VariableExpression> {
        self.as_any().downcast_ref::<VariableExpression>()
    }
    fn as_binary_op(&self) -> Option<&BinaryOperation> {
        self.as_any().downcast_ref::<BinaryOperation>()
    }
    fn as_function(&self) -> Option<&FunctionCall> {
        self.as_any().downcast_ref::<FunctionCall>()
    }
    fn is_number(&self) -> bool {
        self.as_number().is_some()
    }
    fn is_variable(&self) -> bool {
        self.as_variable().is_some()
    }
    fn is_binary_op(&self) -> bool {
        self.as_binary_op().is_some()
    }
    fn is_function(&self) -> bool {
        self.as_function().is_some()
    }

    fn accept(&self, visitor: &mut dyn ExpressionVisitor) -> Result<Box<dyn Expression>, String>;

    // Default implementation for cloning
    fn clone_box(&self) -> Box<dyn Expression>;
}

// Visitor interface for the Visitor pattern (double dispatch).
// Defined here alongside Expression so accept() can reference it without a module cycle.
pub trait ExpressionVisitor {
    fn visit_number(&mut self, expr: &NumberExpression) -> Result<Box<dyn Expression>, String>;
    fn visit_variable(&mut self, expr: &VariableExpression) -> Result<Box<dyn Expression>, String>;
    fn visit_binary_op(&mut self, expr: &BinaryOperation) -> Result<Box<dyn Expression>, String>;
    fn visit_function_call(&mut self, expr: &FunctionCall) -> Result<Box<dyn Expression>, String>;
}

// Extension to allow cloning of trait objects
impl Clone for Box<dyn Expression> {
    fn clone(&self) -> Self {
        self.clone_box()
    }
}

// ToOwned for trait objects delegates to clone_box
impl ToOwned for dyn Expression {
    type Owned = Box<dyn Expression>;

    fn to_owned(&self) -> Self::Owned {
        self.clone_box()
    }
}

// Leaf node for number values
#[derive(Debug, Clone)]
pub struct NumberExpression {
    pub value: f64,
}

impl NumberExpression {
    pub fn new(value: f64) -> Self {
        Self { value }
    }
}

impl Expression for NumberExpression {
    fn evaluate(&self, _variables: &HashMap<String, f64>) -> Result<f64, String> {
        Ok(self.value)
    }

    fn to_string(&self) -> String {
        format!("{}", self.value)
    }

    fn as_any(&self) -> &dyn Any {
        self
    }

    fn accept(&self, visitor: &mut dyn ExpressionVisitor) -> Result<Box<dyn Expression>, String> {
        visitor.visit_number(self)
    }

    fn clone_box(&self) -> Box<dyn Expression> {
        Box::new(self.clone())
    }
}

// Leaf node for variables
#[derive(Debug, Clone)]
pub struct VariableExpression {
    pub name: String,
}

impl VariableExpression {
    pub fn new(name: impl Into<String>) -> Self {
        Self { name: name.into() }
    }
}

impl Expression for VariableExpression {
    fn evaluate(&self, variables: &HashMap<String, f64>) -> Result<f64, String> {
        variables
            .get(&self.name)
            .copied()
            .ok_or_else(|| format!("Undefined variable: {}", self.name))
    }

    fn to_string(&self) -> String {
        self.name.clone()
    }

    fn as_any(&self) -> &dyn Any {
        self
    }

    fn accept(&self, visitor: &mut dyn ExpressionVisitor) -> Result<Box<dyn Expression>, String> {
        visitor.visit_variable(self)
    }

    fn clone_box(&self) -> Box<dyn Expression> {
        Box::new(self.clone())
    }
}

// Composite node for binary operations
// Cannot derive Debug for Box<dyn Expression>, so implement it manually
pub struct BinaryOperation {
    pub left: Box<dyn Expression>,
    pub right: Box<dyn Expression>,
    pub operator: Operator,
}

impl std::fmt::Debug for BinaryOperation {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("BinaryOperation")
            .field("left", &self.left.to_string())
            .field("right", &self.right.to_string())
            .field("operator", &self.operator)
            .finish()
    }
}

impl Clone for BinaryOperation {
    fn clone(&self) -> Self {
        Self {
            left: self.left.clone_box(),
            right: self.right.clone_box(),
            operator: self.operator.clone(),
        }
    }
}

impl BinaryOperation {
    pub fn new(left: Box<dyn Expression>, right: Box<dyn Expression>, operator: Operator) -> Self {
        Self {
            left,
            right,
            operator,
        }
    }

    fn operator_symbol(&self) -> &'static str {
        match self.operator {
            Operator::Add => "+",
            Operator::Subtract => "-",
            Operator::Multiply => "*",
            Operator::Divide => "/",
            Operator::Power => "^",
        }
    }
}

impl Expression for BinaryOperation {
    fn evaluate(&self, variables: &HashMap<String, f64>) -> Result<f64, String> {
        let left_val = self.left.evaluate(variables)?;
        let right_val = self.right.evaluate(variables)?;

        match self.operator {
            Operator::Add => Ok(left_val + right_val),
            Operator::Subtract => Ok(left_val - right_val),
            Operator::Multiply => Ok(left_val * right_val),
            Operator::Divide => {
                if right_val == 0.0 {
                    Err("Division by zero".to_string())
                } else {
                    Ok(left_val / right_val)
                }
            }
            Operator::Power => Ok(left_val.powf(right_val)),
        }
    }

    fn to_string(&self) -> String {
        let left_str = if self.left.precedence() < self.precedence() {
            format!("({})", self.left.to_string())
        } else {
            self.left.to_string()
        };

        let right_str = if self.right.precedence() < self.precedence() {
            format!("({})", self.right.to_string())
        } else {
            self.right.to_string()
        };

        format!("{} {} {}", left_str, self.operator_symbol(), right_str)
    }

    fn precedence(&self) -> u8 {
        match self.operator {
            Operator::Add | Operator::Subtract => 1,
            Operator::Multiply | Operator::Divide => 2,
            Operator::Power => 3,
        }
    }

    fn as_any(&self) -> &dyn Any {
        self
    }

    fn accept(&self, visitor: &mut dyn ExpressionVisitor) -> Result<Box<dyn Expression>, String> {
        visitor.visit_binary_op(self)
    }

    fn clone_box(&self) -> Box<dyn Expression> {
        Box::new(self.clone())
    }
}

// Function call expression
// Implement Debug and Clone manually for FunctionCall
pub struct FunctionCall {
    pub function: Function,
    pub argument: Box<dyn Expression>,
}

impl std::fmt::Debug for FunctionCall {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("FunctionCall")
            .field("function", &self.function)
            .field("argument", &self.argument.to_string())
            .finish()
    }
}

impl Clone for FunctionCall {
    fn clone(&self) -> Self {
        Self {
            function: self.function.clone(),
            argument: self.argument.clone_box(),
        }
    }
}

impl FunctionCall {
    pub fn new(function: Function, argument: Box<dyn Expression>) -> Self {
        Self { function, argument }
    }
}

impl Expression for FunctionCall {
    fn evaluate(&self, variables: &HashMap<String, f64>) -> Result<f64, String> {
        let arg_val = self.argument.evaluate(variables)?;

        match self.function {
            Function::Sin => Ok(arg_val.sin()),
            Function::Cos => Ok(arg_val.cos()),
            Function::Tan => {
                if (arg_val - std::f64::consts::PI / 2.0).abs() % std::f64::consts::PI < 1e-10 {
                    Err("Tangent undefined at this value".to_string())
                } else {
                    Ok(arg_val.tan())
                }
            }
            Function::Sqrt => {
                if arg_val < 0.0 {
                    Err("Cannot take square root of negative number".to_string())
                } else {
                    Ok(arg_val.sqrt())
                }
            }
        }
    }

    fn to_string(&self) -> String {
        let func_name = match self.function {
            Function::Sin => "sin",
            Function::Cos => "cos",
            Function::Tan => "tan",
            Function::Sqrt => "sqrt",
        };

        format!("{}({})", func_name, self.argument.to_string())
    }

    fn precedence(&self) -> u8 {
        4 // Function calls have highest precedence
    }

    fn as_any(&self) -> &dyn Any {
        self
    }

    fn accept(&self, visitor: &mut dyn ExpressionVisitor) -> Result<Box<dyn Expression>, String> {
        visitor.visit_function_call(self)
    }

    fn clone_box(&self) -> Box<dyn Expression> {
        Box::new(self.clone())
    }
}
