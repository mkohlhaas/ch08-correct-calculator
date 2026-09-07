// iterator.rs - Iterator pattern implementation for collections in the calculator

// Several iterators for history, reverse history and variables

use crate::expression::{
    BinaryOperation, Expression, FunctionCall, NumberExpression, VariableExpression,
};
use std::collections::HashMap;

// =================== //
// A. History Iterator //
// =================== //

// Since slices already implement Iterator in Rust, HistoryIterator is redundant. You can replace it with slice.iter().
// see cargo project `iterator-pattern`

// =========================== //
// B. Reverse History Iterator //
// =========================== //

// NOTE: HistoryIterator and ReverseHistoryIterator were removed because slices
// natively support iteration via slice.iter() and reverse iteration via
// slice.iter().rev().

// =========================== //
// C. Variables Iterator      //
// =========================== //

// Variables map iterator
pub struct VariablesIterator<'a> {
    inner: std::collections::hash_map::Iter<'a, String, f64>,
}

impl<'a> VariablesIterator<'a> {
    pub fn new(variables: &'a HashMap<String, f64>) -> Self {
        Self {
            inner: variables.iter(),
        }
    }
}

impl<'a> Iterator for VariablesIterator<'a> {
    type Item = (&'a String, &'a f64);

    fn next(&mut self) -> Option<Self::Item> {
        self.inner.next()
    }
}

// =========================== //
// Traversing expression trees //
// =========================== //

struct ExpressionIterator<'a> {
    stack: Vec<&'a dyn Expression>,
}
impl<'a> ExpressionIterator<'a> {
    fn new(root: &'a dyn Expression) -> Self {
        let stack = vec![root];
        Self { stack }
    }
}

// Code does not work!!!

// Google AI:
//
// Trait Object Method Dispatch: ExpressionIterator calls node.as_binary_op(). However, as_binary_op
// is implemented for dyn Expression to always return None. The overridden methods on concrete
// structs like BinaryOperation are completely bypassed when using a trait object (&dyn Expression),
// because Rust does not support structural downcasting or automatic virtual dispatch for extension
// traits this way.

// Google AI advices to use the visitor pattern.

impl<'a> Iterator for ExpressionIterator<'a> {
    type Item = &'a dyn Expression;

    fn next(&mut self) -> Option<Self::Item> {
        if let Some(node) = self.stack.pop() {
            // NOTE: Wrong!
            // Push children onto stack for depth-first traversal
            // as_binary_op() is a downcast method on Expression that
            // returns Some(&BinaryOperation) if the expression is a
            // binary operation, or None otherwise. as_function() works
            // similarly for FunctionCall.
            if let Some(op) = node.as_binary_op() {
                self.stack.push(&*op.right); // dereference-then-borrow pattern for Box
                self.stack.push(&*op.left);
            } else if let Some(func) = node.as_function() {
                self.stack.push(&*func.argument);
            }
            Some(node)
        } else {
            None
        }
    }
}

// Downcasting and Safety
//
// The code relies on safe downcasting patterns (as_binary_op, as_function).
// This is a common workaround in Rust because standard trait objects do not natively support
// downcasting without explicit helper methods or the use of Any.

// Extension trait for expression tree traversal
pub trait ExpressionExt {
    fn as_binary_op(&self) -> Option<&BinaryOperation> {
        None
    }
    fn as_number(&self) -> Option<&NumberExpression> {
        None
    }
    fn as_variable(&self) -> Option<&VariableExpression> {
        None
    }
    fn as_function(&self) -> Option<&FunctionCall> {
        None
    }
    fn is_constant(&self) -> bool {
        self.as_number().is_some()
    }
}

// NOTE: that was the change `+ 'a`
// But the other implementations won't be called!
impl<'a> ExpressionExt for dyn Expression + 'a {
    fn as_binary_op(&self) -> Option<&BinaryOperation> {
        None
    }
    fn as_number(&self) -> Option<&NumberExpression> {
        None
    }
    fn as_variable(&self) -> Option<&VariableExpression> {
        None
    }
    fn as_function(&self) -> Option<&FunctionCall> {
        None
    }
}

// NOTE: These functions will never be called.
impl ExpressionExt for BinaryOperation {
    fn as_binary_op(&self) -> Option<&BinaryOperation> {
        Some(self)
    }
}

impl ExpressionExt for NumberExpression {
    fn as_number(&self) -> Option<&NumberExpression> {
        Some(self)
    }
    fn is_constant(&self) -> bool {
        true
    }
}

impl ExpressionExt for VariableExpression {
    fn as_variable(&self) -> Option<&VariableExpression> {
        Some(self)
    }
}

impl ExpressionExt for FunctionCall {
    fn as_function(&self) -> Option<&FunctionCall> {
        Some(self)
    }
}

// Non-recursive approach to collecting expressions
pub fn find_constant_nodes(expr: &dyn Expression) -> Vec<Box<dyn Expression>> {
    let mut result = Vec::new();
    collect_nodes_by_type(expr, NodeType::Constant, &mut result);
    result
}

pub fn find_variable_nodes(expr: &dyn Expression) -> Vec<Box<dyn Expression>> {
    let mut result = Vec::new();
    collect_nodes_by_type(expr, NodeType::Variable, &mut result);
    result
}

#[derive(PartialEq, Eq, Clone, Copy)]
enum NodeType {
    Constant,
    Variable,
}

