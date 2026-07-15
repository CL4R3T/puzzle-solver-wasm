use crate::constraint::Constraint;
use crate::constraints::ConstraintKind;
use crate::state::SolverState;
use crate::types::{SolveResult, ValidationResult};

/// Generic constraint-based number-puzzle solver.
///
/// Uses fixed-point constraint propagation followed by backtracking search
/// with the minimum-remaining-values (MRV) heuristic.  Timeout is enforced
/// cooperatively via an iteration budget.
pub struct NumberPuzzleSolver {
    state: SolverState,
    constraints: Vec<ConstraintKind>,
    /// Number of propagation cycles performed so far.
    iteration_count: u64,
    /// Maximum propagation cycles before timing out.
    max_iterations: u64,
    /// Instant when solving started (set by `solve()`).
    solve_start: Option<std::time::Instant>,
}

impl NumberPuzzleSolver {
    /// Create a new solver from a board and a set of constraints.
    pub fn new(board: Vec<Vec<u32>>, constraints: Vec<ConstraintKind>) -> Result<Self, String> {
        let state = SolverState::new(board)?;
        Ok(Self {
            state,
            constraints,
            iteration_count: 0,
            max_iterations: 10_000_000,
            solve_start: None,
        })
    }

    /// Override the default iteration budget.
    pub fn with_max_iterations(mut self, max: u64) -> Self {
        self.max_iterations = max;
        self
    }

    // ── Timeout ─────────────────────────────────────────────

    fn check_timeout(&mut self) -> bool {
        self.iteration_count += 1;
        self.iteration_count <= self.max_iterations
    }

    // ── Constraint propagation ──────────────────────────────

    /// Fixed-point iteration: naked-single detection → constraint propagation,
    /// repeating until no further progress or a contradiction is found.
    ///
    /// Returns `true` if the board is consistent (may still have unknowns),
    /// `false` if a contradiction was detected.
    fn propagate(&mut self) -> bool {
        loop {
            if !self.check_timeout() {
                return false;
            }
            let mut made_progress = false;

            // Naked singles: cells with exactly one candidate → assign it.
            for r in 0..self.state.n {
                for c in 0..self.state.n {
                    if self.state.cells[r][c] == 0 {
                        let bits = self.state.pos[r][c].count_ones();
                        if bits == 0 {
                            return false; // contradiction: no possible value
                        }
                        if bits == 1 {
                            let val = self
                                .state
                                .mask_to_values(self.state.pos[r][c])
                                .into_iter()
                                .next()
                                .unwrap();
                            self.state.cells[r][c] = val;
                            made_progress = true;
                        }
                    }
                }
            }

            // Run each constraint's propagation
            for constraint in &self.constraints {
                let result = constraint.propagate(&mut self.state);
                if result == -1 {
                    return false;
                }
                if result > 0 {
                    made_progress = true;
                }
            }

            if !made_progress {
                break;
            }
        }

        true
    }

    // ── Backtracking search ─────────────────────────────────

    /// Find the unfilled cell with the fewest candidates (MRV heuristic).
    fn find_min_cell(&self) -> Option<(usize, usize)> {
        let mut best: Option<(usize, usize)> = None;
        let mut min_size = self.state.n as u32 + 1;
        for r in 0..self.state.n {
            for c in 0..self.state.n {
                let s = self.state.pos[r][c].count_ones();
                if s > 1 && s < min_size {
                    min_size = s;
                    best = Some((r, c));
                }
            }
        }
        best
    }

    /// Recursive constraint-propagation + backtracking search.
    fn solve_with_cp(&mut self) -> bool {
        if !self.check_timeout() {
            return false;
        }
        if !self.propagate() {
            return false;
        }

        let cell = self.find_min_cell();
        if cell.is_none() {
            // All cells determined — solved!
            return true;
        }

        let (row, col) = cell.unwrap();
        let candidates = self.state.mask_to_values(self.state.pos[row][col]);

        for val in candidates {
            let (saved_cells, saved_pos) = self.state.clone_state();
            self.state.cells[row][col] = val;
            self.state.pos[row][col] = 1u32 << (val - 1);
            if self.solve_with_cp() {
                return true;
            }
            self.state.restore_state(saved_cells, saved_pos);
        }

        false
    }

    // ── Public API ──────────────────────────────────────────

