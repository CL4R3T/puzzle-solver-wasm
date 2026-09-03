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

// ── Shared helper: all-different propagation ───────────────────────

/// A collection of cells governed by one all-different rule.
pub type Unit = Vec<(usize, usize)>;

/// Apply standard unit-propagation over a collection of units.
///
/// Each unit is a `Vec<(r, c)>` of cell coordinates. For each unit, remove
/// determined values from the candidate masks of unfilled cells.
///
/// Returns >0 eliminations, 0 no change, or -1 contradiction.
///
/// Higher-level deductions such as hidden singles intentionally live in the
/// solving-strategy layer rather than in this constraint helper.
pub(crate) fn propagate_all_different(state: &mut SolverState, units: &[Unit]) -> i32 {
    let mut eliminations: i32 = 0;

    for unit in units {
        // Collect determined values in this unit
        let mut determined = vec![false; state.n + 1]; // 1-indexed
        for &(r, c) in unit {
            let v = state.cells[r][c];
            if v != 0 {
                determined[v as usize] = true;
            }
        }

        // Remove determined values from unfilled cells
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
    }

    eliminations
}
