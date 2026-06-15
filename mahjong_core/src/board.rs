use std::collections::HashMap;

use crate::round::{Event, Input, Output, State};
use crate::structs::Wind;

/// What a player decided to do given their available options.
#[derive(Debug, Clone, Copy)]
pub enum Decision {
    /// Pick one of the available events.
    Pick(Event),
    /// Exit the game (e.g. user quit to menu).
    Exit,
}

/// Injected per-seat decision maker.
pub trait Player {
    fn decide(&self, options: &[Event]) -> Decision;
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

/// What `Board::run()` returns after the game loop finishes.
#[derive(Debug, Clone, Copy)]
pub enum BoardOutput {
    /// The game concluded normally (Hu or draw).
    GameConcluded(GameResult),
    /// A player exited mid-game (e.g. user quit to menu).
    UserExited,
}

/// Result of prompting a player — either a concrete action or an exit signal.
#[derive(Debug, Clone)]
enum Prompted {
    Action(Input),
    Exit,
}

// ── Board ──

/// Game driver wrapping the round state machine.
///
/// # Usage
///
/// ```ignore
/// use mahjong_core::board::*;
/// use mahjong_core::round::{Event, State};
/// use mahjong_core::structs::Wind;
///
/// struct Dummy;
/// impl Player for Dummy {
///     fn decide(&self, options: &[Event]) -> Decision {
///         Decision::Pick(options[0])
///     }
/// }
///
/// let mut state = State::new(Wind::East, Wind::East);
/// state.init();
/// let p = Dummy;
/// let mut board = Board::new(state, [&p; 4]);
/// board.run(|_e| {})?;
/// # Ok::<_, BoardError>(())
/// ```
///
/// Players are borrowed, not owned — the same player can be reused
/// across multiple rounds.
pub struct Board<'a, P: Player> {
    state: State,
    players: [&'a P; 4],
}

impl<'a, P: Player> Board<'a, P> {
    pub fn new(state: State, players: [&'a P; 4]) -> Self {
        Self { state, players }
    }

    pub fn state(&self) -> &State {
        &self.state
    }

    pub fn into_state(self) -> State {
        self.state
    }

    pub fn player(&self, seat: Wind) -> &P {
        self.players[seat as usize]
    }

    /// Collect decisions from all relevant players and wrap into `Input`.
    fn prompt(&self, output: &Output) -> Prompted {
        match output {
            Output::NeedSelfAction { options } => {
                match self.players[self.state.turn as usize].decide(options) {
                    Decision::Pick(event) => {
                        Prompted::Action(Input::SelfAction(event))
                    }
                    Decision::Exit => Prompted::Exit,
                }
            }
            Output::NeedDiscard { options } => {
                match self.players[self.state.turn as usize].decide(options) {
                    Decision::Pick(event) => {
                        Prompted::Action(Input::Discard(event))
                    }
                    Decision::Exit => Prompted::Exit,
                }
            }
            Output::NeedReactions { options } => {
                self.prompt_reactions(options)
            }
            Output::NeedDrawTile => unreachable!(),
        }
    }

    /// Collect reactions from each seat with options.
    fn prompt_reactions(&self, options: &[Event]) -> Prompted {
        let mut by_seat: HashMap<Wind, Vec<Event>> = HashMap::new();
        for event in options {
            by_seat.entry(event.seat()).or_default().push(*event);
        }
        let mut choices = Vec::new();
        for (seat, seat_options) in by_seat {
            match self.players[seat as usize].decide(&seat_options) {
                Decision::Pick(event) => choices.push(event),
                Decision::Exit => return Prompted::Exit,
            }
        }
        Prompted::Action(Input::Reactions(choices))
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
    fn validate_and_apply(
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

    /// Run the game loop until the round ends or a player exits.
    ///
    /// `on_event` is called for every committed event (draws, discards,
    /// reactions, turn advances). The game result is returned via
    /// [`BoardOutput`].
    ///
    /// Returns `Err` only on invalid player input — the caller typically
    /// unwraps, since the UI layer should guard against invalid choices.
    pub fn run(
        &mut self,
        mut on_event: impl FnMut(&Event),
    ) -> Result<BoardOutput, BoardError> {
        loop {
            match self.step(|e| on_event(e))? {
                StepResult::Waiting { output } => {
                    let input = match self.prompt(&output) {
                        Prompted::Action(input) => input,
                        Prompted::Exit => return Ok(BoardOutput::UserExited),
                    };
                    let event = self.validate_and_apply(&output, input)?;
                    on_event(&event);
                    if matches!(event, Event::Hu { .. }) {
                        return Ok(BoardOutput::GameConcluded(GameResult::Hu {
                            winner: event.seat(),
                        }));
                    }
                }
                StepResult::Over => {
                    return Ok(BoardOutput::GameConcluded(GameResult::Draw));
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
