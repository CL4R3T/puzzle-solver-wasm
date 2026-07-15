use crate::constraint::Constraint;
use crate::state::SolverState;
use crate::types::ValidationResult;

/// Killer-cage constraint: cells in the cage must sum to `target`, with no
/// duplicates within the cage.
pub struct KillerCageConstraint {
    n: u32,
    cells: Vec<(usize, usize)>,
    target: u32,
}

impl KillerCageConstraint {
    pub fn new(n: usize, cells: Vec<(usize, usize)>, target: u32) -> Result<Self, String> {
        if cells.is_empty() {
            return Err("Cage must contain at least one cell".to_string());
        }
        for &(r, c) in &cells {
            if r >= n || c >= n {
                return Err("Cage cell is outside the board".to_string());
            }
        }
        Ok(Self {
            n: n as u32,
            cells,
            target,
        })
    }

    /// Extract values (1-based) from a bitmask.
    fn values_of(&self, mask: u32) -> Vec<u32> {
        let mut vals = Vec::new();
        let mut m = mask;
        while m != 0 {
            let lsb = m & m.wrapping_neg();
            vals.push(lsb.trailing_zeros() + 1);
            m ^= lsb;
        }
        vals
    }

    /// Backtracking search: find all combinations of values (one per cell,
    /// no repeats) that sum to `target`.
    fn find_combos(
        &self,
        candidates: &[Vec<u32>],
        idx: usize,
        current_sum: u32,
        current_combo: &mut Vec<u32>,
        result: &mut Vec<Vec<u32>>,
        target: u32,
    ) {
        if idx == candidates.len() {
            if current_sum == target {
                result.push(current_combo.clone());
            }
            return;
        }

        // Pruning: estimate min and max possible sum from remaining cells
        let mut min_possible = current_sum;
        let mut max_possible = current_sum;
        let used: std::collections::HashSet<u32> = current_combo.iter().copied().collect();

        for cand in candidates.iter().skip(idx) {
            let avail: Vec<u32> = cand
                .iter()
                .copied()
                .filter(|v| !used.contains(v))
                .collect();
            if avail.is_empty() {
                return; // a cell has no usable value, prune
            }
            // For pruning, use global min/max of original candidates (loose bound)
            let cmin = cand.iter().min().copied().unwrap_or(1);
            let cmax = cand.iter().max().copied().unwrap_or(self.n);
            min_possible += cmin;
            max_possible += cmax;
        }

        if min_possible > target || max_possible < target {
            return;
        }

        for &val in &candidates[idx] {
            if current_combo.contains(&val) {
                continue;
            }
            if current_sum + val > target {
                continue;
            }
            current_combo.push(val);
            self.find_combos(
                candidates,
                idx + 1,
                current_sum + val,
                current_combo,
                result,
                target,
            );
            current_combo.pop();
        }
    }
}

impl Constraint for KillerCageConstraint {
    fn propagate(&self, state: &mut SolverState) -> i32 {
        let mut eliminations: i32 = 0;

        let mut determined_sum: u32 = 0;
        let mut unfilled: Vec<(usize, usize)> = Vec::new();

        for &(r, c) in &self.cells {
            if state.cells[r][c] != 0 {
                determined_sum += state.cells[r][c];
            } else {
                unfilled.push((r, c));
            }
        }

        // Already exceeds target
        if determined_sum > self.target {
            return -1;
        }

        // All filled
        if unfilled.is_empty() {
            return if determined_sum == self.target { 0 } else { -1 };
        }

        let remaining = self.target - determined_sum;

        // Exactly one empty cell — fill directly
        if unfilled.len() == 1 {
            let (r, c) = unfilled[0];
            let val = remaining;
            if val < 1 || val > self.n {
                return -1;
            }
            let bit = 1u32 << (val - 1);
            if state.pos[r][c] & bit == 0 {
                return -1;
            }
            if state.pos[r][c].count_ones() > 1 {
                let removed = state.pos[r][c] & !bit;
                eliminations += removed.count_ones() as i32;
                state.pos[r][c] = bit;
                state.cells[r][c] = val;
            }
            return eliminations;
        }

        // Extract candidates for each unfilled cell
        let cell_candidates: Vec<Vec<u32>> = unfilled
            .iter()
            .map(|&(r, c)| self.values_of(state.pos[r][c]))
            .collect();

        // Find all valid combinations
        let mut valid_combos: Vec<Vec<u32>> = Vec::new();
        self.find_combos(
            &cell_candidates,
            0,
            0,
            &mut Vec::new(),
            &mut valid_combos,
            remaining,
        );

        if valid_combos.is_empty() {
            return -1;
        }

        // Remove candidates that never appear in any valid combination
        for (i, &(r, c)) in unfilled.iter().enumerate() {
            let valid_vals: std::collections::HashSet<u32> =
                valid_combos.iter().map(|combo| combo[i]).collect();

            let mut new_mask: u32 = 0;
            for &val in &valid_vals {
                new_mask |= 1u32 << (val - 1);
            }
            let old_mask = state.pos[r][c];
            if new_mask != old_mask {
                let removed = old_mask & !new_mask;
                eliminations += removed.count_ones() as i32;
                state.pos[r][c] = new_mask;
                if state.pos[r][c] == 0 {
                    return -1;
                }
            }
        }

        eliminations
    }

    fn validate(&self, state: &SolverState) -> ValidationResult {
        let vals: Vec<u32> = self.cells.iter().map(|&(r, c)| state.cells[r][c]).collect();

        // Skip if not fully filled
        if vals.contains(&0) {
            return ValidationResult::valid("Cage not fully filled — skip validation");
        }

        let sum: u32 = vals.iter().sum();
        if sum != self.target {
            return ValidationResult::invalid(format!(
                "Cage sum {} does not equal target {}",
                sum, self.target
            ));
        }

        // Check for duplicates
        let mut seen = std::collections::HashSet::new();
        for v in &vals {
            if !seen.insert(v) {
                return ValidationResult::invalid("Cage has duplicate values");
            }
        }

        ValidationResult::valid("Killer cage constraint satisfied")
    }
}
