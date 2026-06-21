pub(crate) mod array_vec;
pub mod engine;
pub mod round;
pub mod solver;
pub mod structs;

// Re-exports for common usage.
pub use engine::{
    Engine, EngineError, EngineOutput, GameResult, Player, PlayerDecision,
};
pub use round::{GameEvent, PlayerAction};
pub use structs::Wind;
