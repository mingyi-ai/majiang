use std::collections::HashMap;

use crate::round::{GameEvent, Input, Output, PlayerAction, State};
use crate::structs::Wind;

/// What a player decided to do given their available options.
#[derive(Debug, Clone, Copy)]
pub enum Decision {
    /// Pick one of the available actions.
    Pick(PlayerAction),
    /// Exit the game (e.g. user quit to menu).
    Exit,
}

/// Injected per-seat decision maker.
pub trait Player {
    fn decide(&self, options: &[PlayerAction]) -> Decision;
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

/// Result of prompting a player — either a concrete input or an exit signal.
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
/// use mahjong_core::round::PlayerAction;
/// use mahjong_core::structs::Wind;
/// use rand::rngs::StdRng;
/// use rand::SeedableRng;
///
/// struct Dummy;
/// impl Player for Dummy {
///     fn decide(&self, _options: &[PlayerAction]) -> Decision {
///         Decision::Exit
///     }
/// }
///
/// let p = Dummy;
/// let mut board = Board::new(
///     Wind::East, Wind::East,
///     [&p; 4],
///     &mut StdRng::seed_from_u64(42),
/// );
/// board.run(|_e| {}, |_e| {})?;
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
    /// Create a new board, shuffling the wall with the given RNG.
    /// Hands start empty — tiles are dealt at the start of [`run`].
    pub fn new<R: rand::Rng>(
        wind: Wind,
        turn: Wind,
        players: [&'a P; 4],
        rng: &mut R,
    ) -> Self {
        Self {
            state: State::new_shuffled(wind, turn, rng),
            players,
        }
    }

    /// Access the underlying round state (e.g. for inspection during
    /// snapshot or replay).
    pub fn state(&self) -> &State {
        &self.state
    }

    /// Consume the board and return the underlying round state.
    pub fn into_state(self) -> State {
        self.state
    }

    /// Access a player by seat.
    pub fn player(&self, seat: Wind) -> &P {
        self.players[seat as usize]
    }

    /// Collect decisions from all relevant players and wrap into `Input`.
    fn prompt(&self, output: &Output) -> Prompted {
        match output {
            Output::NeedSelfAction { options } => {
                match self.players[self.state.turn as usize].decide(options) {
                    Decision::Pick(action) => {
                        Prompted::Action(Input::SelfAction(action))
                    }
                    Decision::Exit => Prompted::Exit,
                }
            }
            Output::NeedDiscard { options } => {
                match self.players[self.state.turn as usize].decide(options) {
                    Decision::Pick(action) => {
                        Prompted::Action(Input::Discard(action))
                    }
                    Decision::Exit => Prompted::Exit,
                }
            }
            Output::NeedReactions { options } => self.prompt_reactions(options),
            Output::NeedDrawTile => unreachable!(),
        }
    }

    /// Collect reactions from each seat with options.
    fn prompt_reactions(&self, options: &[PlayerAction]) -> Prompted {
        let mut by_seat: HashMap<Wind, Vec<PlayerAction>> = HashMap::new();
        for action in options {
            by_seat.entry(action.seat()).or_default().push(*action);
        }
        let mut choices = Vec::new();
        for (seat, seat_options) in by_seat {
            match self.players[seat as usize].decide(&seat_options) {
                Decision::Pick(action) => choices.push(action),
                Decision::Exit => return Prompted::Exit,
            }
        }
        Prompted::Action(Input::Reactions(choices))
    }

    /// Auto-advance through mechanical phases. Calls `on_event` for each
    /// draw event produced during mechanical phases.
    fn step(
        &mut self,
        mut on_event: impl FnMut(&GameEvent),
    ) -> Result<StepResult, BoardError> {
        loop {
            let output = self.state.query();
            match output {
                Output::NeedDrawTile => {
                    if self.state.wall.is_empty() {
                        return Ok(StepResult::Over);
                    }
                    if let Some(event) = self.state.apply(Input::DrawTile) {
                        on_event(&event);
                    }
                }
                Output::NeedSelfAction { ref options }
                    if options.is_empty() =>
                {
                    self.state.apply(Input::SelfAction(PlayerAction::Skip {
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
    ) -> Result<Option<GameEvent>, BoardError> {
        match (expected, &input) {
            (
                Output::NeedSelfAction { options },
                Input::SelfAction(action),
            ) => {
                check_in_options(action, options, "self-action")?;
                Ok(self.state.apply(input))
            }

            (Output::NeedDiscard { options }, Input::Discard(action)) => {
                check_in_options(action, options, "discard")?;
                Ok(self.state.apply(input))
            }

            (
                Output::NeedReactions { options },
                Input::Reactions(choices),
            ) => {
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
    /// 1. **Deal phase**: the initial 13 tiles are dealt to each seat.
    ///    Each deal event is forwarded to `on_initial_deal`. In a real UI,
    ///    each player would see only their own deal events (private).
    ///
    /// 2. **Game loop**: `on_event` is called for every committed event
    ///    visible at the table (tile draws and real player actions).
    ///    Internal mechanics (skips, turn advances) are not reported.
    ///
    /// The event feed is sufficient for the caller to reconstruct game
    /// state: a Skip is implicit when a Discard follows a DrawTile
    /// without an intervening Kong/Hu event.
    ///
    /// Returns `Err` only on invalid player input — the caller typically
    /// unwraps, since the UI layer should guard against invalid choices.
    pub fn run(
        &mut self,
        mut on_initial_deal: impl FnMut(&GameEvent),
        mut on_event: impl FnMut(&GameEvent),
    ) -> Result<BoardOutput, BoardError> {
        // Deal initial tiles. Events go to the deal callback so the UI
        // can handle them privately per-player.
        for event in self.state.deal() {
            on_initial_deal(&event);
        }

        loop {
            match self.step(|e| on_event(e))? {
                StepResult::Waiting { output } => {
                    let input = match self.prompt(&output) {
                        Prompted::Action(input) => input,
                        Prompted::Exit => return Ok(BoardOutput::UserExited),
                    };
                    if let Some(event) = self.validate_and_apply(&output, input)? {
                        on_event(&event);
                        if matches!(&event, GameEvent::Action(PlayerAction::Hu { .. })) {
                            return Ok(BoardOutput::GameConcluded(GameResult::Hu {
                                winner: event.seat(),
                            }));
                        }
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

/// Validates that `action` is in the available `options`.
fn check_in_options(
    action: &PlayerAction,
    options: &[PlayerAction],
    ctx: &str,
) -> Result<(), BoardError> {
    if options.contains(action) {
        Ok(())
    } else {
        Err(BoardError::InvalidInput(format!(
            "{} {:?} not in options",
            ctx, action
        )))
    }
}

/// Validates reaction choices: no turn seat, no duplicates, each in options.
fn check_reactions(
    choices: &[PlayerAction],
    options: &[PlayerAction],
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
