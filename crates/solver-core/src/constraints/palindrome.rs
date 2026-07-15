use crate::constraint::Constraint;
use crate::state::SolverState;
use crate::types::ValidationResult;

/// Palindrome constraint: values along the path must read the same forwards and
/// backwards.  Paired cells `(i, k-1-i)` must share the same value.
pub struct PalindromeConstraint {
    n: usize,
    cells: Vec<(usize, usize)>,
    k: usize,
}

impl PalindromeConstraint {
    pub fn new(n: usize, cells: Vec<(usize, usize)>) -> Result<Self, String> {
        if cells.len() < 2 {
            return Err("Palindrome path must contain at least two cells".to_string());
        }
        for &(r, c) in &cells {
            if r >= n || c >= n {
                return Err("Palindrome cell is outside the board".to_string());
            }
        }
        let k = cells.len();
        Ok(Self { n, cells, k })
    }
}

impl Constraint for PalindromeConstraint {
    fn propagate(&self, state: &mut SolverState) -> i32 {
        let mut eliminations: i32 = 0;
        let half = self.k / 2;

        for i in 0..half {
            let (r1, c1) = self.cells[i];
            let (r2, c2) = self.cells[self.k - 1 - i];

            let v1 = state.cells[r1][c1];
            let v2 = state.cells[r2][c2];

            if v1 != 0 && v2 != 0 {
                // Both determined — must match
                if v1 != v2 {
                    return -1;
                }
            } else if v1 != 0 {
                // First determined — force second
                let bit = 1u32 << (v1 - 1);
                if state.pos[r2][c2] & bit == 0 {
                    return -1;
                }
                if state.cells[r2][c2] == 0 && state.pos[r2][c2] != bit {
                    let removed = state.pos[r2][c2] & !bit;
                    eliminations += removed.count_ones() as i32;
                    state.pos[r2][c2] = bit;
                    state.cells[r2][c2] = v1;
                }
            } else if v2 != 0 {
                // Second determined — force first
                let bit = 1u32 << (v2 - 1);
                if state.pos[r1][c1] & bit == 0 {
                    return -1;
                }
                if state.cells[r1][c1] == 0 && state.pos[r1][c1] != bit {
                    let removed = state.pos[r1][c1] & !bit;
                    eliminations += removed.count_ones() as i32;
                    state.pos[r1][c1] = bit;
                    state.cells[r1][c1] = v2;
                }
            } else {
                // Both undetermined — restrict to overlapping candidates
                let mask1 = state.pos[r1][c1];
                let mask2 = state.pos[r2][c2];
                let intersection = mask1 & mask2;

                if intersection == 0 {
                    return -1;
                }

                if mask1 != intersection {
                    let removed = mask1 & !intersection;
                    eliminations += removed.count_ones() as i32;
                    state.pos[r1][c1] = intersection;
                }
                if mask2 != intersection {
                    let removed = mask2 & !intersection;
                    eliminations += removed.count_ones() as i32;
                    state.pos[r2][c2] = intersection;
                }

                // If intersection is a single value, assign both
                if intersection.count_ones() == 1 {
                    let val = state.mask_to_values(intersection)[0];
                    if state.cells[r1][c1] == 0 {
                        state.cells[r1][c1] = val;
                    }
                    if state.cells[r2][c2] == 0 {
                        state.cells[r2][c2] = val;
                    }
                }
            }
        }

        // Middle cell (odd-length path): no symmetry constraint needed

        eliminations
    }

    fn validate(&self, state: &SolverState) -> ValidationResult {
        let vals: Vec<u32> = self.cells.iter().map(|&(r, c)| state.cells[r][c]).collect();

        // Skip if not fully filled
        if vals.contains(&0) {
            return ValidationResult::valid("Palindrome not fully filled — skip validation");
        }

        for v in &vals {
            if *v < 1 || *v > self.n as u32 {
                return ValidationResult::invalid(format!("Palindrome has invalid value {}", v));
            }
        }

        let k = self.k;
        for i in 0..k / 2 {
            if vals[i] != vals[k - 1 - i] {
                return ValidationResult::invalid(format!(
                    "Palindrome asymmetry: position {} is {}, position {} is {}",
                    i,
                    vals[i],
                    k - 1 - i,
                    vals[k - 1 - i]
                ));
            }
        }

        ValidationResult::valid("Palindrome constraint satisfied")
    }
}
