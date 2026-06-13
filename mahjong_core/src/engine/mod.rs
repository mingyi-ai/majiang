mod actions;
mod round;

use crate::structs::{Hand, Tile};
pub use round::{Event, Phase, RoundState, SeatState, Wall, Wind};
use std::collections::HashMap;

pub struct Engine {
    pub round: RoundState,
}

#[derive(Debug, Clone)]
pub enum EngineOutput {
    NeedDrawTile { player: Wind },
    NeedSelfAction { player: Wind, options: Vec<Event> },
    NeedDiscard { player: Wind, options: Vec<Tile> },
    NeedReactions { options: HashMap<Wind, Vec<Event>> },
    RoundEnded { winner: Option<Wind> },
}

#[derive(Debug, Clone)]
pub enum EngineInput {
    DrawTile,
    SelfAction(Event),
    Discard(Tile),
    Reactions(HashMap<Wind, Event>),
}

impl Engine {
    pub fn new() -> Self {
        Self {
            round: RoundState {
                wind: Wind::East,
                wall: Wall::new_mcr(),
                turn: Wind::East,
                phase: Phase::RequestDrawTile,
                seats: core::array::from_fn(|_| SeatState {
                    hand: Hand::default(),
                    discards: [None; 32],
                }),
            },
        }
    }

    /// Shuffle the wall, deal the initial 13 tiles to each seat,
    /// replace flowers, and set the first draw phase.
    pub fn init_round(&mut self) {
        unimplemented!()
    }

    /// Peek at the current state. Never mutates.
    pub fn query(&self) -> EngineOutput {
        match self.round.phase {
            Phase::RequestDrawTile => EngineOutput::NeedDrawTile {
                player: self.round.turn,
            },
            Phase::RequestSelfAction => {
                let options = self.round.legit_events();
                EngineOutput::NeedSelfAction {
                    player: self.round.turn,
                    options,
                }
            }
            Phase::RequestDiscard => {
                let options = self.round.legit_events();
                EngineOutput::NeedDiscard {
                    player: self.round.turn,
                    options: Self::extract_discard_tiles(&options),
                }
            }
            Phase::RequestReaction => {
                let options = self.round.legit_events();
                EngineOutput::NeedReactions {
                    options: Self::group_reactions_by_seat(&options),
                }
            }
            Phase::RoundEnded => EngineOutput::RoundEnded { winner: None },
        }
    }

    /// Apply an input action, mutate state, return the committed event.
    /// Returns `None` only when all reaction participants skipped (no single winner event).
    pub fn apply(&mut self, input: EngineInput) -> Option<Event> {
        match (self.round.phase.clone(), input.clone()) {
            (Phase::RequestDrawTile, EngineInput::DrawTile) => {
                let tile = self.round.draw_tile();
                Some(Event::DrawTile {
                    seat: self.round.turn,
                    tile,
                })
            }
            (Phase::RequestSelfAction, EngineInput::SelfAction(event)) => {
                self.round.apply_self_action(event);
                Some(event)
            }
            (Phase::RequestDiscard, EngineInput::Discard(tile)) => {
                let seat = self.round.turn;
                self.round.add_discard(tile);

                // Check reactions from other players
                let reactions = self.round.possible_reactions(tile);
                if reactions.is_empty() {
                    // No one can react: advance turn, next player draws
                    self.round.advance_turn();
                    self.round.phase = Phase::RequestDrawTile;
                } else {
                    // Reactions exist: cache in RoundState, wait for input
                    self.round.phase = Phase::RequestReaction;
                }

                Some(Event::Discard { seat, tile })
            }
            (Phase::RequestReaction, EngineInput::Reactions(choices)) => {
                match self.round.resolve_reactions(&choices) {
                    Some(winning) => {
                        self.round.apply_reaction(winning);
                        Some(winning)
                    }
                    None => {
                        // All players skipped: advance turn, next player draws
                        self.round.advance_turn();
                        self.round.phase = Phase::RequestDrawTile;
                        None
                    }
                }
            }
            _ => panic!(
                "Invalid phase-action combination: {:?} {:?}",
                self.round.phase, input
            ),
        }
    }

    // ── helpers ──

    fn extract_discard_tiles(events: &[Event]) -> Vec<Tile> {
        events
            .iter()
            .filter_map(|e| match e {
                Event::Discard { tile, .. } => Some(*tile),
                _ => None,
            })
            .collect()
    }

    fn group_reactions_by_seat(events: &[Event]) -> HashMap<Wind, Vec<Event>> {
        let mut map: HashMap<Wind, Vec<Event>> = HashMap::new();
        for event in events {
            if let Some(seat) = Self::reaction_seat(event) {
                map.entry(seat).or_default().push(*event);
            }
        }
        map
    }

    fn reaction_seat(event: &Event) -> Option<Wind> {
        match event {
            Event::Chow { seat, .. }
            | Event::Pong { seat, .. }
            | Event::Kong { seat, .. }
            | Event::Hu { seat, .. }
            | Event::ReactionSkip { seat } => Some(*seat),
            _ => None,
        }
    }
}

