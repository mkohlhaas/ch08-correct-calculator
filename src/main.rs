// main.rs - Main entry point for the calculator
// Incorporates all design patterns from Chapters 5-8

#![allow(dead_code)]

// Combined application module
mod calculator;

// Chapter 5-7 modules
mod adapter;
mod bridge;
mod chain;
mod command;
mod config;
mod expression;
mod mediator;
mod parser;
mod strategy;
mod template;
mod token;

// Chapter 8 modules
mod iterator;
mod memento;
mod observer;
mod state;
mod visitor;

use std::collections::HashMap;
use std::io::{self, Write};

use calculator::CorrectCalculator;

use observer::{CalculatorEvent, LoggerObserver, ObservableCalculator, Subject};
use parser::ExpressionParser;
use state::StateCalculator;
use visitor::optimize_expression;

// Entry point: select which pattern to run
fn main() {
    loop {
        println!("\n=== Correct Calculator - Design Patterns Application ===");
        println!("1. Full Calculator (all patterns combined)");
        println!("2. State Pattern Demo");
        println!("3. Observer Pattern Demo");
        println!("4. Visitor Pattern Demo");
        println!("5. Exit");
        print!("Select an option: ");
        io::stdout().flush().unwrap();

        let mut choice = String::new();
        if io::stdin().read_line(&mut choice).is_err() {
            println!("Error reading input, please try again");
            continue;
        }

        match choice.trim() {
            "1" => run_with_full_calculator(),
            "2" => run_with_state(),
            "3" => run_with_observer(),
            "4" => run_with_visitor(),
            "5" | "exit" | "quit" => {
                println!("Goodbye!");
                break;
            }
            _ => println!("Unknown option, please try again"),
        }
    }
}

// Example using the full calculator with all patterns
fn run_with_full_calculator() {
    let mut calculator = CorrectCalculator::new();
    calculator.run();
}

// Example using the State pattern directly
fn run_with_state() {
    println!("Correct Calculator with State Pattern");

    let mut calculator = StateCalculator::new();

    loop {
        print!("{}", calculator.display_prompt());
        io::stdout().flush().unwrap();

        let mut input = String::new();
        if io::stdin().read_line(&mut input).is_err() {
            println!("Error reading input, please try again");
            continue;
        }

        let input = input.trim();
        if input == "/exit" {
            break;
        }

        match calculator.process_input(input) {
            Ok(Some(result)) => println!("= {}", result),
            Ok(None) => {} // Command executed with no result to display
            Err(error) => println!("Error: {}", error),
        }
    }

    println!("Goodbye!");
}

// Example using the Observer pattern directly
fn run_with_observer() {
    println!("Correct Calculator with Observer Pattern");

    let mut calculator = StateCalculator::new();
    let mut observable = ObservableCalculator::new();

    // Add observers
    observable.attach(Box::new(LoggerObserver));

    loop {
        print!("{}", calculator.display_prompt());
        io::stdout().flush().unwrap();

        let mut input = String::new();
        if io::stdin().read_line(&mut input).is_err() {
            println!("Error reading input, please try again");
            continue;
        }

        let input = input.trim();
        if input == "/exit" {
            break;
        }

        match calculator.process_input(input) {
            Ok(Some(result)) => {
                println!("= {}", result);
                observable.notify(&CalculatorEvent::ResultCalculated(
                    result,
                    input.to_string(),
                ));
            }
            Ok(None) => {} // Command executed with no result to display
            Err(error) => {
                println!("Error: {}", error);
                observable.notify(&CalculatorEvent::Error(error));
            }
        }
    }

    println!("Goodbye!");
}

// Example using the Visitor pattern directly
fn run_with_visitor() {
    println!("Correct Calculator with Visitor Pattern");

    let parser = ExpressionParser::new();
    let variables = HashMap::new();

    loop {
        print!("> ");
        io::stdout().flush().unwrap();

        let mut input = String::new();
        if io::stdin().read_line(&mut input).is_err() {
            println!("Error reading input, please try again");
            continue;
        }

        let input = input.trim();
        if input == "exit" {
            break;
        }

        if input.starts_with("optimize ") {
            let expr_str = input.trim_start_matches("optimize ").trim();
            match parser.parse(expr_str) {
                Ok(expr) => match optimize_expression(&*expr, &variables) {
                    Ok(optimized) => {
                        println!("Original: {}", expr.to_string());
                        println!("Optimized: {}", optimized.to_string());

                        match optimized.evaluate(&variables) {
                            Ok(result) => println!("Result: {}", result),
                            Err(e) => println!("Evaluation error: {}", e),
                        }
                    }
                    Err(e) => println!("Optimization error: {}", e),
                },
                Err(e) => println!("Parsing error: {}", e),
            }
            continue;
        }

        match parser.parse(input) {
            Ok(expr) => match expr.evaluate(&variables) {
                Ok(result) => println!("= {}", result),
                Err(e) => println!("Error: {}", e),
            },
            Err(e) => println!("Error: {}", e),
        }
    }

    println!("Goodbye!");
}
