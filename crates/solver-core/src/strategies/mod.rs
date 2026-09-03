pub mod hidden_single;
pub mod naked_single;

use crate::constraint::Unit;
use crate::constraints::ConstraintKind;
use crate::state::SolverState;
use crate::strategy::SolvingStrategy;

use hidden_single::HiddenSingleStrategy;
use naked_single::NakedSingleStrategy;

/// Compile-time dispatch over the deduction strategies enabled by default.
pub enum StrategyKind {
    NakedSingle(NakedSingleStrategy),
    HiddenSingle(HiddenSingleStrategy),
}

impl SolvingStrategy for StrategyKind {
    fn apply(&self, state: &mut SolverState) -> i32 {
        match self {
            Self::NakedSingle(strategy) => strategy.apply(state),
            Self::HiddenSingle(strategy) => strategy.apply(state),
        }
    }
}

/// Build the standard strategy pipeline from the supplied constraints.
///
/// Hidden-single detection needs the all-different units exposed by row,
/// column, box, and diagonal constraints. Other constraint types do not add
/// units to this strategy.
pub fn build_default_strategies(constraints: &[ConstraintKind]) -> Vec<StrategyKind> {
    let units: Vec<Unit> = constraints
        .iter()
        .flat_map(|constraint| {
            constraint
                .all_different_units()
                .unwrap_or_default()
                .iter()
                .cloned()
        })
        .collect();

    vec![
        StrategyKind::NakedSingle(NakedSingleStrategy),
        StrategyKind::HiddenSingle(HiddenSingleStrategy::new(units)),
    ]
}