// impl Engine {
//     pub fn step(
//         &mut self,
//         input: Option<EngineInput>,
//     ) -> Result<EngineEvent, EngineError> {
//         match input {
//             Some(EngineInput::Exit) => {
//                 println!("Exiting game engine.");
//                 return Ok(EngineEvent::EngineExited);
//             }
//             Some(EngineInput::NextRound) => {
//                 if self.round.phase != Phase::RoundEnded {
//                     return Err(EngineError::InvalidAction);
//                 }
//                 self.rounds_count += 1;
//                 self.rounds_since_wind_change += 1;
//                 if self.rounds_since_wind_change >= 4 {
//                     self.rounds_since_wind_change = 0;
//                     self.last_round_wind = self.last_round_wind.next();
//                 }

//                 self.seating_plan = SeatingPlan {
//                     east: self.seating_plan.south,
//                     south: self.seating_plan.west,
//                     west: self.seating_plan.north,
//                     north: self.seating_plan.east,
//                 };
//                 self.round = RoundState::new(self.last_round_wind);
//             }
//             _ => {}
//         }

//         let mut turn_events: Vec<EngineEvent> = Vec::new();

//         let to_return: EngineEvent = match self.round.phase {
//             Phase::RoundStarted => {
//                 turn_events.push(EngineEvent::RoundStarted {
//                     round_wind: self.round.round_wind,
//                 });
//                 self.round.wall.shuffle();
//                 turn_events.push(EngineEvent::WallShuffled);

//                 let dealt = self.round.deal_initial_hands()?;
//                 turn_events.push(EngineEvent::InitialHandsDealt(dealt));

//                 EngineEvent::RoundStarted {
//                     round_wind: self.round.round_wind,
//                 }
//             }
//             Phase::ExecuteDraw => {
//                 let drawn = self.round.execute_draw().unwrap_or_default();
//                 if drawn.is_empty() {
//                     let event = EngineEvent::RoundEnded { winner: None };
//                     turn_events.push(event.clone());
//                     self.round.phase = Phase::RoundEnded;
//                     event
//                 } else {
//                     let event = EngineEvent::from(PlayerAction::from(drawn));
//                     turn_events.push(event.clone());
//                     event
//                 }
//             }
//             Phase::RequestSelfAction => {
//                 let requests = self.round.request_self_action().unwrap();
//                 let event = EngineEvent::RequestAction(requests);
//                 turn_events.push(event.clone());
//                 event
//             }
//             Phase::ExecuteSelfAction => {
//                 let self_action = match input {
//                     Some(EngineInput::ChosenSelfAction(a)) => a,
//                     _ => return Err(EngineError::InvalidAction),
//                 };
//                 let self_action =
//                     self.round.execute_self_action(self_action).unwrap();
//                 let event = EngineEvent::from(PlayerAction::from(self_action));
//                 turn_events.push(event.clone());
//                 event
//             }
//             Phase::ExecuteReaction => {
//                 let reaction = self.round.execute_pending_reaction().unwrap();
//                 let event = EngineEvent::from(PlayerAction::from(reaction));
//                 turn_events.push(event.clone());
//                 event
//             }
//             Phase::RequestDiscard => {
//                 let requests = self.round.request_discard().unwrap();
//                 let event = EngineEvent::RequestDiscard(requests);
//                 turn_events.push(event.clone());
//                 event
//             }
//             Phase::ExecuteDiscard => {
//                 let discard_action = match input {
//                     Some(EngineInput::ChosenDiscard(d)) => d,
//                     _ => return Err(EngineError::InvalidAction),
//                 };
//                 let discard_event =
//                     self.round.execute_discard(discard_action)?;
//                 let event =
//                     EngineEvent::from(PlayerAction::from(discard_event));
//                 turn_events.push(event.clone());
//                 event
//             }
//             Phase::RequestReaction => {
//                 let requests = self.round.request_reaction().unwrap();
//                 let event = EngineEvent::RequestReaction(requests);
//                 turn_events.push(event.clone());
//                 event
//             }
//             Phase::HandleReactions => {
//                 let reactions = match input {
//                     Some(EngineInput::ChosenReactions(r)) => r,
//                     _ => return Err(EngineError::InvalidAction),
//                 };
//                 let proceed_event =
//                     self.round.handle_reactions(reactions).unwrap();
//                 let event = EngineEvent::from(proceed_event);
//                 turn_events.push(event.clone());
//                 event
//             }
//             Phase::RoundEnded => {
//                 let event = EngineEvent::RoundEnded {
//                     winner: self.round.winner,
//                 };
//                 turn_events.push(event.clone());
//                 self.rounds_count += 1;
//                 self.last_round_wind = self.round.round_wind;

//                 event
//             }
//         };

//         self.history.extend(turn_events);
//         Ok(to_return)
//     }
// }
