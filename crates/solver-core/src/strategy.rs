use crate::state::SolverState;

/// A reusable deduction step that follows from puzzle constraints but does
/// not define whether a completed board is valid.
///
/// `apply` returns:
/// - `> 0` when the strategy changed the state;
/// - `0` when it made no progress;
/// - `-1` when it found a contradiction.
pub trait SolvingStrategy {
    fn apply(&self, state: &mut SolverState) -> i32;
}
