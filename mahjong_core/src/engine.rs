use std::collections::HashMap;

use crate::round::{GameEvent, Input, Output, PlayerAction, State};
use crate::structs::Wind;

/// What a player chose to do — either pick an action or exit the game.
#[derive(Debug, Clone, Copy)]
pub enum PlayerDecision {
    /// Pick one of the available actions.
    Pick(PlayerAction),
    /// Exit the game (e.g. user quit to menu).
    Exit,
}

/// Injected per-seat decision maker.
pub trait Player {
    fn decide(&self, options: &[PlayerAction]) -> PlayerDecision;
}

// ── Internal helpers ──

#[derive(Debug, Clone)]
enum StepResult {
    Waiting { output: Output },
    Over,
}

#[derive(Debug, Clone)]
pub enum EngineError {
    InvalidInput(String),
}

/// How the game ended.
#[derive(Debug, Clone, Copy)]
pub enum GameResult {
    Hu { winner: Wind },
    Draw,
}

/// What [`Engine::run()`] returns after the game loop finishes.
#[derive(Debug, Clone, Copy)]
pub enum EngineOutput {
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

// ── Engine ──

/// Game driver wrapping the round state machine.
///
/// # Usage
///
/// ```ignore
/// use mahjong_core::{Engine, EngineError, PlayerDecision, Player, PlayerAction, Wind};
/// use rand::rngs::StdRng;
/// use rand::SeedableRng;
///
/// struct Dummy;
/// impl Player for Dummy {
///     fn decide(&self, _options: &[PlayerAction]) -> PlayerDecision {
///         PlayerDecision::Exit
///     }
/// }
///
/// let p = Dummy;
/// let mut engine = Engine::new(
///     Wind::East, Wind::East,
///     [&p; 4],
///     &mut StdRng::seed_from_u64(42),
/// );
/// engine.run(|_e| {}, |_e| {})?;
/// # Ok::<_, EngineError>(())
/// ```
///
/// Players are borrowed, not owned — the same player can be reused
/// across multiple rounds.
pub struct Engine<'a, P: Player> {
    state: State,
    players: [&'a P; 4],
}

impl<'a, P: Player> Engine<'a, P> {
    /// Create a new engine, shuffling the wall with the given RNG.
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

    /// Consume the engine and return the underlying round state.
    pub fn into_state(self) -> State {
        self.state
    }

    /// Access a player by seat.
    pub fn player(&self, seat: Wind) -> &P {
        self.players[seat as usize]
    }

    /// Collect plays from all relevant players and wrap into `Input`.
    fn prompt(&self, output: &Output) -> Prompted {
        match output {
            Output::NeedSelfAction { options } => {
                match self.players[self.state.turn as usize].decide(options) {
                    PlayerDecision::Pick(action) => {
                        Prompted::Action(Input::SelfAction(action))
                    }
                    PlayerDecision::Exit => Prompted::Exit,
                }
            }
            Output::NeedDiscard { options } => {
                match self.players[self.state.turn as usize].decide(options) {
                    PlayerDecision::Pick(action) => {
                        Prompted::Action(Input::Discard(action))
                    }
                    PlayerDecision::Exit => Prompted::Exit,
                }
            }
            Output::NeedReactions { options } => {
                self.prompt_reactions(options)
            }
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
                PlayerDecision::Pick(action) => choices.push(action),
                PlayerDecision::Exit => return Prompted::Exit,
            }
        }
        Prompted::Action(Input::Reactions(choices))
    }

