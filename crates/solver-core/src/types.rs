/// Result of validating a board against a single constraint.
#[derive(Debug, Clone)]
pub struct ValidationResult {
    pub valid: bool,
    pub message: String,
}

impl ValidationResult {
    pub fn valid(message: impl Into<String>) -> Self {
        Self {
            valid: true,
            message: message.into(),
        }
    }

    pub fn invalid(message: impl Into<String>) -> Self {
        Self {
            valid: false,
            message: message.into(),
        }
    }
}

use serde::{Deserialize, Serialize};

/// Result returned by the solver (serialized to JSON across the WASM boundary).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SolveResult {
    pub success: bool,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub solution: Option<Vec<Vec<u32>>>,
    pub message: String,
    pub solve_time_ms: f64,
}