// Helper function to collect nodes by type without using an iterator
fn collect_nodes_by_type(
    expr: &dyn Expression,
    node_type: NodeType,
    result: &mut Vec<Box<dyn Expression>>,
) {
    if let Some(op) = expr.as_any().downcast_ref::<BinaryOperation>() {
        // Check if the node matches the criteria
        match node_type {
            NodeType::Constant => {
                if op.is_constant() {
                    result.push(op.clone_box());
                }
            }
            NodeType::Variable => {
                if op.as_variable().is_some() {
                    result.push(op.clone_box());
                }
            }
        }

        // Process children recursively
        collect_nodes_by_type(&*op.left, node_type, result);
        collect_nodes_by_type(&*op.right, node_type, result);
    } else if let Some(func) = expr.as_any().downcast_ref::<FunctionCall>() {
        // Check if the node matches the criteria
        match node_type {
            NodeType::Constant => {
                if func.is_constant() {
                    result.push(func.clone_box());
                }
            }
            NodeType::Variable => {
                if func.as_variable().is_some() {
                    result.push(func.clone_box());
                }
            }
        }

        // Process argument recursively
        collect_nodes_by_type(&*func.argument, node_type, result);
    } else if let Some(num) = expr.as_any().downcast_ref::<NumberExpression>() {
        if node_type == NodeType::Constant {
            result.push(num.clone_box());
        }
    } else if let Some(var) = expr.as_any().downcast_ref::<VariableExpression>()
        && node_type == NodeType::Variable
    {
        result.push(var.clone_box());
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::expression::Expression;
    use crate::parser::ExpressionParser;
    use crate::token::Operator;

    fn parse(expression: &str) -> Box<dyn Expression> {
        ExpressionParser::new().parse(expression).unwrap()
    }

    #[test]
    fn variables_iterator_yields_all_entries() {
        let mut variables = HashMap::new();
        variables.insert("x".to_string(), 1.0);
        variables.insert("y".to_string(), 2.0);
        variables.insert("z".to_string(), 3.0);

        let collected: HashMap<String, f64> = VariablesIterator::new(&variables)
            .map(|(name, value)| (name.clone(), *value))
            .collect();

        assert_eq!(collected.len(), 3);
        assert_eq!(collected["x"], 1.0);
        assert_eq!(collected["y"], 2.0);
        assert_eq!(collected["z"], 3.0);
    }

    #[test]
    fn variables_iterator_is_empty_for_empty_map() {
        let variables = HashMap::new();
        assert_eq!(VariablesIterator::new(&variables).count(), 0);
    }

    #[test]
    fn find_constant_nodes_returns_number_leaves() {
        let expression = parse("1 + 2");
        let constants = find_constant_nodes(&*expression);

        assert_eq!(constants.len(), 2);
        let mut values: Vec<f64> = constants
            .iter()
            .map(|node| {
                node.as_any()
                    .downcast_ref::<NumberExpression>()
                    .unwrap()
                    .value
            })
            .collect();
        values.sort_by(f64::total_cmp);
        assert_eq!(values, vec![1.0, 2.0]);
    }

    #[test]
    fn find_constant_nodes_handles_single_number() {
        let expression = parse("42");
        let constants = find_constant_nodes(&*expression);

        assert_eq!(constants.len(), 1);
        let value = constants[0]
            .as_any()
            .downcast_ref::<NumberExpression>()
            .unwrap()
            .value;
        assert_eq!(value, 42.0);
    }

    #[test]
    fn find_constant_nodes_does_not_return_variables() {
        let expression = parse("1 + x");
        let constants = find_constant_nodes(&*expression);
        assert_eq!(constants.len(), 1);
    }

    #[test]
    fn find_variable_nodes_returns_variable_leaves() {
        let expression = parse("x + y");
        let variables = find_variable_nodes(&*expression);

        assert_eq!(variables.len(), 2);
        let names: Vec<String> = variables
            .iter()
            .map(|node| {
                node.as_any()
                    .downcast_ref::<VariableExpression>()
                    .unwrap()
                    .name
                    .clone()
            })
            .collect();
        assert!(names.contains(&"x".to_string()));
        assert!(names.contains(&"y".to_string()));
    }

    #[test]
    fn find_variable_nodes_ignores_constants() {
        let expression = parse("1 + x");
        let variables = find_variable_nodes(&*expression);

        assert_eq!(variables.len(), 1);
        let name = variables[0]
            .as_any()
            .downcast_ref::<VariableExpression>()
            .unwrap()
            .name
            .clone();
        assert_eq!(name, "x");
    }

    #[test]
    fn expression_ext_classifies_concrete_types() {
        let number = NumberExpression::new(5.0);
        assert!(number.is_constant());
        assert!(number.as_number().is_some());
        assert!(number.as_variable().is_none());

        let variable = VariableExpression::new("pi");
        assert!(!variable.is_constant());
        assert!(variable.as_variable().is_some());

        let op = BinaryOperation::new(
            Box::new(NumberExpression::new(1.0)),
            Box::new(NumberExpression::new(2.0)),
            Operator::Add,
        );
        assert!(op.as_binary_op().is_some());
        assert!(op.as_number().is_none());
        assert!(!op.is_constant());

        let function = FunctionCall::new(
            crate::token::Function::Sqrt,
            Box::new(NumberExpression::new(4.0)),
        );
        assert!(function.as_function().is_some());
    }

    #[test]
    fn expression_ext_trait_object_dispatch_returns_none() {
        // Documented limitation: calling the extension methods through a
        // `dyn Expression` trait object hits the default impl that always
        // returns None -- the concrete impls are never reached.
        let op = parse("2 * 3");
        let dyn_op: &dyn Expression = &*op;
        assert!(dyn_op.as_binary_op().is_none());
        assert!(dyn_op.as_number().is_none());
    }

    #[test]
    fn expression_iterator_yields_at_least_the_root() {
        let expression = parse("1 + 2");
        let visited: Vec<&dyn Expression> = ExpressionIterator::new(&*expression).collect();
        assert!(!visited.is_empty());
        assert_eq!(visited[0].to_string(), expression.to_string());
    }
}
