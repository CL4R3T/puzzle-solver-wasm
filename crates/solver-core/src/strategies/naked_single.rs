use crate::state::SolverState;
use crate::strategy::SolvingStrategy;

/// Assign every unfilled cell whose domain contains exactly one value.
pub struct NakedSingleStrategy;

impl SolvingStrategy for NakedSingleStrategy {
    fn apply(&self, state: &mut SolverState) -> i32 {
        let mut assignments = 0;

        for r in 0..state.n {
            for c in 0..state.n {
                if state.cells[r][c] != 0 {
                    continue;
                }

                match state.pos[r][c].count_ones() {
                    0 => return -1,
                    1 => {
                        state.cells[r][c] = state.pos[r][c].trailing_zeros() + 1;
                        assignments += 1;
                    }
                    _ => {}
                }
            }
        }

        assignments
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn assigns_singleton_domains() {
        let mut state = SolverState::new(vec![vec![0, 0], vec![0, 0]]).unwrap();
        state.pos[0][1] = 0b10;

        assert_eq!(NakedSingleStrategy.apply(&mut state), 1);
        assert_eq!(state.cells[0][1], 2);
    }

    #[test]
    fn reports_an_empty_domain() {
        let mut state = SolverState::new(vec![vec![0, 0], vec![0, 0]]).unwrap();
        state.pos[1][0] = 0;

        assert_eq!(NakedSingleStrategy.apply(&mut state), -1);
    }
}
