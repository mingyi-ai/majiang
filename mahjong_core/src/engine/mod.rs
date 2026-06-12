mod actions;
mod round;

use crate::structs::Tile;
pub use round::{Event, Phase, RoundState, Wind};
use std::collections::HashMap;

pub struct Engine {
    pub round: RoundState,
    request_buffer: Option<EngineOutput>,
}

pub enum EngineOutput {
    EventCommitted(Event),
    NeedSelfAction { player: Wind, options: Vec<Event> },
    NeedDiscard { player: Wind, options: Vec<Tile> },
    NeedReactions { options: HashMap<Wind, Vec<Event>> },
    RoundEnded { winner: Option<Wind> },
}

pub enum EngineInput {
    SelfAction(Event),
    Discard(Tile),
    Reactions(HashMap<Wind, Event>),
}

impl Engine {
    pub fn new(round_wind: Wind) -> Self {
        unimplemented!()
    }

    pub fn init_round(&mut self) {
        unimplemented!()
    }

    pub fn step(&mut self, input: EngineInput) -> EngineOutput {
        unimplemented!()
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
