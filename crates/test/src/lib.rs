//! nya-test: Test framework for nya

use anyhow::Result;
use std::path::PathBuf;

/// Test configuration
#[derive(Debug, Clone)]
pub struct TestConfig {
    pub filter: Option<String>,
    pub jobs: Option<usize>,
    pub nocapture: bool,
}

/// Test result
#[derive(Debug, Clone)]
pub struct TestResult {
    pub name: String,
    pub passed: bool,
    pub duration: std::time::Duration,
    pub output: String,
}

/// Test runner
pub struct TestRunner {
    config: TestConfig,
}

impl TestRunner {
    pub fn new(config: TestConfig) -> Self {
        Self { config }
    }
    
    pub fn run(&self, test_bin: &PathBuf) -> Result<Vec<TestResult>> {
        // Placeholder - would run actual tests
        Ok(vec![])
    }
    
    pub fn run_all(&self, test_dir: &PathBuf) -> Result<Vec<TestResult>> {
        // Placeholder - would discover and run all tests
        Ok(vec![])
    }
}

/// Assertion macros
#[macro_export]
macro_rules! assert {
    ($cond:expr) => {
        if !$cond {
            panic!("assertion failed: {}", stringify!($cond));
        }
    };
    ($cond:expr, $($arg:tt)+) => {
        if !$cond {
            panic!("assertion failed: {}: {}", stringify!($cond), format!($($arg)+));
        }
    };
}

#[macro_export]
macro_rules! assert_eq {
    ($left:expr, $right:expr) => {
        let left_val = $left;
        let right_val = $right;
        if left_val != right_val {
            panic!("assertion failed: `(left == right)`\n  left: `{:?}`,\n right: `{:?}`", left_val, right_val);
        }
    };
    ($left:expr, $right:expr, $($arg:tt)+) => {
        let left_val = $left;
        let right_val = $right;
        if left_val != right_val {
            panic!("assertion failed: `(left == right)`\n  left: `{:?}`,\n right: `{:?}`: {}", left_val, right_val, format!($($arg)+));
        }
    };
}

#[macro_export]
macro_rules! assert_ne {
    ($left:expr, $right:expr) => {
        let left_val = $left;
        let right_val = $right;
        if left_val == right_val {
            panic!("assertion failed: `(left != right)`\n  left: `{:?}`,\n right: `{:?}`", left_val, right_val);
        }
    };
}

#[macro_export]
macro_rules! assert_approx_eq {
    ($left:expr, $right:expr, $epsilon:expr) => {
        let left_val = $left;
        let right_val = $right;
        let eps = $epsilon;
        if (left_val - right_val).abs() > eps {
            panic!("assertion failed: `(left ≈ right)`\n  left: `{:?}`,\n right: `{:?}`,\n epsilon: `{:?}`", left_val, right_val, eps);
        }
    };
}