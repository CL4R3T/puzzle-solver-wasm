use serde::Serialize;
use solver_core::params::{build_sudoku_constraints, SolveInput};
use solver_core::solver::NumberPuzzleSolver;
use wasm_bindgen::prelude::*;

/// Serialized result returned by `validate`.
#[derive(Serialize)]
struct ValidateOutput {
    valid: bool,
    unique_solution: Option<bool>,
    message: String,
}

/// Solve a puzzle board from JSON.
///
/// Input format:
/// ```json
/// {
///   "board": [[5,3,0,...], ...],
///   "params": { "box_shape": [3,3], "diagonals": true, ... }
/// }
/// ```
///
/// Returns a JSON `SolveResult`.
#[wasm_bindgen]
pub fn solve(json_input: &str) -> String {
    let result = do_solve(json_input);
    serde_json::to_string(&result).unwrap_or_else(|e| {
        format!(
            r#"{{"success":false,"solution":null,"message":"Serialization error: {}","solve_time_ms":0.0}}"#,
            e
        )
    })
}

/// Validate a puzzle board against all constraints.
///
/// Same input format as `solve`.  Returns a JSON object with `valid`,
/// `unique_solution`, and `message` fields.
#[wasm_bindgen]
pub fn validate(json_input: &str) -> String {
    let result = do_validate(json_input);
    serde_json::to_string(&result).unwrap_or_else(|e| {
        format!(
            r#"{{"valid":false,"unique_solution":null,"message":"Serialization error: {}"}}"#,
            e
        )
    })
}

fn do_solve(json_input: &str) -> solver_core::types::SolveResult {
    let input: SolveInput = match serde_json::from_str(json_input) {
        Ok(i) => i,
        Err(e) => {
            return solver_core::types::SolveResult {
                success: false,
                solution: None,
                message: format!("Invalid input JSON: {}", e),
                solve_time_ms: 0.0,
            }
        }
    };

    let n = input.board.len();
    let constraints = build_sudoku_constraints(n, &input.params);

    let solver = match NumberPuzzleSolver::new(input.board, constraints) {
        Ok(s) => s,
        Err(e) => {
            return solver_core::types::SolveResult {
                success: false,
                solution: None,
                message: format!("Solver setup error: {}", e),
                solve_time_ms: 0.0,
            }
        }
    };

    let mut solver = solver.with_max_iterations(5_000_000);
    solver.solve_with_result()
}

fn do_validate(json_input: &str) -> ValidateOutput {
    let input: SolveInput = match serde_json::from_str(json_input) {
        Ok(i) => i,
        Err(e) => {
            return ValidateOutput {
                valid: false,
                unique_solution: None,
                message: format!("Invalid input JSON: {}", e),
            }
        }
    };

    let n = input.board.len();
    let constraints = build_sudoku_constraints(n, &input.params);

    let solver = match NumberPuzzleSolver::new(input.board, constraints) {
        Ok(s) => s,
        Err(e) => {
            return ValidateOutput {
                valid: false,
                unique_solution: None,
                message: format!("Solver setup error: {}", e),
            }
        }
    };

    let vr = solver.validate();
    ValidateOutput {
        valid: vr.valid,
        unique_solution: None,
        message: vr.message,
    }
}
