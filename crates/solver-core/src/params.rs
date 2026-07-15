use serde::Deserialize;

use crate::constraints::box_constraint::BoxConstraint;
use crate::constraints::column::ColumnConstraint;
use crate::constraints::diagonal::DiagonalConstraint;
use crate::constraints::killer_cage::KillerCageConstraint;
use crate::constraints::palindrome::PalindromeConstraint;
use crate::constraints::row::RowConstraint;
use crate::constraints::slow_thermo::SlowThermoConstraint;
use crate::constraints::thermo::ThermoConstraint;
use crate::constraints::ConstraintKind;

/// Parameters sent by the frontend for a Sudoku-like puzzle.
///
/// JSON shape matches `SolveParams` in the TypeScript codebase.
#[derive(Debug, Clone, Deserialize)]
#[serde(default)]
pub struct SudokuParams {
    /// Box shape as `[rows, cols]`.  Default `[3, 3]` for 9x9 Sudoku.
    #[serde(default = "default_box_shape")]
    pub box_shape: [usize; 2],
    /// Enable diagonal (X-Sudoku) constraints.
    #[serde(default)]
    pub diagonals: bool,
    /// Killer cage definitions.
    #[serde(default)]
    pub cages: Vec<CageDef>,
    /// Thermometer paths (strictly increasing).
    #[serde(default)]
    pub thermos: Vec<PathDef>,
    /// Slow thermometer paths (non-decreasing, >=).
    #[serde(default)]
    pub slow_thermos: Vec<PathDef>,
    /// Palindrome paths.
    #[serde(default)]
    pub palindromes: Vec<PathDef>,
}

fn default_box_shape() -> [usize; 2] {
    [3, 3]
}

impl Default for SudokuParams {
    fn default() -> Self {
        Self {
            box_shape: [3, 3],
            diagonals: false,
            cages: Vec::new(),
            thermos: Vec::new(),
            slow_thermos: Vec::new(),
            palindromes: Vec::new(),
        }
    }
}

/// A single killer cage: list of cells and the target sum.
#[derive(Debug, Clone, Deserialize)]
pub struct CageDef {
    pub cells: Vec<[usize; 2]>,
    pub sum: u32,
}

/// A path through cells (shared by thermometers and palindromes).
pub type PathDef = Vec<[usize; 2]>;

/// Input wrapper parsed from the JSON the frontend sends.
#[derive(Debug, Deserialize)]
pub struct SolveInput {
    pub board: Vec<Vec<u32>>,
    pub params: SudokuParams,
}

// ── Constraint builder ───────────────────────────────────────

/// Build the constraint list for a Sudoku-type puzzle from the given params.
///
/// Always includes Row + Column + Box.  Extra constraints (diagonals, cages,
/// thermos, palindromes) are added when present in `params`.
pub fn build_sudoku_constraints(n: usize, params: &SudokuParams) -> Vec<ConstraintKind> {
    let mut constraints: Vec<ConstraintKind> = Vec::new();

    // Built-in constraints
    constraints.push(ConstraintKind::Row(RowConstraint::new(n)));
    constraints.push(ConstraintKind::Column(ColumnConstraint::new(n)));

    // Box shape: scale with board size if the params still hold the default
    let box_rows = params.box_shape[0];
    let box_cols = params.box_shape[1];
    if let Ok(bc) = BoxConstraint::new(n, (box_rows, box_cols)) {
        constraints.push(ConstraintKind::Box(bc));
    }

    // Extra constraints
    if params.diagonals {
        constraints.push(ConstraintKind::Diagonal(DiagonalConstraint::new(n)));
    }

    for cage in &params.cages {
        let cells: Vec<(usize, usize)> = cage.cells.iter().map(|&[r, c]| (r, c)).collect();
        if let Ok(cc) = KillerCageConstraint::new(n, cells, cage.sum) {
            constraints.push(ConstraintKind::KillerCage(cc));
        }
    }

    for thermo in &params.thermos {
        let cells: Vec<(usize, usize)> = thermo.iter().map(|&[r, c]| (r, c)).collect();
        if let Ok(tc) = ThermoConstraint::new(n, cells) {
            constraints.push(ConstraintKind::Thermo(tc));
        }
    }

    for slow in &params.slow_thermos {
        let cells: Vec<(usize, usize)> = slow.iter().map(|&[r, c]| (r, c)).collect();
        if let Ok(sc) = SlowThermoConstraint::new(n, cells) {
            constraints.push(ConstraintKind::SlowThermo(sc));
        }
    }

    for palindrome in &params.palindromes {
        let cells: Vec<(usize, usize)> = palindrome.iter().map(|&[r, c]| (r, c)).collect();
        if let Ok(pc) = PalindromeConstraint::new(n, cells) {
            constraints.push(ConstraintKind::Palindrome(pc));
        }
    }

    constraints
}