    /// Attempt to solve.  Returns `Some(solution)` on success, `None` if
    /// unsolvable or timed out.
    pub fn solve(&mut self) -> Option<Vec<Vec<u32>>> {
        self.solve_start = Some(std::time::Instant::now());
        if self.solve_with_cp() {
            Some(self.state.cells.clone())
        } else {
            None
        }
    }

    /// Solve and return a structured `SolveResult` (with timing).
    pub fn solve_with_result(&mut self) -> SolveResult {
        let start = std::time::Instant::now();
        self.solve_start = Some(start);
        if self.solve_with_cp() {
            SolveResult {
                success: true,
                solution: Some(self.state.cells.clone()),
                message: "Solve succeeded".to_string(),
                solve_time_ms: start.elapsed().as_secs_f64() * 1000.0,
            }
        } else {
            SolveResult {
                success: false,
                solution: None,
                message: "No solution found or timed out".to_string(),
                solve_time_ms: start.elapsed().as_secs_f64() * 1000.0,
            }
        }
    }

    /// Validate the board against all constraints.
    pub fn validate(&self) -> ValidationResult {
        for constraint in &self.constraints {
            let result = constraint.validate(&self.state);
            if !result.valid {
                return result;
            }
        }
        ValidationResult::valid("Board is valid")
    }
}

