// iterator.rs - Iterator pattern implementation for collections in the calculator

// Several iterators for history, reverse history and variables

use crate::command::Calculation;
use crate::expression::{
    BinaryOperation, Expression, FunctionCall, NumberExpression, VariableExpression,
};
use std::collections::HashMap;

// =================== //
// A. History Iterator //
// =================== //

// see cargo project `iterator-pattern` for an alternative implementation

// History iterator that provides access to past results
pub struct HistoryIterator<'a> {
    history: &'a [Calculation],
    position: usize,
}

impl<'a> HistoryIterator<'a> {
    pub fn new(history: &'a [Calculation]) -> Self {
        Self {
            history,
            position: 0,
        }
    }
}

impl<'a> Iterator for HistoryIterator<'a> {
    type Item = &'a Calculation;

    fn next(&mut self) -> Option<Self::Item> {
        if self.position < self.history.len() {
            let item = &self.history[self.position];
            self.position += 1;
            Some(item)
        } else {
            None
        }
    }
}

// =========================== //
// B. Reverse History Iterator //
// =========================== //

// A reverse iterator for the history
pub struct ReverseHistoryIterator<'a> {
    history: &'a [Calculation],
    position: usize,
}

impl<'a> ReverseHistoryIterator<'a> {
    pub fn new(history: &'a [Calculation]) -> Self {
        Self {
            history,
            position: history.len(),
        }
    }
}

impl<'a> Iterator for ReverseHistoryIterator<'a> {
    type Item = &'a Calculation;

    fn next(&mut self) -> Option<Self::Item> {
        if self.position > 0 {
            self.position -= 1;
            Some(&self.history[self.position])
        } else {
            None
        }
    }
}

// ===================== //
// C. Variables Iterator //
// ===================== //

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
