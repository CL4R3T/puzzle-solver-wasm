pub mod box_constraint;
pub mod column;
pub mod diagonal;
pub mod killer_cage;
pub mod palindrome;
pub mod row;
pub mod slow_thermo;
pub mod thermo;

use crate::constraint::{Constraint, Unit};
use crate::state::SolverState;
use crate::types::ValidationResult;

use box_constraint::BoxConstraint;
use column::ColumnConstraint;
use diagonal::DiagonalConstraint;
use killer_cage::KillerCageConstraint;
use palindrome::PalindromeConstraint;
use row::RowConstraint;
use slow_thermo::SlowThermoConstraint;
use thermo::ThermoConstraint;

/// Compile-time dispatch over all constraint types.
pub enum ConstraintKind {
    Row(RowConstraint),
    Column(ColumnConstraint),
    Box(BoxConstraint),
    Diagonal(DiagonalConstraint),
    KillerCage(KillerCageConstraint),
    Thermo(ThermoConstraint),
    SlowThermo(SlowThermoConstraint),
    Palindrome(PalindromeConstraint),
}

impl ConstraintKind {
    /// Return complete permutation units used by Sudoku-style deductions.
    ///
    /// Every returned unit contains `n` cells over the domain `1..=n`, so an
    /// all-different rule also requires every value to appear exactly once.
    /// This stronger invariant makes deductions such as hidden singles valid.
    /// Partial all-different groups, such as killer cages, are intentionally
    /// excluded because they do not require every value to have a position.
    pub(crate) fn all_different_units(&self) -> Option<&[Unit]> {
        match self {
            Self::Row(c) => Some(c.units()),
            Self::Column(c) => Some(c.units()),
            Self::Box(c) => Some(c.units()),
            Self::Diagonal(c) => Some(c.units()),
            Self::KillerCage(_) | Self::Thermo(_) | Self::SlowThermo(_) | Self::Palindrome(_) => {
                None
            }
        }
    }
}

impl Constraint for ConstraintKind {
    fn propagate(&self, state: &mut SolverState) -> i32 {
        match self {
            Self::Row(c) => c.propagate(state),
            Self::Column(c) => c.propagate(state),
            Self::Box(c) => c.propagate(state),
            Self::Diagonal(c) => c.propagate(state),
            Self::KillerCage(c) => c.propagate(state),
            Self::Thermo(c) => c.propagate(state),
            Self::SlowThermo(c) => c.propagate(state),
            Self::Palindrome(c) => c.propagate(state),
        }
    }

    fn validate(&self, state: &SolverState) -> ValidationResult {
        match self {
            Self::Row(c) => c.validate(state),
            Self::Column(c) => c.validate(state),
            Self::Box(c) => c.validate(state),
            Self::Diagonal(c) => c.validate(state),
            Self::KillerCage(c) => c.validate(state),
            Self::Thermo(c) => c.validate(state),
            Self::SlowThermo(c) => c.validate(state),
            Self::Palindrome(c) => c.validate(state),
        }
    }
}
