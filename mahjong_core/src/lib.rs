pub mod engine;
pub mod round;
pub mod solver;
pub mod structs;

// Re-exports for common usage.
pub use engine::{Engine, EngineOutput, EngineError, GameResult, PlayerDecision, Player};
pub use round::{GameEvent, PlayerAction};
pub use structs::Wind;
