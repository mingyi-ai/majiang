mod actions;
mod round;

use crate::structs::{Hand, Tile};
pub use round::{Event, Phase, RoundState, SeatState, Wall, Wind};
use std::collections::HashMap;

// ── Public API types ──

#[derive(Debug, Clone)]
pub enum RoundOutput {
    NeedDrawTile { player: Wind },
    NeedSelfAction { player: Wind, options: Vec<Event> },
    NeedDiscard { player: Wind, options: Vec<Tile> },
    NeedReactions { options: HashMap<Wind, Vec<Event>> },
    RoundEnded { winner: Option<Wind> },
}

#[derive(Debug, Clone)]
pub enum RoundInput {
    DrawTile,
    SelfAction(Event),
    Discard(Tile),
    Reactions(HashMap<Wind, Event>),
}

// ── Lifecycle ──

impl RoundState {
    pub fn new() -> Self {
        Self {
            wind: Wind::East,
            wall: Wall::new_mcr(),
            turn: Wind::East,
            phase: Phase::RequestDrawTile,
            seats: core::array::from_fn(|_| SeatState {
                hand: Hand::default(),
                discards: [None; 32],
            }),
        }
    }

    /// Shuffle the wall, deal the initial 13 tiles to each seat,
    /// replace flowers, and set the first draw phase.
    pub fn init(&mut self) {
        unimplemented!()
    }
}

// ── Orchestration ──

impl RoundState {
    /// Peek at the current state. Never mutates.
    pub fn query(&self) -> RoundOutput {
        match self.phase {
            Phase::RequestDrawTile => {
                RoundOutput::NeedDrawTile { player: self.turn }
            }
            Phase::RequestSelfAction => {
                let options = self.legit_events();
                RoundOutput::NeedSelfAction {
                    player: self.turn,
                    options,
                }
            }
            Phase::RequestDiscard => {
                let options = self.legit_events();
                RoundOutput::NeedDiscard {
                    player: self.turn,
                    options: extract_discard_tiles(&options),
                }
            }
            Phase::RequestReaction(tile) => {
                let options = self.possible_reactions(tile);
                RoundOutput::NeedReactions { options }
            }
            Phase::RoundEnded => RoundOutput::RoundEnded { winner: None },
        }
    }

    /// Apply an input action, mutate state, return the committed event.
    /// Returns `None` only when all reaction participants skipped (no single winner event).
    pub fn apply(&mut self, input: RoundInput) -> Option<Event> {
        match (self.phase.clone(), input.clone()) {
            (Phase::RequestDrawTile, RoundInput::DrawTile) => {
                let tile = self.draw_tile();
                Some(Event::DrawTile {
                    seat: self.turn,
                    tile,
                })
            }
            (Phase::RequestSelfAction, RoundInput::SelfAction(event)) => {
                self.apply_self_action(event);
                Some(event)
            }
            (Phase::RequestDiscard, RoundInput::Discard(tile)) => {
                let seat = self.turn;
                self.add_discard(tile);

                let reactions = self.possible_reactions(tile);
                if reactions.is_empty() {
                    self.advance_turn();
                    self.phase = Phase::RequestDrawTile;
                } else {
                    self.phase = Phase::RequestReaction(tile);
                }

                Some(Event::Discard { seat, tile })
            }
            (Phase::RequestReaction(_), RoundInput::Reactions(choices)) => {
                match self.resolve_reactions(&choices) {
                    Some(winning) => {
                        self.apply_reaction(winning);
                        Some(winning)
                    }
                    None => {
                        self.advance_turn();
                        self.phase = Phase::RequestDrawTile;
                        None
                    }
                }
            }
            _ => panic!(
                "Invalid phase-action combination: {:?} {:?}",
                self.phase, input
            ),
        }
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
