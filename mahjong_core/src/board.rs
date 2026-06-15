use std::collections::HashMap;

use crate::round::{Event, Input, Output, State};
use crate::structs::Wind;

/// Injected per-seat decision maker.
pub trait Player {
    fn decide(&self, output: &Output) -> Event;
}

// ── Public types ──

/// Result of `Board::step()`.
#[derive(Debug, Clone)]
pub enum StepResult {
    Waiting { output: Output },
    Over,
}

#[derive(Debug, Clone)]
pub enum BoardError {
    InvalidInput(String),
}

// ── Board ──

/// Game driver wrapping the round state machine.
///
/// # Usage
///
/// ```ignore
/// let mut board = Board::new(state, [p0, p1, p2, p3]);
/// loop {
///     match board.step()? {
///         (events, StepResult::Waiting { output }) => {
///             for e in &events { println!("{e:?}"); }
///             let input = board.prompt(&output);
///             let event = board.decide(&output, input)?;
///             if matches!(event, Event::Hu { .. }) { break; }
///         }
///         (events, StepResult::Over) => {
///             for e in &events { println!("{e:?}"); }
///             break;
///         }
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

    pub fn state(&self) -> &State {
        &self.state
    }

    pub fn into_state(self) -> State {
        self.state
    }

    pub fn player(&self, seat: Wind) -> &P {
        &self.players[seat as usize]
    }

    /// Collect decisions from all relevant players and wrap into `Input`.
    pub fn prompt(&self, output: &Output) -> Input {
        match output {
            Output::NeedSelfAction { .. } => Input::SelfAction(
                self.players[self.state.turn as usize].decide(output),
            ),
            Output::NeedDiscard { .. } => Input::Discard(
                self.players[self.state.turn as usize].decide(output),
            ),
            Output::NeedReactions { options } => {
                self.prompt_reactions(options)
            }
            Output::NeedDrawTile => unreachable!(),
        }
    }

    /// Collect reactions from each seat with options.
    fn prompt_reactions(&self, options: &[Event]) -> Input {
        let mut by_seat: HashMap<Wind, Vec<Event>> = HashMap::new();
        for event in options {
            by_seat.entry(event.seat()).or_default().push(*event);
        }
        let mut choices = Vec::new();
        for (seat, seat_options) in by_seat {
            let filtered = Output::NeedReactions {
                options: seat_options,
            };
            choices.push(self.players[seat as usize].decide(&filtered));
        }
        Input::Reactions(choices)
    }

    /// Auto-advance and return all committed events with the result.
    pub fn step(&mut self) -> Result<(Vec<Event>, StepResult), BoardError> {
        let mut events = Vec::new();
        loop {
            let output = self.state.query();
            match output {
                Output::NeedDrawTile => {
                    if self.state.wall.is_empty() {
                        return Ok((events, StepResult::Over));
                    }
                    events.push(self.state.apply(Input::DrawTile));
                }
                Output::NeedSelfAction { ref options }
                    if options.is_empty() =>
                {
                    self.state.apply(Input::SelfAction(Event::Skip {
                        seat: self.state.turn,
                    }));
                }
                Output::NeedReactions { ref options }
                    if options.is_empty() =>
                {
                    self.state.apply(Input::Reactions(vec![]));
                }
                output => {
                    return Ok((events, StepResult::Waiting { output }));
                }
            }
        }
    }

    /// Validate and apply a player's decision.
    pub fn decide(
        &mut self,
        expected: &Output,
        input: Input,
    ) -> Result<Event, BoardError> {
        match (expected, &input) {
            (Output::NeedSelfAction { options }, Input::SelfAction(event)) => {
                check_in_options(event, options, "self-action")?;
                Ok(self.state.apply(input))
            }

            (Output::NeedDiscard { options }, Input::Discard(event)) => {
                check_in_options(event, options, "discard")?;
                Ok(self.state.apply(input))
            }

            (Output::NeedReactions { options }, Input::Reactions(choices)) => {
                check_reactions(choices, options, self.state.turn)?;
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

/// Validates that `event` is in the available `options`.
fn check_in_options(
    event: &Event,
    options: &[Event],
    ctx: &str,
) -> Result<(), BoardError> {
    if options.contains(event) {
        Ok(())
    } else {
        Err(BoardError::InvalidInput(format!(
            "{} {:?} not in options",
            ctx, event
        )))
    }
}

/// Validates reaction choices: no turn seat, no duplicates, each in options.
fn check_reactions(
    choices: &[Event],
    options: &[Event],
    turn: Wind,
) -> Result<(), BoardError> {
    let mut seen = 0u8;
    for choice in choices {
        let seat = choice.seat();
        if seat == turn {
            return Err(BoardError::InvalidInput(format!(
                "reaction from current turn seat {:?}",
                seat
            )));
        }
        let bit = 1u8 << (seat as u8);
        if seen & bit != 0 {
            return Err(BoardError::InvalidInput(format!(
                "duplicate reaction from seat {:?}",
                seat
            )));
        }
        seen |= bit;
        check_in_options(choice, options, "reaction")?;
    }
    Ok(())
}