    /// Auto-advance through mechanical phases. Calls `on_event` for each
    /// draw event produced during mechanical phases.
    fn step(
        &mut self,
        mut on_event: impl FnMut(&GameEvent),
    ) -> Result<StepResult, EngineError> {
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

    /// Validate and apply a player's play.
    fn validate_and_apply(
        &mut self,
        expected: &Output,
        input: Input,
    ) -> Result<Option<GameEvent>, EngineError> {
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

            (Output::NeedReactions { options }, Input::Reactions(choices)) => {
                check_reactions(choices, options, self.state.turn)?;
                Ok(self.state.apply(input))
            }

            _ => Err(EngineError::InvalidInput(format!(
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
    ) -> Result<EngineOutput, EngineError> {
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
                        Prompted::Exit => return Ok(EngineOutput::UserExited),
                    };
                    if let Some(event) =
                        self.validate_and_apply(&output, input)?
                    {
                        on_event(&event);
                        if matches!(
                            &event,
                            GameEvent::Action(PlayerAction::Hu { .. })
                        ) {
                            return Ok(EngineOutput::GameConcluded(
                                GameResult::Hu {
                                    winner: event.seat(),
                                },
                            ));
                        }
                    }
                }
                StepResult::Over => {
                    return Ok(EngineOutput::GameConcluded(GameResult::Draw));
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
) -> Result<(), EngineError> {
    if options.contains(action) {
        Ok(())
    } else {
        Err(EngineError::InvalidInput(format!(
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
) -> Result<(), EngineError> {
    let mut seen = 0u8;
    for choice in choices {
        let seat = choice.seat();
        if seat == turn {
            return Err(EngineError::InvalidInput(format!(
                "reaction from current turn seat {:?}",
                seat
            )));
        }
        let bit = 1u8 << (seat as u8);
        if seen & bit != 0 {
            return Err(EngineError::InvalidInput(format!(
                "duplicate reaction from seat {:?}",
                seat
            )));
        }
        seen |= bit;
        check_in_options(choice, options, "reaction")?;
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::round::Phase;
    use crate::structs::{Hand, Tile};
    use rand::SeedableRng;
    use rand::rngs::StdRng;

    /// A mock player that returns a canned decision.
    struct MockPlayer {
        decision: PlayerDecision,
        /// Track how many times decide was called.
        calls: std::cell::Cell<u32>,
    }

    impl MockPlayer {
        fn new(decision: PlayerDecision) -> Self {
            Self {
                decision,
                calls: std::cell::Cell::new(0),
            }
        }

        fn call_count(&self) -> u32 {
            self.calls.get()
        }
    }

    impl Player for MockPlayer {
        fn decide(&self, _options: &[PlayerAction]) -> PlayerDecision {
            self.calls.set(self.calls.get() + 1);
            self.decision
        }
    }

    /// A player that always picks the first available action.
    struct FirstOptionPlayer;
    impl Player for FirstOptionPlayer {
        fn decide(&self, options: &[PlayerAction]) -> PlayerDecision {
            PlayerDecision::Pick(options[0])
        }
    }

    fn hand_with(tiles: &[Tile]) -> Hand {
        let mut h = Hand::default();
        for &t in tiles {
            h.concealed.insert(t);
        }
        h
    }

    // ── Engine construction ──

    #[test]
    fn engine_new_creates_state() {
        let p = MockPlayer::new(PlayerDecision::Exit);
        let mut rng = StdRng::seed_from_u64(1);
        let engine = Engine::new(Wind::South, Wind::West, [&p; 4], &mut rng);
        assert_eq!(engine.state().wind, Wind::South);
        assert_eq!(engine.state().turn, Wind::West);
        assert_eq!(engine.state().phase, Phase::RequestDrawTile);
    }

    #[test]
    fn engine_player_returns_correct_seat() {
        let players = [
            MockPlayer::new(PlayerDecision::Exit),
            MockPlayer::new(PlayerDecision::Exit),
            MockPlayer::new(PlayerDecision::Exit),
            MockPlayer::new(PlayerDecision::Exit),
        ];
        let mut rng = StdRng::seed_from_u64(1);
        // We need different players per seat, so build Engine differently
        let refs: [&MockPlayer; 4] =
            [&players[0], &players[1], &players[2], &players[3]];
        let engine = Engine::new(Wind::East, Wind::East, refs, &mut rng);
        assert_eq!(
            engine.player(Wind::East) as *const _,
            &players[0] as *const _
        );
        assert_eq!(
            engine.player(Wind::South) as *const _,
            &players[1] as *const _
        );
        assert_eq!(
            engine.player(Wind::West) as *const _,
            &players[2] as *const _
        );
        assert_eq!(
            engine.player(Wind::North) as *const _,
            &players[3] as *const _
        );
    }

    #[test]
    fn engine_into_state_consumes() {
        let p = MockPlayer::new(PlayerDecision::Exit);
        let mut rng = StdRng::seed_from_u64(1);
        let engine = Engine::new(Wind::East, Wind::East, [&p; 4], &mut rng);
        let _state = engine.into_state();
        // Compile-time check: engine is consumed
    }

    // ── Engine::step ──

    #[test]
    fn step_auto_draws_then_skips_to_discard() {
        let p = MockPlayer::new(PlayerDecision::Exit);
        let mut rng = StdRng::seed_from_u64(1);
        let mut engine =
            Engine::new(Wind::East, Wind::East, [&p; 4], &mut rng);
        let mut events = Vec::new();
        let result = engine.step(|e| events.push(e.clone())).unwrap();
        assert!(matches!(result, StepResult::Waiting { .. }));
        assert_eq!(events.len(), 1);
        assert!(matches!(events[0], GameEvent::DrawTile { .. }));
        // After draw, the hand has 1 tile. Self-action options are empty
        // (no kong/hu possible), so step auto-skips to RequestDiscard.
        assert_eq!(engine.state.phase, Phase::RequestDiscard);
    }

    #[test]
    fn step_auto_skip_when_no_self_options() {
        let p = MockPlayer::new(PlayerDecision::Exit);
        let mut rng = StdRng::seed_from_u64(1);
        let mut engine =
            Engine::new(Wind::East, Wind::East, [&p; 4], &mut rng);
        // Step once to draw a tile
        let _ = engine.step(|_| {}).unwrap();
        // Now phase is RequestSelfAction with an empty hand → options empty
        // Step again should auto-skip to RequestDiscard
        let result = engine.step(|_| {}).unwrap();
        assert!(matches!(result, StepResult::Waiting { .. }));
        assert_eq!(engine.state.phase, Phase::RequestDiscard);
    }

    #[test]
    fn step_returns_over_when_wall_empty() {
        let p = MockPlayer::new(PlayerDecision::Exit);
        let mut rng = StdRng::seed_from_u64(1);
        let mut engine =
            Engine::new(Wind::East, Wind::East, [&p; 4], &mut rng);
        // Drain the wall
        while engine.state.wall.yield_tile().is_some() {}
        // Reset phase to RequestDrawTile (simulate mid-game empty wall)
        engine.state.phase = Phase::RequestDrawTile;
        let result = engine.step(|_| {}).unwrap();
        assert!(matches!(result, StepResult::Over));
    }

    // ── Engine::prompt ──

    #[test]
    fn prompt_self_action_delegates_to_current_turn() {
        let p = MockPlayer::new(PlayerDecision::Pick(PlayerAction::Skip {
            seat: Wind::East,
        }));
        let mut rng = StdRng::seed_from_u64(1);
        let engine = Engine::new(Wind::East, Wind::East, [&p; 4], &mut rng);
        let output = Output::NeedSelfAction {
            options: vec![PlayerAction::Skip { seat: Wind::East }],
        };
        match engine.prompt(&output) {
            Prompted::Action(Input::SelfAction(PlayerAction::Skip {
                seat,
            })) => {
                assert_eq!(seat, Wind::East);
            }
            other => panic!("expected SelfAction, got {:?}", other),
        }
    }

    #[test]
    fn prompt_reactions_collects_all_seats() {
        let p_south =
            MockPlayer::new(PlayerDecision::Pick(PlayerAction::Skip {
                seat: Wind::South,
            }));
        let p_west =
            MockPlayer::new(PlayerDecision::Pick(PlayerAction::Skip {
                seat: Wind::West,
            }));
        let p_north =
            MockPlayer::new(PlayerDecision::Pick(PlayerAction::Skip {
                seat: Wind::North,
            }));
        let p_east =
            MockPlayer::new(PlayerDecision::Pick(PlayerAction::Skip {
                seat: Wind::East,
            }));
        let refs: [&MockPlayer; 4] = [&p_east, &p_south, &p_west, &p_north];
        let mut rng = StdRng::seed_from_u64(1);
        let engine = Engine::new(Wind::East, Wind::East, refs, &mut rng);

        let options = vec![
            PlayerAction::Skip { seat: Wind::South },
            PlayerAction::Skip { seat: Wind::West },
            PlayerAction::Skip { seat: Wind::North },
        ];
        match engine.prompt(&Output::NeedReactions { options }) {
            Prompted::Action(Input::Reactions(choices)) => {
                assert_eq!(choices.len(), 3);
            }
            other => panic!("expected Reactions, got {:?}", other),
        }
    }

    #[test]
    fn prompt_exit_on_player_exit() {
        let p = MockPlayer::new(PlayerDecision::Exit);
        let mut rng = StdRng::seed_from_u64(1);
        let engine = Engine::new(Wind::East, Wind::East, [&p; 4], &mut rng);
        let output = Output::NeedSelfAction {
            options: vec![PlayerAction::Skip { seat: Wind::East }],
        };
        assert!(matches!(engine.prompt(&output), Prompted::Exit));
    }

    // ── Engine::validate_and_apply ──

    #[test]
    fn validate_and_apply_valid_self_action() {
        let p = MockPlayer::new(PlayerDecision::Exit);
        let mut rng = StdRng::seed_from_u64(1);
        let mut engine =
            Engine::new(Wind::East, Wind::East, [&p; 4], &mut rng);
        engine.state.phase = Phase::RequestSelfAction(Tile::Character1);
        let output = Output::NeedSelfAction {
            options: vec![PlayerAction::Skip { seat: Wind::East }],
        };
        let result = engine.validate_and_apply(
            &output,
            Input::SelfAction(PlayerAction::Skip { seat: Wind::East }),
        );
        assert!(result.is_ok());
    }

    #[test]
    fn validate_and_apply_invalid_self_action_returns_error() {
        let p = MockPlayer::new(PlayerDecision::Exit);
        let mut rng = StdRng::seed_from_u64(1);
        let mut engine =
            Engine::new(Wind::East, Wind::East, [&p; 4], &mut rng);
        engine.state.phase = Phase::RequestSelfAction(Tile::Character1);
        let output = Output::NeedSelfAction {
            options: vec![PlayerAction::Skip { seat: Wind::East }],
        };
        let result = engine.validate_and_apply(
            &output,
            Input::SelfAction(PlayerAction::ConcealedKong {
                seat: Wind::East,
                tile: Tile::Red,
            }),
        );
        assert!(matches!(result, Err(EngineError::InvalidInput(_))));
    }

    #[test]
    fn validate_and_apply_phase_mismatch_returns_error() {
        let p = MockPlayer::new(PlayerDecision::Exit);
        let mut rng = StdRng::seed_from_u64(1);
        let mut engine =
            Engine::new(Wind::East, Wind::East, [&p; 4], &mut rng);
        let output = Output::NeedDiscard {
            options: vec![PlayerAction::Discard {
                seat: Wind::East,
                tile: Tile::Character1,
            }],
        };
        // Send SelfAction during a Discard phase
        let result = engine.validate_and_apply(
            &output,
            Input::SelfAction(PlayerAction::Skip { seat: Wind::East }),
        );
        assert!(matches!(result, Err(EngineError::InvalidInput(_))));
    }

    // ── check_in_options / check_reactions ──

    #[test]
    fn check_in_options_ok_when_present() {
        let action = PlayerAction::Skip { seat: Wind::East };
        let options = vec![PlayerAction::Skip { seat: Wind::East }];
        assert!(check_in_options(&action, &options, "test").is_ok());
    }

    #[test]
    fn check_in_options_err_when_absent() {
        let action = PlayerAction::Skip { seat: Wind::East };
        let options = vec![PlayerAction::Skip { seat: Wind::South }];
        assert!(matches!(
            check_in_options(&action, &options, "test"),
            Err(EngineError::InvalidInput(_))
        ));
    }

    #[test]
    fn check_reactions_rejects_current_turn_seat() {
        let choices = vec![PlayerAction::Skip { seat: Wind::East }];
        let options = vec![PlayerAction::Skip { seat: Wind::East }];
        assert!(matches!(
            check_reactions(&choices, &options, Wind::East),
            Err(EngineError::InvalidInput(msg)) if msg.contains("current turn")
        ));
    }

    #[test]
    fn check_reactions_rejects_duplicate_seat() {
        let choices = vec![
            PlayerAction::Skip { seat: Wind::South },
            PlayerAction::Skip { seat: Wind::South },
        ];
        let options = vec![PlayerAction::Skip { seat: Wind::South }];
        assert!(matches!(
            check_reactions(&choices, &options, Wind::East),
            Err(EngineError::InvalidInput(msg)) if msg.contains("duplicate")
        ));
    }

    // ── Engine::run (integration) ──

    #[test]
    fn run_user_exit_on_first_prompt() {
        let p = MockPlayer::new(PlayerDecision::Exit);
        let mut rng = StdRng::seed_from_u64(1);
        let mut engine =
            Engine::new(Wind::East, Wind::East, [&p; 4], &mut rng);
        let result = engine.run(|_| {}, |_| {});
        assert!(matches!(result, Ok(EngineOutput::UserExited)));
        // Player should have been prompted exactly once (after deal + auto-draw)
        assert_eq!(p.call_count(), 1);
    }

    #[test]
    fn run_concludes_with_first_option_player() {
        let mut rng = StdRng::seed_from_u64(1);
        let mut engine = Engine::new(
            Wind::East,
            Wind::East,
            [
                &FirstOptionPlayer,
                &FirstOptionPlayer,
                &FirstOptionPlayer,
                &FirstOptionPlayer,
            ],
            &mut rng,
        );
        let result = engine.run(|_| {}, |_| {});
        // With first-option picking, the game should progress to a conclusion
        // (draw when wall runs out, or hu if a hand happens to be complete).
        assert!(
            matches!(result, Ok(EngineOutput::GameConcluded(_))),
            "expected game concluded, got {:?}",
            result
        );
    }

    #[test]
    fn run_invalid_action_returns_error() {
        struct BadPlayer;
        impl Player for BadPlayer {
            fn decide(&self, options: &[PlayerAction]) -> PlayerDecision {
                // If discard options include Character1, pick it (valid).
                // Otherwise return a tile not in options.
                for &opt in options {
                    if let PlayerAction::Discard {
                        tile: Tile::Character1,
                        ..
                    } = opt
                    {
                        return PlayerDecision::Pick(opt);
                    }
                }
                PlayerDecision::Pick(PlayerAction::Discard {
                    seat: Wind::East,
                    tile: Tile::Dot9,
                })
            }
        }

        let mut rng = StdRng::seed_from_u64(1);
        let mut engine = Engine::new(
            Wind::East,
            Wind::East,
            [&BadPlayer, &BadPlayer, &BadPlayer, &BadPlayer],
            &mut rng,
        );
        // Give East a Dot3 so there's at least one discard option.
        // BadPlayer will try to discard Dot9 which isn't in hand → error.
        engine.state.seats[0].hand = hand_with(&[Tile::Dot3]);
        let result = engine.run(|_| {}, |_| {});
        assert!(matches!(result, Err(EngineError::InvalidInput(_))));
    }
}
