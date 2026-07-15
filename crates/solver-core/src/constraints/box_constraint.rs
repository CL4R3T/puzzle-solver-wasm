use crate::constraint::{propagate_units, Constraint};
use crate::state::SolverState;
use crate::types::ValidationResult;

pub struct BoxConstraint {
    n: usize,
    box_rows: usize,
    box_cols: usize,
    units: Vec<Vec<(usize, usize)>>,
}

impl BoxConstraint {
    /// Create a box constraint.  `box_shape` is `(rows, cols)` and must satisfy
    /// `rows * cols == n`.
    pub fn new(n: usize, box_shape: (usize, usize)) -> Result<Self, String> {
        let (br, bc) = box_shape;
        if br * bc != n {
            return Err(format!(
                "box_shape {:?} area {} must equal board side length {}",
                box_shape,
                br * bc,
                n
            ));
        }

        let mut units: Vec<Vec<(usize, usize)>> = Vec::new();
        let mut box_r = 0;
        while box_r < n {
            let mut box_c = 0;
            while box_c < n {
                let mut cells = Vec::with_capacity(n);
                for r in box_r..box_r + br {
                    for c in box_c..box_c + bc {
                        cells.push((r, c));
                    }
                }
                units.push(cells);
                box_c += bc;
            }
            box_r += br;
        }

        Ok(Self {
            n,
            box_rows: br,
            box_cols: bc,
            units,
        })
    }
}

impl Constraint for BoxConstraint {
    fn propagate(&self, state: &mut SolverState) -> i32 {
        propagate_units(state, &self.units)
    }

    fn validate(&self, state: &SolverState) -> ValidationResult {
        let mut box_r = 0;
        while box_r < self.n {
            let mut box_c = 0;
            while box_c < self.n {
                let mut seen = vec![false; self.n + 1];
                for r in box_r..box_r + self.box_rows {
                    for c in box_c..box_c + self.box_cols {
                        let v = state.cells[r][c];
                        if v != 0 {
                            if seen[v as usize] {
                                return ValidationResult::invalid(
                                    "Box has duplicate value".to_string(),
                                );
                            }
                            seen[v as usize] = true;
                        }
                    }
                }
                box_c += self.box_cols;
            }
            box_r += self.box_rows;
        }
        ValidationResult::valid("Box constraints satisfied")
    }
}
