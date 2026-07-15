use crate::constraint::{propagate_units, Constraint};
use crate::state::SolverState;
use crate::types::ValidationResult;

/// X-Sudoku diagonal constraint: no duplicates on either the main or anti diagonal.
pub struct DiagonalConstraint {
    units: Vec<Vec<(usize, usize)>>,
}

impl DiagonalConstraint {
    pub fn new(n: usize) -> Self {
        let main_diag: Vec<(usize, usize)> = (0..n).map(|i| (i, i)).collect();
        let anti_diag: Vec<(usize, usize)> = (0..n).map(|i| (i, n - 1 - i)).collect();
        Self {
            units: vec![main_diag, anti_diag],
        }
    }
}

impl Constraint for DiagonalConstraint {
    fn propagate(&self, state: &mut SolverState) -> i32 {
        propagate_units(state, &self.units)
    }

    fn validate(&self, state: &SolverState) -> ValidationResult {
        let n = state.n;

        // Main diagonal
        let mut seen = vec![false; n + 1];
        for i in 0..n {
            let v = state.cells[i][i];
            if v != 0 {
                if seen[v as usize] {
                    return ValidationResult::invalid("Main diagonal has duplicate value");
                }
                seen[v as usize] = true;
            }
        }

        // Anti diagonal
        let mut seen = vec![false; n + 1];
        for i in 0..n {
            let v = state.cells[i][n - 1 - i];
            if v != 0 {
                if seen[v as usize] {
                    return ValidationResult::invalid("Anti diagonal has duplicate value");
                }
                seen[v as usize] = true;
            }
        }

        ValidationResult::valid("Diagonal constraints satisfied")
    }
}
