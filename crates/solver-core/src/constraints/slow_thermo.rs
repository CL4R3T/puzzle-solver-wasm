use crate::constraint::Constraint;
use crate::state::SolverState;
use crate::types::ValidationResult;

/// Slow thermometer constraint: values must be **non-decreasing** along the path
/// (each >= previous).  In contrast to the strict thermo (>), repeated values are
/// allowed unless they would violate a separate unit constraint (row/col/box).
///
/// When two (or more) cells of the path lie in the same no-repeat unit, the unit
/// constraint and this constraint together force a strict increase between those
/// positions — but that interaction emerges naturally during propagation and does
/// not need to be coded into this struct.
pub struct SlowThermoConstraint {
    n: usize,
    cells: Vec<(usize, usize)>,
    k: usize,
}

impl SlowThermoConstraint {
    pub fn new(n: usize, cells: Vec<(usize, usize)>) -> Result<Self, String> {
        if cells.len() < 2 {
            return Err("Slow-thermo path must contain at least two cells".to_string());
        }
        for &(r, c) in &cells {
            if r >= n || c >= n {
                return Err("Slow-thermo cell is outside the board".to_string());
            }
        }
        let mut seen = std::collections::HashSet::new();
        for &(r, c) in &cells {
            if !seen.insert((r, c)) {
                return Err("Slow-thermo path cannot contain duplicate cells".to_string());
            }
        }
        let k = cells.len();
        Ok(Self { n, cells, k })
    }

    fn range_mask(&self, min_val: u32, max_val: u32) -> u32 {
        let min_val = min_val.max(1);
        let max_val = max_val.min(self.n as u32);
        if min_val > max_val {
            return 0;
        }
        let width = max_val - min_val + 1;
        ((1u32 << width) - 1) << (min_val - 1)
    }

    /// Forward pass: position i must be >= every possible value at i-1.
    fn forward_support(&self, domains: &[u32]) -> Option<Vec<u32>> {
        let mut supported: Vec<u32> = vec![0; self.k];
        supported[0] = domains[0];

        for i in 1..self.k {
            let mut mask: u32 = 0;
            for value in 1..=self.n as u32 {
                if supported[i - 1] & (1u32 << (value - 1)) != 0 {
                    mask |= self.range_mask(value, self.n as u32);
                }
            }
            mask &= domains[i];
            if mask == 0 {
                return None;
            }
            supported[i] = mask;
        }

        Some(supported)
    }

    /// Backward pass: position i must be <= every possible value at i+1.
    fn backward_support(&self, domains: &[u32]) -> Option<Vec<u32>> {
        let mut supported: Vec<u32> = vec![0; self.k];
        supported[self.k - 1] = domains[self.k - 1];

        for i in (0..self.k - 1).rev() {
            let mut mask: u32 = 0;
            for value in 1..=self.n as u32 {
                if supported[i + 1] & (1u32 << (value - 1)) != 0 {
                    mask |= self.range_mask(1, value);
                }
            }
            mask &= domains[i];
            if mask == 0 {
                return None;
            }
            supported[i] = mask;
        }

        Some(supported)
    }
}

impl Constraint for SlowThermoConstraint {
    fn propagate(&self, state: &mut SolverState) -> i32 {
        // Every position has the full [1..n] absolute range — no tight bounds.
        let mut domains: Vec<u32> = Vec::with_capacity(self.k);

        for &(r, c) in &self.cells {
            let domain = if state.cells[r][c] != 0 {
                let value = state.cells[r][c];
                if value < 1 || value > self.n as u32 {
                    return -1;
                }
                1u32 << (value - 1)
            } else {
                state.pos[r][c]
            };

            if domain == 0 {
                return -1;
            }
            domains.push(domain);
        }

        let forward = match self.forward_support(&domains) {
            Some(f) => f,
            None => return -1,
        };

        let backward = match self.backward_support(&domains) {
            Some(b) => b,
            None => return -1,
        };

        let mut eliminations: i32 = 0;
        for (i, &(r, c)) in self.cells.iter().enumerate() {
            let supported = forward[i] & backward[i];
            if supported == 0 {
                return -1;
            }

            if state.cells[r][c] != 0 {
                if supported & (1u32 << (state.cells[r][c] - 1)) == 0 {
                    return -1;
                }
                continue;
            }

            let new_mask = state.pos[r][c] & supported;
            if new_mask == 0 {
                return -1;
            }
            if new_mask != state.pos[r][c] {
                let removed = state.pos[r][c] & !new_mask;
                eliminations += removed.count_ones() as i32;
                state.pos[r][c] = new_mask;
            }
        }

        eliminations
    }

    fn validate(&self, state: &SolverState) -> ValidationResult {
        let vals: Vec<u32> = self.cells.iter().map(|&(r, c)| state.cells[r][c]).collect();

        if vals.contains(&0) {
            return ValidationResult::valid("Slow thermo not fully filled");
        }

        for v in &vals {
            if *v < 1 || *v > self.n as u32 {
                return ValidationResult::invalid(format!("Slow thermo has invalid value {}", v));
            }
        }

        for i in 1..self.k {
            if vals[i] < vals[i - 1] {
                return ValidationResult::invalid(format!(
                    "Slow thermo: position {} value {} is less than position {} value {}",
                    i,
                    vals[i],
                    i - 1,
                    vals[i - 1]
                ));
            }
        }

        ValidationResult::valid("Slow thermometer constraint satisfied")
    }
}