// ── Tests ───────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;
    use crate::solver::NumberPuzzleSolver;

    #[test]
    fn test_params_default() {
        let params = SudokuParams::default();
        assert_eq!(params.box_shape, [3, 3]);
        assert!(!params.diagonals);
        assert!(params.cages.is_empty());
        assert!(params.thermos.is_empty());
        assert!(params.palindromes.is_empty());
    }

    #[test]
    fn test_deserialize_solve_input() {
        let json = r#"{
            "board": [[5,3,0],[6,0,0],[0,9,8]],
            "params": {
                "box_shape": [3,3],
                "diagonals": true
            }
        }"#;
        let input: SolveInput = serde_json::from_str(json).unwrap();
        assert_eq!(input.board.len(), 3);
        assert!(input.params.diagonals);
    }

    #[test]
    fn test_deserialize_with_cages() {
        let json = r#"{
            "board": [[0,0],[0,0]],
            "params": {
                "box_shape": [1,2],
                "cages": [{"cells": [[0,0],[0,1]], "sum": 5}]
            }
        }"#;
        let input: SolveInput = serde_json::from_str(json).unwrap();
        assert_eq!(input.params.cages.len(), 1);
        assert_eq!(input.params.cages[0].sum, 5);
    }

    #[test]
    fn test_build_constraints_basic() {
        let params = SudokuParams::default();
        let constraints = build_sudoku_constraints(9, &params);
        assert!(constraints.len() >= 3); // Row + Col + Box
    }

    #[test]
    fn test_build_constraints_with_diagonals() {
        let params = SudokuParams {
            diagonals: true,
            ..Default::default()
        };
        let constraints = build_sudoku_constraints(9, &params);
        // Should have Row + Col + Box + Diagonal = 4
        assert_eq!(constraints.len(), 4);
    }

    #[test]
    fn test_slow_thermo_deserialize() {
        let json = r#"{
            "board": [[0,0,0],[0,0,0],[0,0,0]],
            "params": {
                "box_shape": [1,3],
                "slow_thermos": [[[0,0],[0,1],[0,2]]]
            }
        }"#;
        let input: SolveInput = serde_json::from_str(json).unwrap();
        assert_eq!(input.params.slow_thermos.len(), 1);
        assert_eq!(input.params.slow_thermos[0].len(), 3);
    }

    #[test]
    fn test_slow_thermo_solve() {
        let json = r#"{
            "board": [[0,0,0],[0,0,0],[0,0,0]],
            "params": {
                "box_shape": [1,3],
                "slow_thermos": [[[0,0],[0,1],[0,2]]]
            }
        }"#;
        let input: SolveInput = serde_json::from_str(json).unwrap();
        let n = input.board.len();
        let constraints = build_sudoku_constraints(n, &input.params);
        // Row + Col + Box + 1 slow thermo = 4
        assert_eq!(
            constraints.len(),
            4,
            "expected 4 constraints, got {}",
            constraints.len()
        );
        let mut solver = NumberPuzzleSolver::new(input.board, constraints).unwrap();
        let sol = solver.solve().unwrap();
        // Slow thermo: non-decreasing along path (each >= previous)
        assert!(
            sol[0][1] >= sol[0][0],
            "slow thermo violated: {} < {}",
            sol[0][1],
            sol[0][0]
        );
        assert!(
            sol[0][2] >= sol[0][1],
            "slow thermo violated: {} < {}",
            sol[0][2],
            sol[0][1]
        );
    }

    #[test]
    fn test_end_to_end_from_json() {
        let json = r#"{
            "board": [
                [5,3,0,0,7,0,0,0,0],
                [6,0,0,1,9,5,0,0,0],
                [0,9,8,0,0,0,0,6,0],
                [8,0,0,0,6,0,0,0,3],
                [4,0,0,8,0,3,0,0,1],
                [7,0,0,0,2,0,0,0,6],
                [0,6,0,0,0,0,2,8,0],
                [0,0,0,4,1,9,0,0,5],
                [0,0,0,0,8,0,0,7,9]
            ],
            "params": {}
        }"#;
        let input: SolveInput = serde_json::from_str(json).unwrap();
        let n = input.board.len();
        let constraints = build_sudoku_constraints(n, &input.params);
        let mut solver = NumberPuzzleSolver::new(input.board, constraints).unwrap();
        let sol = solver.solve().unwrap();
        assert_eq!(sol[0][2], 4);
        assert_eq!(sol[8][8], 9);
    }
}
