use crate::constraint::Constraint;
use crate::state::SolverState;
use crate::types::ValidationResult;

/// Thermometer constraint: values must strictly increase along the path.
pub struct ThermoConstraint {
    n: usize,
    cells: Vec<(usize, usize)>,
    k: usize,
    /// Absolute minimum possible value at each position (1-based).
    abs_min: Vec<u32>,
    /// Absolute maximum possible value at each position (1-based).
    abs_max: Vec<u32>,
}

impl ThermoConstraint {
    pub fn new(n: usize, cells: Vec<(usize, usize)>) -> Result<Self, String> {
        if cells.len() < 2 {
            return Err("Thermometer path must contain at least two cells".to_string());
        }
        if cells.len() > n {
            return Err("Thermometer path length cannot exceed board size".to_string());
        }
        for &(r, c) in &cells {
            if r >= n || c >= n {
                return Err("Thermometer cell is outside the board".to_string());
            }
        }
        // Check for duplicates
        let mut seen = std::collections::HashSet::new();
        for &(r, c) in &cells {
            if !seen.insert((r, c)) {
                return Err("Thermometer path cannot contain duplicate cells".to_string());
            }
        }

        let k = cells.len();
        let abs_min: Vec<u32> = (0..k).map(|i| i as u32 + 1).collect();
        let abs_max: Vec<u32> = (0..k).map(|i| n as u32 - (k - 1 - i) as u32).collect();

        Ok(Self {
            n,
            cells,
            k,
            abs_min,
            abs_max,
        })
    }

    /// Build a bitmask covering values in [min_val, max_val] (inclusive, 1-based).
    fn range_mask(&self, min_val: u32, max_val: u32) -> u32 {
        let min_val = min_val.max(1);
        let max_val = max_val.min(self.n as u32);
        if min_val > max_val {
            return 0;
        }
        let width = max_val - min_val + 1;
        ((1u32 << width) - 1) << (min_val - 1)
    }

    /// Forward pass: for each position i, restrict domain to values that can be
    /// reached from some supported value at position i-1.
    fn forward_support(&self, domains: &[u32]) -> Option<Vec<u32>> {
        let mut supported: Vec<u32> = vec![0; self.k];
        supported[0] = domains[0];

        for i in 1..self.k {
            let mut mask: u32 = 0;
            for value in 1..=self.n as u32 {
                if supported[i - 1] & (1u32 << (value - 1)) != 0 {
                    mask |= self.range_mask(value + 1, self.n as u32);
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

    /// Backward pass: for each position i, restrict domain to values that can
    /// reach some supported value at position i+1.
    fn backward_support(&self, domains: &[u32]) -> Option<Vec<u32>> {
        let mut supported: Vec<u32> = vec![0; self.k];
        supported[self.k - 1] = domains[self.k - 1];

        for i in (0..self.k - 1).rev() {
            let mut mask: u32 = 0;
            for value in 1..=self.n as u32 {
                if supported[i + 1] & (1u32 << (value - 1)) != 0 {
                    mask |= self.range_mask(1, value - 1);
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

impl Constraint for ThermoConstraint {
    fn propagate(&self, state: &mut SolverState) -> i32 {
        // Build initial domains from board + absolute position bounds
        let mut domains: Vec<u32> = Vec::with_capacity(self.k);

        for (i, &(r, c)) in self.cells.iter().enumerate() {
            let domain = if state.cells[r][c] != 0 {
                let value = state.cells[r][c];
                if value < 1 || value > self.n as u32 {
                    return -1;
                }
                1u32 << (value - 1)
            } else {
                state.pos[r][c]
            };

            let domain = domain & self.range_mask(self.abs_min[i], self.abs_max[i]);
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
            return ValidationResult::valid("Thermometer not fully filled");
        }

        for v in &vals {
            if *v < 1 || *v > self.n as u32 {
                return ValidationResult::invalid(format!("Thermometer has invalid value {}", v));
            }
        }

        for i in 1..self.k {
            if vals[i] <= vals[i - 1] {
                return ValidationResult::invalid(format!(
                    "Thermometer: position {} value {} not greater than position {} value {}",
                    i,
                    vals[i],
                    i - 1,
                    vals[i - 1]
                ));
            }
        }

        ValidationResult::valid("Thermometer constraint satisfied")
    }
}
