pub(crate) mod fan_types;
mod search;

pub(crate) use fan_types::{
    FanCandidate, FanExclusionSet, FanInstance, FanSolveResult, FanType,
};
pub(crate) use search::solve_max_score;