// ── Tests ───────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;
    use crate::constraints::box_constraint::BoxConstraint;
    use crate::constraints::column::ColumnConstraint;
    use crate::constraints::diagonal::DiagonalConstraint;
    use crate::constraints::killer_cage::KillerCageConstraint;
    use crate::constraints::row::RowConstraint;
    use crate::constraints::thermo::ThermoConstraint;

    fn standard_9x9_board() -> Vec<Vec<u32>> {
        vec![
            vec![5, 3, 0, 0, 7, 0, 0, 0, 0],
            vec![6, 0, 0, 1, 9, 5, 0, 0, 0],
            vec![0, 9, 8, 0, 0, 0, 0, 6, 0],
            vec![8, 0, 0, 0, 6, 0, 0, 0, 3],
            vec![4, 0, 0, 8, 0, 3, 0, 0, 1],
            vec![7, 0, 0, 0, 2, 0, 0, 0, 6],
            vec![0, 6, 0, 0, 0, 0, 2, 8, 0],
            vec![0, 0, 0, 4, 1, 9, 0, 0, 5],
            vec![0, 0, 0, 0, 8, 0, 0, 7, 9],
        ]
    }

    fn sudoku_constraints(n: usize) -> Vec<ConstraintKind> {
        vec![
            ConstraintKind::Row(RowConstraint::new(n)),
            ConstraintKind::Column(ColumnConstraint::new(n)),
            ConstraintKind::Box(BoxConstraint::new(n, (3, 3)).unwrap()),
        ]
    }

    // ── Basic solver ─────────────────────────────────────────

    #[test]
    fn test_solve_standard_9x9() {
        let board = standard_9x9_board();
        let constraints = sudoku_constraints(9);
        let mut solver = NumberPuzzleSolver::new(board, constraints).unwrap();
        let solution = solver.solve();
        assert!(solution.is_some(), "Standard 9x9 should have a solution");
        let sol = solution.unwrap();

        for (r, row) in sol.iter().enumerate() {
            let mut seen = std::collections::HashSet::new();
            for &val in row {
                assert!(seen.insert(val), "Row {} duplicate {}", r, val);
            }
        }
        for c in 0..9 {
            let mut seen = std::collections::HashSet::new();
            for row in sol.iter() {
                assert!(seen.insert(row[c]), "Column {} duplicate {}", c, row[c]);
            }
        }
        assert_eq!(sol[0][2], 4);
        assert_eq!(sol[8][8], 9);
    }

    #[test]
    fn test_solve_custom_block_6x6() {
        let board = vec![
            vec![1, 0, 3, 0, 5, 6],
            vec![0, 0, 6, 1, 0, 3],
            vec![2, 3, 0, 5, 0, 0],
            vec![5, 0, 1, 0, 3, 0],
            vec![3, 4, 0, 0, 1, 2],
            vec![0, 0, 2, 0, 0, 5],
        ];
        let constraints = vec![
            ConstraintKind::Row(RowConstraint::new(6)),
            ConstraintKind::Column(ColumnConstraint::new(6)),
            ConstraintKind::Box(BoxConstraint::new(6, (2, 3)).unwrap()),
        ];
        let mut solver = NumberPuzzleSolver::new(board, constraints).unwrap();
        assert!(
            solver.solve().is_some(),
            "6x6 custom block should have a solution"
        );
    }

    #[test]
    fn test_latin_square() {
        let board = vec![vec![0, 0, 0], vec![0, 0, 0], vec![0, 0, 0]];
        let constraints = vec![
            ConstraintKind::Row(RowConstraint::new(3)),
            ConstraintKind::Column(ColumnConstraint::new(3)),
        ];
        let mut solver = NumberPuzzleSolver::new(board, constraints).unwrap();
        let sol = solver.solve().unwrap();
        for row in sol.iter() {
            assert_eq!(
                row.iter().collect::<std::collections::HashSet<_>>().len(),
                3
            );
        }
    }

    #[test]
    fn test_validate_rejects_bad() {
        let bad = vec![vec![1, 1], vec![0, 0]];
        let constraints = vec![
            ConstraintKind::Row(RowConstraint::new(2)),
            ConstraintKind::Column(ColumnConstraint::new(2)),
            ConstraintKind::Box(BoxConstraint::new(2, (1, 2)).unwrap()),
        ];
        let solver = NumberPuzzleSolver::new(bad, constraints).unwrap();
        assert!(!solver.validate().valid);
    }

    #[test]
    fn test_unsolvable_returns_none() {
        let board = vec![vec![1, 0], vec![1, 0]];
        let constraints = vec![
            ConstraintKind::Row(RowConstraint::new(2)),
            ConstraintKind::Column(ColumnConstraint::new(2)),
            ConstraintKind::Box(BoxConstraint::new(2, (1, 2)).unwrap()),
        ];
        let mut solver = NumberPuzzleSolver::new(board, constraints).unwrap();
        assert!(solver.solve().is_none());
    }

    #[test]
    fn test_solve_4x4() {
        let board = vec![
            vec![1, 0, 0, 4],
            vec![0, 0, 0, 0],
            vec![0, 0, 0, 0],
            vec![3, 0, 0, 2],
        ];
        let constraints = vec![
            ConstraintKind::Row(RowConstraint::new(4)),
            ConstraintKind::Column(ColumnConstraint::new(4)),
            ConstraintKind::Box(BoxConstraint::new(4, (2, 2)).unwrap()),
        ];
        let mut solver = NumberPuzzleSolver::new(board, constraints).unwrap();
        assert!(solver.solve().is_some());
    }

    // ── Constraint validation ─────────────────────────────────

    #[test]
    fn test_row_constraint_validation() {
        let rc = RowConstraint::new(3);
        let state = SolverState::new(vec![vec![1, 2, 3], vec![0, 0, 0], vec![0, 0, 0]]).unwrap();
        assert!(rc.validate(&state).valid);
        let bad = SolverState::new(vec![vec![1, 1, 0], vec![0, 0, 0], vec![0, 0, 0]]).unwrap();
        assert!(!rc.validate(&bad).valid);
    }

    #[test]
    fn test_column_constraint_validation() {
        let cc = ColumnConstraint::new(3);
        let state = SolverState::new(vec![vec![1, 0, 0], vec![2, 0, 0], vec![3, 0, 0]]).unwrap();
        assert!(cc.validate(&state).valid);
        let bad = SolverState::new(vec![vec![1, 0, 0], vec![1, 0, 0], vec![0, 0, 0]]).unwrap();
        assert!(!cc.validate(&bad).valid);
    }

    #[test]
    fn test_box_constraint_validation() {
        let bc = BoxConstraint::new(4, (2, 2)).unwrap();
        let state = SolverState::new(vec![
            vec![1, 2, 0, 0],
            vec![3, 4, 0, 0],
            vec![0, 0, 0, 0],
            vec![0, 0, 0, 0],
        ])
        .unwrap();
        assert!(bc.validate(&state).valid);
        let bad = SolverState::new(vec![
            vec![1, 2, 0, 0],
            vec![2, 0, 0, 0],
            vec![0, 0, 0, 0],
            vec![0, 0, 0, 0],
        ])
        .unwrap();
        assert!(!bc.validate(&bad).valid);
    }

    #[test]
    fn test_diagonal_constraint_validation() {
        let dc = DiagonalConstraint::new(3);
        let state = SolverState::new(vec![vec![1, 0, 0], vec![0, 2, 0], vec![0, 0, 3]]).unwrap();
        assert!(dc.validate(&state).valid);
        let bad = SolverState::new(vec![vec![1, 0, 0], vec![0, 1, 0], vec![0, 0, 2]]).unwrap();
        assert!(!dc.validate(&bad).valid);
    }

    // ── Diagonal Sudoku ──────────────────────────────────────

    #[test]
    fn test_diagonal_sudoku() {
        let mut board = vec![vec![0u32; 9]; 9];
        board[4][4] = 1;
        let constraints = vec![
            ConstraintKind::Row(RowConstraint::new(9)),
            ConstraintKind::Column(ColumnConstraint::new(9)),
            ConstraintKind::Box(BoxConstraint::new(9, (3, 3)).unwrap()),
            ConstraintKind::Diagonal(DiagonalConstraint::new(9)),
        ];
        let mut solver = NumberPuzzleSolver::new(board, constraints).unwrap();
        let sol = solver.solve().unwrap();
        let main_diag: Vec<u32> = (0..9).map(|i| sol[i][i]).collect();
        let anti_diag: Vec<u32> = (0..9).map(|i| sol[i][8 - i]).collect();
        assert_eq!(
            main_diag
                .iter()
                .collect::<std::collections::HashSet<_>>()
                .len(),
            9
        );
        assert_eq!(
            anti_diag
                .iter()
                .collect::<std::collections::HashSet<_>>()
                .len(),
            9
        );
    }

    // ── Killer Cage ───────────────────────────────────────────

    #[test]
    fn test_killer_cage_validation() {
        let cage = KillerCageConstraint::new(4, vec![(0, 0), (0, 1)], 5).unwrap();
        assert!(
            cage.validate(
                &SolverState::new(vec![
                    vec![1, 4, 0, 0],
                    vec![0, 0, 0, 0],
                    vec![0, 0, 0, 0],
                    vec![0, 0, 0, 0]
                ])
                .unwrap()
            )
            .valid
        );
        // Wrong sum
        assert!(
            !cage
                .validate(
                    &SolverState::new(vec![
                        vec![2, 4, 0, 0],
                        vec![0, 0, 0, 0],
                        vec![0, 0, 0, 0],
                        vec![0, 0, 0, 0]
                    ])
                    .unwrap()
                )
                .valid
        );
        // Duplicate
        assert!(
            !cage
                .validate(
                    &SolverState::new(vec![
                        vec![2, 2, 0, 0],
                        vec![0, 0, 0, 0],
                        vec![0, 0, 0, 0],
                        vec![0, 0, 0, 0]
                    ])
                    .unwrap()
                )
                .valid
        );
        // Not fully filled — skip
        assert!(
            cage.validate(
                &SolverState::new(vec![
                    vec![1, 0, 0, 0],
                    vec![0, 0, 0, 0],
                    vec![0, 0, 0, 0],
                    vec![0, 0, 0, 0]
                ])
                .unwrap()
            )
            .valid
        );
    }

    #[test]
    fn test_killer_cage_propagate_single_remaining() {
        let cage = KillerCageConstraint::new(4, vec![(0, 0), (0, 1)], 5).unwrap();
        let mut board = vec![vec![0u32; 4]; 4];
        board[0][0] = 1;
        let mut state = SolverState::new(board).unwrap();
        state.pos[0][0] = 1; // val=1
        let result = cage.propagate(&mut state);
        assert!(result > 0);
        assert_eq!(state.cells[0][1], 4);
        assert_eq!(state.pos[0][1], 1 << 3); // val=4
    }

    #[test]
    fn test_killer_cage_propagate_elimination() {
        // 3-cell cage with target 6: only {1,2,3} works
        let cage = KillerCageConstraint::new(4, vec![(0, 0), (0, 1), (0, 2)], 6).unwrap();
        let mut state = SolverState::new(vec![vec![0u32; 4]; 4]).unwrap();
        let result = cage.propagate(&mut state);
        assert!(result > 0);
        let bit4 = 1u32 << 3;
        for c in 0..3 {
            assert_eq!(
                state.pos[0][c] & bit4,
                0,
                "cell (0,{}) should not contain 4",
                c
            );
            assert_ne!(state.pos[0][c], 0);
        }
    }

    #[test]
    fn test_killer_cage_contradiction() {
        let cage = KillerCageConstraint::new(4, vec![(0, 0), (0, 1)], 3).unwrap();
        let mut board = vec![vec![0u32; 4]; 4];
        board[0][0] = 4;
        let mut state = SolverState::new(board).unwrap();
        state.pos[0][0] = 1 << 3;
        let result = cage.propagate(&mut state);
        assert_eq!(result, -1);
    }

    #[test]
    fn test_killer_sudoku_solve() {
        let board = vec![
            vec![1, 0, 0, 4],
            vec![0, 4, 0, 0],
            vec![2, 0, 0, 0],
            vec![0, 0, 0, 1],
        ];
        let constraints = vec![
            ConstraintKind::Row(RowConstraint::new(4)),
            ConstraintKind::Column(ColumnConstraint::new(4)),
            ConstraintKind::Box(BoxConstraint::new(4, (2, 2)).unwrap()),
            ConstraintKind::KillerCage(
                KillerCageConstraint::new(4, vec![(0, 1), (0, 2)], 5).unwrap(),
            ),
        ];
        let mut solver = NumberPuzzleSolver::new(board, constraints).unwrap();
        let sol = solver.solve().unwrap();
        assert_eq!(sol[0][1] + sol[0][2], 5);
    }

    #[test]
    fn test_killer_sudoku_full_cage() {
        let board = vec![vec![0u32; 4]; 4];
        let cages = vec![
            (vec![(0, 0), (0, 1)], 3), // 1+2
            (vec![(0, 2), (1, 2)], 4), // 3+1
            (vec![(0, 3), (1, 3)], 6), // 4+2
            (vec![(1, 0), (1, 1)], 7), // 3+4
            (vec![(2, 0), (3, 0)], 6), // 2+4
            (vec![(2, 1), (2, 2)], 5), // 1+4
            (vec![(2, 3), (3, 3)], 4), // 3+1
            (vec![(3, 1), (3, 2)], 5), // 3+2
        ];
        let mut constraints: Vec<ConstraintKind> = vec![
            ConstraintKind::Row(RowConstraint::new(4)),
            ConstraintKind::Column(ColumnConstraint::new(4)),
            ConstraintKind::Box(BoxConstraint::new(4, (2, 2)).unwrap()),
        ];
        for (cells, sum) in &cages {
            constraints.push(ConstraintKind::KillerCage(
                KillerCageConstraint::new(4, cells.clone(), *sum).unwrap(),
            ));
        }
        let mut solver = NumberPuzzleSolver::new(board, constraints).unwrap();
        let sol = solver.solve().unwrap();
        for (cells, sum) in &cages {
            let cage_sum: u32 = cells.iter().map(|&(r, c)| sol[r][c]).sum();
            assert_eq!(cage_sum, *sum);
        }
    }

    // ── Thermo ────────────────────────────────────────────────

    #[test]
    fn test_thermo_validation() {
        let thermo = ThermoConstraint::new(4, vec![(0, 0), (0, 1), (0, 2)]).unwrap();
        // Strictly increasing
        assert!(
            thermo
                .validate(
                    &SolverState::new(vec![
                        vec![1, 2, 3, 0],
                        vec![0, 0, 0, 0],
                        vec![0, 0, 0, 0],
                        vec![0, 0, 0, 0]
                    ])
                    .unwrap()
                )
                .valid
        );
        // Not strictly increasing
        assert!(
            !thermo
                .validate(
                    &SolverState::new(vec![
                        vec![1, 2, 2, 0],
                        vec![0, 0, 0, 0],
                        vec![0, 0, 0, 0],
                        vec![0, 0, 0, 0]
                    ])
                    .unwrap()
                )
                .valid
        );
        // Decreasing
        assert!(
            !thermo
                .validate(
                    &SolverState::new(vec![
                        vec![3, 2, 1, 0],
                        vec![0, 0, 0, 0],
                        vec![0, 0, 0, 0],
                        vec![0, 0, 0, 0]
                    ])
                    .unwrap()
                )
                .valid
        );
        // Not filled
        assert!(
            thermo
                .validate(
                    &SolverState::new(vec![
                        vec![1, 2, 0, 0],
                        vec![0, 0, 0, 0],
                        vec![0, 0, 0, 0],
                        vec![0, 0, 0, 0]
                    ])
                    .unwrap()
                )
                .valid
        );
    }

    #[test]
    fn test_thermo_propagate_lower_bound() {
        let thermo = ThermoConstraint::new(6, vec![(0, 0), (0, 1), (0, 2)]).unwrap();
        let mut board = vec![vec![0u32; 6]; 6];
        board[0][0] = 3;
        let mut state = SolverState::new(board).unwrap();
        state.pos[0][0] = 1 << 2; // val=3
        let result = thermo.propagate(&mut state);
        assert!(result > 0);
        // Position 1 must be >= 4
        assert_eq!(state.pos[0][1] & 0b111, 0);
        // Position 2 must be >= 5
        assert_eq!(state.pos[0][2] & 0b1111, 0);
    }

    #[test]
    fn test_thermo_propagate_upper_bound() {
        let thermo = ThermoConstraint::new(6, vec![(0, 0), (0, 1), (0, 2), (0, 3)]).unwrap();
        let mut board = vec![vec![0u32; 6]; 6];
        board[0][3] = 5;
        let mut state = SolverState::new(board).unwrap();
        state.pos[0][3] = 1 << 4; // val=5
        let result = thermo.propagate(&mut state);
        assert!(result >= 0);
        // Position 2 must be <= 4 (5-1), so 5 and 6 are eliminated
        assert_eq!(state.pos[0][2] & (1 << 4), 0); // value 5
        assert_eq!(state.pos[0][2] & (1 << 5), 0); // value 6
    }

    #[test]
    fn test_thermo_propagate_contradiction() {
        let thermo = ThermoConstraint::new(4, vec![(0, 0), (0, 1)]).unwrap();
        let mut board = vec![vec![0u32; 4]; 4];
        board[0][0] = 3;
        board[0][1] = 2;
        let mut state = SolverState::new(board).unwrap();
        state.pos[0][0] = 1 << 2;
        state.pos[0][1] = 1 << 1;
        assert_eq!(thermo.propagate(&mut state), -1);
    }

    #[test]
    fn test_thermo_propagate_candidate_support() {
        let thermo = ThermoConstraint::new(6, vec![(0, 0), (0, 1), (0, 2)]).unwrap();
        let mut state = SolverState::new(vec![vec![0u32; 6]; 6]).unwrap();
        state.pos[0][0] = (1 << 0) | (1 << 4); // 1, 5
        state.pos[0][1] = (1 << 1) | (1 << 5); // 2, 6
        state.pos[0][2] = (1 << 2) | (1 << 3); // 3, 4
        let result = thermo.propagate(&mut state);
        assert!(result > 0);
        assert_eq!(state.pos[0][0], 1 << 0); // only 1
        assert_eq!(state.pos[0][1], 1 << 1); // only 2
        assert!(state.pos[0][2] & ((1 << 2) | (1 << 3)) != 0);
    }

    #[test]
    fn test_thermo_propagate_candidate_contradiction() {
        let thermo = ThermoConstraint::new(6, vec![(0, 0), (0, 1), (0, 2)]).unwrap();
        let mut state = SolverState::new(vec![vec![0u32; 6]; 6]).unwrap();
        state.pos[0][0] = 1 << 3; // 4
        state.pos[0][1] = 1 << 1; // 2
        state.pos[0][2] = 1 << 2; // 3
        assert_eq!(thermo.propagate(&mut state), -1);
    }

    #[test]
    fn test_thermo_sudoku_solve() {
        let board = vec![
            vec![0, 0, 0, 4],
            vec![0, 0, 0, 0],
            vec![0, 0, 0, 0],
            vec![1, 0, 0, 0],
        ];
        let constraints = vec![
            ConstraintKind::Row(RowConstraint::new(4)),
            ConstraintKind::Column(ColumnConstraint::new(4)),
            ConstraintKind::Box(BoxConstraint::new(4, (2, 2)).unwrap()),
            ConstraintKind::Thermo(ThermoConstraint::new(4, vec![(0, 1), (0, 2)]).unwrap()),
        ];
        let mut solver = NumberPuzzleSolver::new(board, constraints).unwrap();
        let sol = solver.solve().unwrap();
        assert!(sol[0][1] < sol[0][2]);
    }
}
