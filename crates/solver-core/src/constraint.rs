use crate::state::SolverState;
use crate::types::ValidationResult;

/// A constraint encapsulates domain-specific propagation and validation logic.
///
/// `propagate`: eliminate impossible candidates from `state`, returning:
///   - \>0: number of eliminations made this call
///   - 0: no new eliminations (fixed point reached for this constraint)
///   - -1: contradiction detected
///
/// `validate`: check the fully- (or partially-) filled board against this
///   constraint. May return `valid=true` for partially filled boards when the
///   constraint cannot yet detect a violation.
pub trait Constraint {
    fn propagate(&self, state: &mut SolverState) -> i32;
    fn validate(&self, state: &SolverState) -> ValidationResult;
}

// ── Shared helper: unit-based propagation ──────────────────────────

/// Apply standard unit-propagation over a collection of units.
///
/// Each unit is a `Vec<(r, c)>` of cell coordinates.  For each unit we:
/// 1. Remove determined values from the candidate masks of unfilled cells.
/// 2. Perform hidden-single detection: if a value can only go in one cell of
///    the unit, assign it.
///
/// Returns >0 eliminations, 0 no change, or -1 contradiction.
pub(crate) fn propagate_units(state: &mut SolverState, units: &[Vec<(usize, usize)>]) -> i32 {
    let n = state.n;
    let mut eliminations: i32 = 0;

    for unit in units {
        // Collect determined values in this unit
        let mut determined = vec![false; n + 1]; // 1-indexed
        for &(r, c) in unit {
            let v = state.cells[r][c];
            if v != 0 {
                determined[v as usize] = true;
            }
        }

        // 1. Remove determined values from unfilled cells
        for &(r, c) in unit {
            if state.cells[r][c] == 0 {
                for (val, &is_set) in determined.iter().enumerate().skip(1) {
                    if is_set {
                        let bit = 1u32 << (val - 1);
                        if state.pos[r][c] & bit != 0 {
                            state.pos[r][c] &= !bit;
                            eliminations += 1;
                            if state.pos[r][c] == 0 {
                                return -1;
                            }
                        }
                    }
                }
            }
        }

        // 2. Hidden single detection
        for (val, &is_set) in determined.iter().enumerate().skip(1) {
            if is_set {
                continue;
            }
            let bit = 1u32 << (val - 1);
            let mut possible_cells: Vec<(usize, usize)> = Vec::new();
            for &(r, c) in unit {
                if state.cells[r][c] == 0 && state.pos[r][c] & bit != 0 {
                    possible_cells.push((r, c));
                }
            }
            if possible_cells.is_empty() {
                return -1; // value has no home
            }
            if possible_cells.len() == 1 {
                let (r, c) = possible_cells[0];
                if state.pos[r][c] != bit {
                    let removed = state.pos[r][c] & !bit;
                    eliminations += removed.count_ones() as i32;
                    state.pos[r][c] = bit;
                    state.cells[r][c] = val as u32;
                }
            }
        }
    }

    eliminations
}
