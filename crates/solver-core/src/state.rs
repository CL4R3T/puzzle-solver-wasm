/// Holds the board state and bitmask domains for constraint solving.
///
/// Each cell's candidates are stored as a u32 bitmask:
/// bit 0 = value 1, bit 1 = value 2, …, bit (n-1) = value n.
/// A determined cell has exactly one bit set.
pub struct SolverState {
    /// Current board: 0 = empty, 1..n = filled value.
    pub cells: Vec<Vec<u32>>,
    /// Bitmask of possible values for each cell.  bit (v-1) set => value v is possible.
    pub pos: Vec<Vec<u32>>,
    /// Board side length.
    pub n: usize,
    /// Bitmask with all n bits set: (1 << n) - 1.
    pub all_mask: u32,
}

impl SolverState {
    /// Construct state from a square board.  Empty cells are 0.
    ///
    /// # Errors
    /// Returns an error string if the board is not square or contains out-of-range values.
    pub fn new(board: Vec<Vec<u32>>) -> Result<Self, String> {
        let n = board.len();
        if n == 0 || n > 16 {
            return Err(format!("board side length must be 1–16, got {}", n));
        }
        for (r, row) in board.iter().enumerate() {
            if row.len() != n {
                return Err(format!(
                    "board must be square: row {} has length {} (expected {})",
                    r,
                    row.len(),
                    n
                ));
            }
            for (c, &v) in row.iter().enumerate() {
                if v > n as u32 {
                    return Err(format!(
                        "cell ({},{}) value {} exceeds board size {}",
                        r, c, v, n
                    ));
                }
            }
        }

        let all_mask: u32 = if n == 32 { u32::MAX } else { (1u32 << n) - 1 };

        let mut pos = vec![vec![0u32; n]; n];
        for r in 0..n {
            for c in 0..n {
                let v = board[r][c];
                if v != 0 {
                    pos[r][c] = 1u32 << (v - 1);
                } else {
                    pos[r][c] = all_mask;
                }
            }
        }

        Ok(Self {
            cells: board,
            pos,
            n,
            all_mask,
        })
    }

    /// Collect all values (1-based) represented in a mask.
    /// Replaces Python's `_mask_to_values` generator.
    pub fn mask_to_values(&self, mask: u32) -> Vec<u32> {
        let mut vals = Vec::new();
        let mut m = mask;
        while m != 0 {
            let lsb = m & m.wrapping_neg(); // isolate lowest set bit
            let idx = lsb.trailing_zeros(); // 0-based index
            vals.push(idx + 1); // 1-based value
            m ^= lsb;
        }
        vals
    }

    /// Deep-clone the mutable parts for backtracking save points.
    /// Returns `(cells_snapshot, pos_snapshot)`.
    pub fn clone_state(&self) -> (Vec<Vec<u32>>, Vec<Vec<u32>>) {
        (self.cells.clone(), self.pos.clone())
    }

    /// Restore board and domains from a previously saved snapshot.
    pub fn restore_state(&mut self, cells: Vec<Vec<u32>>, pos: Vec<Vec<u32>>) {
        self.cells = cells;
        self.pos = pos;
    }

    /// Returns true when every cell is filled (no zeros remain).
    pub fn is_solved(&self) -> bool {
        self.cells.iter().all(|row| row.iter().all(|&v| v != 0))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_new_empty_board() {
        let state = SolverState::new(vec![vec![0u32; 3]; 3]).unwrap();
        assert_eq!(state.n, 3);
        assert_eq!(state.all_mask, 0b111);
        // every empty cell should have all three candidates
        for r in 0..3 {
            for c in 0..3 {
                assert_eq!(state.pos[r][c], 0b111);
            }
        }
    }

    #[test]
    fn test_new_with_values() {
        let board = vec![vec![1, 0], vec![0, 2]];
        let state = SolverState::new(board).unwrap();
        assert_eq!(state.pos[0][0], 0b01); // value 1
        assert_eq!(state.pos[0][1], 0b11); // candidates {1,2}
        assert_eq!(state.pos[1][0], 0b11);
        assert_eq!(state.pos[1][1], 0b10); // value 2
    }

    #[test]
    fn test_rejects_non_square() {
        assert!(SolverState::new(vec![vec![0, 0], vec![0]]).is_err());
    }

    #[test]
    fn test_rejects_out_of_range() {
        assert!(SolverState::new(vec![vec![0, 5], vec![0, 0]]).is_err());
    }

    #[test]
    fn test_mask_to_values() {
        let state = SolverState::new(vec![vec![0u32; 3]; 3]).unwrap();
        assert_eq!(state.mask_to_values(0b101), vec![1, 3]);
        assert_eq!(state.mask_to_values(0b010), vec![2]);
        assert_eq!(state.mask_to_values(0), Vec::<u32>::new());
    }

    #[test]
    fn test_clone_and_restore() {
        let board = vec![vec![1, 0], vec![0, 0]];
        let mut state = SolverState::new(board).unwrap();
        let (saved_cells, saved_pos) = state.clone_state();

        // mutate
        state.cells[0][1] = 2;
        state.pos[0][1] = 1 << 1;

        state.restore_state(saved_cells, saved_pos);
        assert_eq!(state.cells[0][1], 0);
        assert_eq!(state.pos[0][1], 0b11);
    }

    #[test]
    fn test_is_solved() {
        let board = vec![vec![1, 2], vec![2, 1]];
        let state = SolverState::new(board).unwrap();
        assert!(state.is_solved());

        let board2 = vec![vec![1, 0], vec![2, 1]];
        let state2 = SolverState::new(board2).unwrap();
        assert!(!state2.is_solved());
    }
}
