use std::ops::Index;

use crate::{
    event::{EngineEvent, PlayerAction, PlayerDiscard, Reaction, SelfAction},
    hand::HandError,
    round::{Phase, RoundState, Wind},
};

#[derive(Debug, Clone)]
pub enum EngineInput {
    ChosenSelfAction(SelfAction),
    ChosenReactions(Vec<Reaction>),
    ChosenDiscard(PlayerDiscard),
    NextRound,
    Exit,
}

#[derive(Debug, Clone, Copy)]
pub enum EngineError {
    InvalidAction,
    Hand(HandError),
}

impl From<HandError> for EngineError {
    fn from(e: HandError) -> Self {
        EngineError::Hand(e)
    }
}

pub type PlayerId = u32;

#[derive(Debug, Clone, Copy)]
pub struct SeatingPlan {
    pub east: PlayerId,
    pub south: PlayerId,
    pub west: PlayerId,
    pub north: PlayerId,
}

impl Index<Wind> for SeatingPlan {
    type Output = PlayerId;
    fn index(&self, w: Wind) -> &Self::Output {
        match w {
            Wind::East => &self.east,
            Wind::South => &self.south,
            Wind::West => &self.west,
            Wind::North => &self.north,
        }
    }
}

#[derive(Debug)]
pub struct Engine {
    pub round: RoundState,
    pub seating_plan: SeatingPlan,
    pub rounds_count: usize,
    pub last_round_wind: Wind,
    rounds_since_wind_change: usize,
    history: Vec<EngineEvent>,
}

impl Engine {
    pub fn new_human_vs_ai(
        round_wind: Wind,
        seating_plan: SeatingPlan,
    ) -> Self {
        Self {
            round: RoundState::new(round_wind),
            seating_plan,
            rounds_count: 0,
            last_round_wind: round_wind,
            rounds_since_wind_change: 0,
            history: Vec::new(),
        }
    }

    pub fn view_board(&self) -> &RoundState {
        &self.round
    }

    pub fn step(
        &mut self,
        input: Option<EngineInput>,
    ) -> Result<EngineEvent, EngineError> {
        match input {
            Some(EngineInput::Exit) => {
                println!("Exiting game engine.");
                return Ok(EngineEvent::EngineExited);
            }
            Some(EngineInput::NextRound) => {
                if self.round.phase != Phase::RoundEnded {
                    return Err(EngineError::InvalidAction);
                }
                self.rounds_count += 1;
                self.rounds_since_wind_change += 1;
                if self.rounds_since_wind_change >= 4 {
                    self.rounds_since_wind_change = 0;
                    self.last_round_wind = self.last_round_wind.next();
                }

                self.seating_plan = SeatingPlan {
                    east: self.seating_plan.south,
                    south: self.seating_plan.west,
                    west: self.seating_plan.north,
                    north: self.seating_plan.east,
                };
                self.round = RoundState::new(self.last_round_wind);
            }
            _ => {}
        }

        let mut turn_events: Vec<EngineEvent> = Vec::new();

        let to_return: EngineEvent = match self.round.phase {
            Phase::RoundStarted => {
                turn_events.push(EngineEvent::RoundStarted {
                    round_wind: self.round.round_wind,
                });
                self.round.wall.shuffle();
                turn_events.push(EngineEvent::WallShuffled);

                let dealt = self.round.deal_initial_hands()?;
                turn_events.push(EngineEvent::InitialHandsDealt(dealt));

                EngineEvent::RoundStarted {
                    round_wind: self.round.round_wind,
                }
            }
            Phase::ExecuteDraw => {
                let drawn = self.round.execute_draw()?;
                let event = EngineEvent::from(PlayerAction::from(drawn));
                turn_events.push(event.clone());
                event
            }
            Phase::RequestSelfAction => {
                let requests = self.round.request_self_action().unwrap();
                let event = EngineEvent::RequestAction(requests);
                turn_events.push(event.clone());
                event
            }
            Phase::ExecuteSelfAction => {
                let self_action = match input {
                    Some(EngineInput::ChosenSelfAction(a)) => a,
                    _ => return Err(EngineError::InvalidAction),
                };
                let self_action =
                    self.round.execute_self_action(self_action).unwrap();
                let event = EngineEvent::from(PlayerAction::from(self_action));
                turn_events.push(event.clone());
                event
            }
            Phase::ExecuteReaction => {
                let reaction = self.round.execute_pending_reaction().unwrap();
                let event = EngineEvent::from(PlayerAction::from(reaction));
                turn_events.push(event.clone());
                event
            }
            Phase::ExecuteKongRepl => {
                let repl_tiles = self.round.execute_kong_repl().unwrap();
                let event = EngineEvent::from(PlayerAction::from(repl_tiles));
                turn_events.push(event.clone());
                event
            }
            Phase::RequestDiscard => {
                let requests = self.round.request_discard().unwrap();
                let event = EngineEvent::RequestDiscard(requests);
                turn_events.push(event.clone());
                event
            }
            Phase::ExecuteDiscard => {
                let discard_action = match input {
                    Some(EngineInput::ChosenDiscard(d)) => d,
                    _ => return Err(EngineError::InvalidAction),
                };
                let discard_event =
                    self.round.execute_discard(discard_action)?;
                let event =
                    EngineEvent::from(PlayerAction::from(discard_event));
                turn_events.push(event.clone());
                event
            }
            Phase::RequestReaction => {
                let requests = self.round.request_reaction().unwrap();
                let event = EngineEvent::RequestReaction(requests);
                turn_events.push(event.clone());
                event
            }
            Phase::HandleReactions => {
                let reactions = match input {
                    Some(EngineInput::ChosenReactions(r)) => r,
                    _ => return Err(EngineError::InvalidAction),
                };
                let proceed_event =
                    self.round.handle_reactions(reactions).unwrap();
                let event = EngineEvent::from(proceed_event);
                turn_events.push(event.clone());
                event
            }
            Phase::RoundEnded => {
                let event = EngineEvent::RoundEnded {
                    winner: self.round.winner,
                };
                turn_events.push(event.clone());
                self.rounds_count += 1;
                self.last_round_wind = self.round.round_wind;

                event
            }
        };

        self.history.extend(turn_events);
        Ok(to_return)
    }
}
