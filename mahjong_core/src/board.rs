use std::collections::HashMap;

use crate::round::{Event, Input, Output, State};
use crate::structs::Wind;

/// Injected per-seat decision maker.
pub trait Player {
    fn decide(&self, options: &[Event]) -> Event;
}

// ── Public types ──

/// Result of `Board::step()`.
#[derive(Debug, Clone)]
enum StepResult {
    Waiting { output: Output },
    Over,
}

#[derive(Debug, Clone)]
pub enum BoardError {
    InvalidInput(String),
}

/// How the game ended.
#[derive(Debug, Clone, Copy)]
pub enum GameResult {
    Hu { winner: Wind },
    Draw,
}

// ── Board ──

/// Game driver wrapping the round state machine.
///
/// # Usage
///
/// ```ignore
/// let mut board = Board::new(state, [p0, p1, p2, p3]);
/// board.run(
///     |e| println!("{e:?}"),
///     |result| match result {
///         GameResult::Hu { winner } => println!("{:?} wins!", winner),
///         GameResult::Draw => println!("Draw game"),
///     },
/// )?;
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
    fn prompt(&self, output: &Output) -> Input {
        match output {
            Output::NeedSelfAction { options } => Input::SelfAction(
                self.players[self.state.turn as usize].decide(options),
            ),
            Output::NeedDiscard { options } => Input::Discard(
                self.players[self.state.turn as usize].decide(options),
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
            choices.push(self.players[seat as usize].decide(&seat_options));
        }
        Input::Reactions(choices)
    }

    /// Auto-advance through mechanical phases. Calls `on_event` for each
    /// effective event (draws). Skips and turn-advances are absorbed.
    fn step(
        &mut self,
        mut on_event: impl FnMut(&Event),
    ) -> Result<StepResult, BoardError> {
        loop {
            let output = self.state.query();
            match output {
                Output::NeedDrawTile => {
                    if self.state.wall.is_empty() {
                        return Ok(StepResult::Over);
                    }
                    let event = self.state.apply(Input::DrawTile);
                    on_event(&event);
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
                    return Ok(StepResult::Waiting { output });
                }
            }
        }
    }

    /// Validate and apply a player's decision.
    fn decide(
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

    /// Run the game loop until the round ends.
    ///
    /// `on_event` is called for every committed event.
    /// `on_end` is called once with the game result (Hu winner or draw).
    ///
    /// Returns `Err` only on invalid player input — the caller typically
    /// unwraps, since the UI layer should guard against invalid choices.
    pub fn run(
        &mut self,
        mut on_event: impl FnMut(&Event),
        on_end: impl Fn(GameResult),
    ) -> Result<(), BoardError> {
        loop {
            match self.step(|e| on_event(e))? {
                StepResult::Waiting { output } => {
                    let input = self.prompt(&output);
                    let event = self.decide(&output, input)?;
                    on_event(&event);
                    if matches!(event, Event::Hu { .. }) {
                        on_end(GameResult::Hu {
                            winner: event.seat(),
                        });
                        return Ok(());
                    }
                }
                StepResult::Over => {
                    on_end(GameResult::Draw);
                    return Ok(());
                }
            }
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
