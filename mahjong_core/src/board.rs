use std::collections::HashMap;

use crate::round::{Event, Input, Output, State};
use crate::structs::Wind;

/// Injected per-seat decision maker.
///
/// Receives the full `Output` for context — variant + available options.
/// The caller (Board) asks the right player based on which seat the
/// decision is for (`Output` carries `player: Wind` or a reaction map).
pub trait Player {
    fn decide(&self, output: &Output) -> Input;
}

// ── Public types ──

/// Result of `Board::step()`. Returned when the internal auto-submit loop
/// can't proceed without a player decision, or the wall is empty.
#[derive(Debug, Clone)]
pub enum StepResult {
    /// A player decision is needed.
    Waiting { output: Output },
    /// Wall is empty — round ended with no winner.
    Over,
}

#[derive(Debug, Clone)]
pub enum BoardError {
    WallEmpty,
    InvalidInput(String),
}

// ── Board ──

/// Game driver wrapping the round state machine.
///
/// Owns the `State` and four per-seat `Player` injectees. The caller
/// drives the game by alternating `step()` (auto-advances through trivial
/// phases) and `decide()` (validates + applies a player's choice).
///
/// Resumable: lift the state via `into_state()`, reconstruct via `new()`.
///
/// # Usage
///
/// ```ignore
/// let mut board = Board::new(state, [p0, p1, p2, p3]);
/// loop {
///     match board.step()? {
///         StepResult::Waiting { output } => {
///             let input = match &output {
///                 Output::NeedReactions { .. } => {
///                     let mut choices = HashMap::new();
///                     for seat in output_players(&output, board.state().turn) {
///                         let choice = board.player(seat).decide(&output);
///                         choices.insert(seat, choice);
///                     }
///                     Input::Reactions(choices)
///                 }
///                 _ => {
///                     let seat = output_players(&output, board.state().turn)[0];
///                     board.player(seat).decide(&output)
///                 }
///             };
///             let event = board.decide(&output, input)?;
///             if is_hu(&event) { break; }
///         }
///         StepResult::Over => break,
///     }
/// }
/// ```
pub struct Board<P: Player> {
    state: State,
    players: [P; 4],
}

impl<P: Player> Board<P> {
    pub fn new(state: State, players: [P; 4]) -> Self {
        Self { state, players }
    }

    /// View the current round state (for serialization / resume).
    pub fn state(&self) -> &State {
        &self.state
    }

    /// Consume the Board and return the state.
    pub fn into_state(self) -> State {
        self.state
    }

    /// Access a player by seat (for the caller to call `decide` on).
    pub fn player(&self, seat: Wind) -> &P {
        &self.players[seat as usize]
    }

    // ── Game loop ──

    /// Auto-advance through trivial phases (draw, single-option self-action,
    /// empty reactions). Returns when a player decision is needed or the
    /// wall is empty.
    ///
    /// Hu is never produced by auto phases — it only occurs through
    /// explicit player choices via `decide()`. Use [`is_hu`] to check
    /// the returned event.
    ///
    /// See the [struct-level doc](Self) for a full usage loop.
    pub fn step(&mut self) -> Result<StepResult, BoardError> {
        loop {
            let output = self.state.query();
            match output {
                Output::NeedDrawTile { .. } => {
                    if self.state.wall.is_empty() {
                        return Ok(StepResult::Over);
                    }
                    self.state.apply(Input::DrawTile);
                    // Flower → loop redraws; non-flower → next iteration
                    // handles NeedSelfAction
                }

                Output::NeedSelfAction { ref options, .. }
                    if options.len() == 1 =>
                {
                    self.state.apply(Input::SelfAction(options[0]));
                    // Phase → RequestDiscard or RequestDrawTile; loop
                    // continues
                }

                Output::NeedReactions { ref options }
                    if options.is_empty() =>
                {
                    self.state.apply(Input::Reactions(HashMap::new()));
                    // resolve_reactions returns AdvanceTurnTo; loop
                    // continues to next player's draw
                }

                output => {
                    return Ok(StepResult::Waiting { output });
                }
            }
        }
    }

    /// Validate and apply a player's decision.
    ///
    /// `expected` must be the `Output` returned by the last `step()` call.
    /// The `input` is validated against it — variant must match, and the
    /// chosen event must be in the available options.
    ///
    /// Returns the committed `Event` on success, or `BoardError` on
    /// mismatch.
    pub fn decide(
        &mut self,
        expected: &Output,
        input: Input,
    ) -> Result<Event, BoardError> {
        match (expected, &input) {
            (
                Output::NeedSelfAction { options, .. },
                Input::SelfAction(event),
            ) => {
                if !options.contains(event) {
                    return Err(BoardError::InvalidInput(format!(
                        "self-action {:?} not in options {:?}",
                        event, options
                    )));
                }
                Ok(self.state.apply(input))
            }

            (Output::NeedDiscard { options, .. }, Input::Discard(event)) => {
                if !options.contains(event) {
                    return Err(BoardError::InvalidInput(format!(
                        "discard {:?} not in options",
                        event
                    )));
                }
                Ok(self.state.apply(input))
            }

            (Output::NeedReactions { options }, Input::Reactions(choices)) => {
                for (seat, event) in choices {
                    match options.get(seat) {
                        Some(seat_options) if seat_options.contains(event) => {
                        }
                        _ => {
                            return Err(BoardError::InvalidInput(format!(
                                "seat {:?} chose {:?}, not in options",
                                seat, event
                            )));
                        }
                    }
                }
                Ok(self.state.apply(input))
            }

            _ => Err(BoardError::InvalidInput(format!(
                "input {:?} doesn't match expected {:?}",
                input, expected
            ))),
        }
    }
}

// ── Helpers ──

/// Seats that need to make a decision for this output, ordered clockwise
/// from the current turn.
///
/// For `NeedSelfAction` / `NeedDiscard` this is always a single-element
/// vec. For `NeedReactions` it returns every seat in the options map,
/// sorted by proximity to the discarder.
pub fn output_players(output: &Output, turn: Wind) -> Vec<Wind> {
    let distance = |seat: Wind| -> u8 { (seat as u8 + 4 - turn as u8) % 4 };
    match output {
        Output::NeedDrawTile { .. } => vec![],
        Output::NeedSelfAction { player, .. }
        | Output::NeedDiscard { player, .. } => vec![*player],
        Output::NeedReactions { options } => {
            let mut seats: Vec<Wind> = options.keys().copied().collect();
            seats.sort_by_key(|&s| distance(s));
            seats
        }
    }
}

/// True if the event ends the round (Hu or SelfHu).
pub fn is_hu(event: &Event) -> bool {
    matches!(event, Event::Hu { .. } | Event::SelfHu { .. })
}
