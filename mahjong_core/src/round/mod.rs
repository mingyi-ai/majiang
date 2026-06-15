mod actions;
mod state;

pub use state::{Event, Phase, SeatState, State};

use crate::structs::{Wall, Wind};

#[derive(Debug, Clone)]
pub enum Output {
    NeedDrawTile,
    NeedSelfAction { options: Vec<Event> },
    NeedDiscard { options: Vec<Event> },
    NeedReactions { options: Vec<Event> },
}

#[derive(Debug, Clone)]
pub enum Input {
    DrawTile,
    SelfAction(Event),
    Discard(Event),
    Reactions(Vec<Event>),
}

impl State {
    /// Create an uninitialized state for low-level use (deserialization,
    /// replay). The wall is NOT shuffled and hands are empty. Most callers
    /// should use [`new_shuffled`] instead.
    pub fn new(wind: Wind, turn: Wind) -> Self {
        Self {
            wind,
            wall: Wall::new_mcr(),
            turn,
            phase: Phase::RequestDrawTile,
            seats: [SeatState::default(); 4],
        }
    }

    /// Create a fully initialized state: shuffle the wall using the
    /// caller-provided RNG (enabling deterministic seeds for testing).
    ///
    /// Hands are empty — tiles are dealt lazily when the board starts
    /// running, producing DrawTile events as part of the normal event
    /// stream.
    pub fn new_shuffled<R: rand::Rng>(
        wind: Wind,
        turn: Wind,
        rng: &mut R,
    ) -> Self {
        let mut wall = Wall::new_mcr();
        wall.shuffle(rng);
        Self {
            wind,
            wall,
            turn,
            phase: Phase::RequestDrawTile,
            seats: [SeatState::default(); 4],
        }
    }

    /// Deal 13 non-flower tiles to each seat. Yields one `DrawTile` event
    /// per tile drawn (including flowers, which trigger replacement draws).
    ///
    /// Must only be called on a fresh state with empty hands — called by
    /// `Board::run()` at the start of each round. Not safe for snapshot
    /// resume (see MEMORY.md).
    pub(crate) fn deal(&mut self) -> Vec<Event> {
        let mut events = Vec::new();
        for seat in Wind::iter() {
            let hand = &mut self.seats[seat as usize].hand;
            let mut non_flower_count = 13;
            while non_flower_count > 0 {
                let tile = self
                    .wall
                    .yield_tile()
                    .expect("Wall is empty during initial deal");

                hand.add_tile(tile);
                if !tile.is_flower() {
                    non_flower_count -= 1;
                }
                events.push(Event::DrawTile { seat, tile });
            }
        }
        events
    }
}

impl State {
    /// Peek at the current state. Never mutates.
    pub fn query(&self) -> Output {
        match self.phase {
            Phase::RequestDrawTile => Output::NeedDrawTile,
            Phase::RequestSelfAction(drawn_tile) => Output::NeedSelfAction {
                options: self.self_action_options(drawn_tile),
            },
            Phase::RequestDiscard => Output::NeedDiscard {
                options: self.discard_options(),
            },
            Phase::RequestReaction(tile) => Output::NeedReactions {
                options: self.reaction_options(tile),
            },
        }
    }

    /// Apply an input action, mutate state, return the committed event.
    /// Always returns an event — turn advances are explicit via `AdvanceTurnTo`.
    pub fn apply(&mut self, input: Input) -> Event {
        match (self.phase, input.clone()) {
            (Phase::RequestDrawTile, Input::DrawTile) => {
                let tile = self.draw_tile();
                if !tile.is_flower() {
                    self.phase = Phase::RequestSelfAction(tile);
                }
                Event::DrawTile {
                    seat: self.turn,
                    tile,
                }
            }
            (Phase::RequestSelfAction(_), Input::SelfAction(event)) => {
                self.phase = self.apply_self_action(event);
                event
            }
            (Phase::RequestDiscard, Input::Discard(event)) => {
                self.phase = self.apply_discard(event);
                event
            }
            (Phase::RequestReaction(tile), Input::Reactions(choices)) => {
                let event = self.resolve_reactions(&choices);
                (self.turn, self.phase) = self.apply_reaction(event, tile);
                event
            }
            _ => panic!(
                "Invalid phase-action combination: {:?} {:?}",
                self.phase, input
            ),
        }
    }
}
