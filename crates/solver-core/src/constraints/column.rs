use crate::constraint::{propagate_units, Constraint};
use crate::state::SolverState;
use crate::types::ValidationResult;

pub struct ColumnConstraint {
    units: Vec<Vec<(usize, usize)>>,
}

impl ColumnConstraint {
    pub fn new(n: usize) -> Self {
        let units: Vec<Vec<(usize, usize)>> =
            (0..n).map(|c| (0..n).map(|r| (r, c)).collect()).collect();
        Self { units }
    }
}

impl Constraint for ColumnConstraint {
    fn propagate(&self, state: &mut SolverState) -> i32 {
        propagate_units(state, &self.units)
    }

    fn validate(&self, state: &SolverState) -> ValidationResult {
        for c in 0..state.n {
            let mut seen = vec![false; state.n + 1];
            for r in 0..state.n {
                let v = state.cells[r][c];
                if v != 0 {
                    if seen[v as usize] {
                        return ValidationResult::invalid(format!(
                            "Column {} has duplicate value {}",
                            c, v
                        ));
                    }
                    seen[v as usize] = true;
                }
            }
        }
        ValidationResult::valid("Column constraints satisfied")
    }
}
