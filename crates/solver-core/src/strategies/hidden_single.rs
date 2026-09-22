use crate::constraint::Unit;
use crate::state::SolverState;
use crate::strategy::SolvingStrategy;

/// Assign a value when an all-different unit leaves it only one possible cell.
pub struct HiddenSingleStrategy {
    units: Vec<Unit>,
}

impl HiddenSingleStrategy {
    pub fn new(units: Vec<Unit>) -> Self {
        Self { units }
    }
}

impl SolvingStrategy for HiddenSingleStrategy {
    fn apply(&self, state: &mut SolverState) -> i32 {
        let mut changes = 0;

        for unit in &self.units {
            let mut determined = vec![false; state.n + 1];
            for &(r, c) in unit {
                let value = state.cells[r][c];
                if value != 0 {
                    determined[value as usize] = true;
                }
            }

            for (value, &is_determined) in determined.iter().enumerate().skip(1) {
                if is_determined {
                    continue;
                }

                let bit = 1u32 << (value - 1);
                let mut only_cell = None;
                let mut possible_count = 0;

                for &(r, c) in unit {
                    if state.cells[r][c] == 0 && state.pos[r][c] & bit != 0 {
                        possible_count += 1;
                        only_cell = Some((r, c));
                    }
                }

                if possible_count == 0 {
                    return -1;
                }

                if possible_count == 1 {
                    let (r, c) = only_cell.expect("one possible cell must be recorded");
                    if state.cells[r][c] == 0 {
                        state.pos[r][c] = bit;
                        state.cells[r][c] = value as u32;
                        changes += 1;
                    }
                }
            }
        }

        changes
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn assigns_the_only_home_for_a_value() {
        let mut state =
            SolverState::new(vec![vec![0, 0, 0], vec![0, 0, 0], vec![0, 0, 0]]).unwrap();
        state.pos[0][0] = 0b011;
        state.pos[0][1] = 0b011;
        state.pos[0][2] = 0b111;
        let strategy = HiddenSingleStrategy::new(vec![vec![(0, 0), (0, 1), (0, 2)]]);

        assert!(strategy.apply(&mut state) > 0);
        assert_eq!(state.cells[0][2], 3);
        assert_eq!(state.pos[0][2], 0b100);
    }

    #[test]
    fn reports_when_a_value_has_no_home() {
        let mut state = SolverState::new(vec![vec![0, 0], vec![0, 0]]).unwrap();
        state.pos[0][0] = 0b01;
        state.pos[0][1] = 0b01;
        let strategy = HiddenSingleStrategy::new(vec![vec![(0, 0), (0, 1)]]);

        assert_eq!(strategy.apply(&mut state), -1);
    }
}
