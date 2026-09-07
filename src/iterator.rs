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

impl<'a> Iterator for ExpressionIterator<'a> {
    type Item = &'a dyn Expression;

    fn next(&mut self) -> Option<Self::Item> {
        if let Some(node) = self.stack.pop() {
            // Push children onto the stack for depth-first traversal.
            // as_any().downcast_ref::<T>() downcasts the dyn Expression
            // trait object to a concrete node type to reach its children.
            //
            // In case of BinaryOperation and FunctionCall we extend the stack with its arguments.
            // 'as_any().downcast_ref::<T>()' is idiomatic Rust.
            if let Some(op) = node.as_any().downcast_ref::<BinaryOperation>() {
                // the dereference-then-borrow pattern (&*) is common when working with Box in Rust.
                self.stack.push(&*op.right); // dereference-then-borrow pattern for Box
                self.stack.push(&*op.left);
            } else if let Some(func) = node.as_any().downcast_ref::<FunctionCall>() {
                self.stack.push(&*func.argument);
            }
            Some(node) // NumberExpression or VariableExpression
        } else {
            None
        }
    }
}

// Collect nodes of a given type via ExpressionIterator
pub fn find_constant_nodes(expr: &dyn Expression) -> Vec<Box<dyn Expression>> {
    ExpressionIterator::new(expr)
        .filter(|node| node.as_any().downcast_ref::<NumberExpression>().is_some())
        .map(|node| node.clone_box())
        .collect()
}

pub fn find_variable_nodes(expr: &dyn Expression) -> Vec<Box<dyn Expression>> {
    ExpressionIterator::new(expr)
        .filter(|node| node.as_any().downcast_ref::<VariableExpression>().is_some())
        .map(|node| node.clone_box())
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::expression::Expression;
    use crate::parser::ExpressionParser;

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
    fn expression_iterator_traverses_binary_tree() {
        let expression = parse("1 + 2");
        let visited: Vec<&dyn Expression> = ExpressionIterator::new(&*expression).collect();

        // Root BinaryOperation plus two NumberExpression leaves
        assert_eq!(visited.len(), 3);
        assert_eq!(visited[0].to_string(), expression.to_string());
    }

    #[test]
    fn expression_iterator_traverses_function_call_and_nested_binary() {
        let expression = parse("sqrt ( 1 + 2 )");
        let visited: Vec<&dyn Expression> = ExpressionIterator::new(&*expression).collect();

        // FunctionCall root, nested BinaryOperation, and two NumberExpression leaves
        assert_eq!(visited.len(), 4);
        assert_eq!(visited[0].to_string(), expression.to_string());
    }

    #[test]
    fn expression_iterator_yields_pre_order_sequence() {
        let expression = parse("1 + 2 * 3");
        let visited: Vec<String> = ExpressionIterator::new(&*expression)
            .map(|node| node.to_string())
            .collect();

        // Root first, then the left subtree before the right (stack pushes right first).
        // to_string adds precedence parentheses around leaf operands.
        assert_eq!(visited, vec!["(1) + (2) * (3)", "1", "(2) * (3)", "2", "3"]);
    }

    #[test]
    fn expression_iterator_traverses_nested_function_argument() {
        let expression = parse("sqrt ( x + ( y * 2 ) )");
        let visited: Vec<String> = ExpressionIterator::new(&*expression)
            .map(|node| node.to_string())
            .collect();

        assert_eq!(
            visited,
            vec![
                "sqrt((x) + (y) * (2))",
                "(x) + (y) * (2)",
                "x",
                "(y) * (2)",
                "y",
                "2"
            ]
        );
    }

    #[test]
    fn expression_iterator_visits_all_expression_types() {
        let expression = parse("sin ( x ) + cos ( 5 ) + sqrt ( 4 ) * 2");
        let visited: Vec<&dyn Expression> = ExpressionIterator::new(&*expression).collect();

        let mut binary_ops = 0;
        let mut functions = 0;
        let mut numbers = 0;
        let mut variables = 0;

        for node in visited {
            if node.as_any().downcast_ref::<BinaryOperation>().is_some() {
                binary_ops += 1;
            } else if node.as_any().downcast_ref::<FunctionCall>().is_some() {
                functions += 1;
            } else if node.as_any().downcast_ref::<NumberExpression>().is_some() {
                numbers += 1;
            } else if node.as_any().downcast_ref::<VariableExpression>().is_some() {
                variables += 1;
            }
        }

        assert_eq!(binary_ops, 3);
        assert_eq!(functions, 3);
        assert_eq!(numbers, 3);
        assert_eq!(variables, 1);
    }

    #[test]
    fn expression_iterator_matches_recursive_node_count() {
        // The iterator is non-recursive, so cross-check it against a simple
        // recursive walk of the same tree.
        let expression = parse("x + 2 * ( 3 - sqrt ( 4 ) ) / 8");
        let visited: Vec<&dyn Expression> = ExpressionIterator::new(&*expression).collect();

        assert_eq!(visited.len(), count_subtrees(&*expression));
    }

    #[test]
    fn expression_iterator_handles_single_terminals() {
        let number = parse("42");
        let variable = parse("x");

        assert_eq!(ExpressionIterator::new(&*number).count(), 1);
        assert_eq!(ExpressionIterator::new(&*variable).count(), 1);
        assert_eq!(
            ExpressionIterator::new(&*variable)
                .next()
                .unwrap()
                .to_string(),
            "x"
        );
    }

    fn count_subtrees(expr: &dyn Expression) -> usize {
        if let Some(op) = expr.as_any().downcast_ref::<BinaryOperation>() {
            1 + count_subtrees(&*op.left) + count_subtrees(&*op.right)
        } else if let Some(func) = expr.as_any().downcast_ref::<FunctionCall>() {
            1 + count_subtrees(&*func.argument)
        } else {
            1
        }
    }
}
