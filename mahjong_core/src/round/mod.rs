mod actions;
mod state;

pub use state::{GameEvent, Phase, PlayerAction, SeatState, State};

use crate::structs::{Wall, Wind};

#[derive(Debug, Clone)]
pub enum Output {
    NeedDrawTile,
    NeedSelfAction { options: Vec<PlayerAction> },
    NeedDiscard { options: Vec<PlayerAction> },
    NeedReactions { options: Vec<PlayerAction> },
}

#[derive(Debug, Clone)]
pub enum Input {
    DrawTile,
    SelfAction(PlayerAction),
    Discard(PlayerAction),
    Reactions(Vec<PlayerAction>),
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
    /// Hands are empty — tiles are dealt when the board starts running.
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

    /// Deal 13 non-flower tiles to each seat, following the standard
    /// alternating draw sequence: East → South → West → North → East → …
    ///
    /// When a flower is drawn, the replacement tile is drawn immediately
    /// for the same seat (inline flower replacement).
    ///
    /// Yields one `GameEvent::DrawTile` per tile drawn.
    /// Called by [`Engine::run`](crate::engine::Engine::run) at the start
    /// of each round.
    pub(crate) fn deal(&mut self) -> Vec<GameEvent> {
        const HAND_TILE_COUNTS: usize = 13;
        let mut events = Vec::new();
        let order = [
            self.wind,
            self.wind.next(),
            self.wind.next().next(),
            self.wind.next().next().next(),
        ];

        for _ in 0..HAND_TILE_COUNTS {
            for seat in order {
                // Draw for this seat; if flower, draw again immediately.
                loop {
                    let tile = self
                        .wall
                        .yield_tile()
                        .expect("Wall is empty during initial deal");

                    self.seats[seat as usize].hand.add_tile(tile);
                    events.push(GameEvent::DrawTile { seat, tile });
                    if !tile.is_flower() {
                        break;
                    }
                }
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

    /// Apply an input action, mutate state.
    ///
    /// Returns `Some(GameEvent)` for events visible at the table
    /// (tile draws, real player actions). Returns `None` for internal
    /// mechanics (skips, turn advances) that the caller doesn't need
    /// to see — the flow is inferred from subsequent events.
    pub fn apply(&mut self, input: Input) -> Option<GameEvent> {
        match (self.phase, input.clone()) {
            (Phase::RequestDrawTile, Input::DrawTile) => {
                let tile = self.draw_tile();
                if !tile.is_flower() {
                    self.phase = Phase::RequestSelfAction(tile);
                }
                Some(GameEvent::DrawTile {
                    seat: self.turn,
                    tile,
                })
            }
            (Phase::RequestSelfAction(_), Input::SelfAction(action)) => {
                self.phase = self.apply_self_action(action);
                if matches!(action, PlayerAction::Skip { .. }) {
                    None // skip is internal; next event is the discard
                } else {
                    Some(GameEvent::Action(action))
                }
            }
            (Phase::RequestDiscard, Input::Discard(action)) => {
                self.phase = self.apply_discard(action);
                Some(GameEvent::Action(action))
            }
            (Phase::RequestReaction(tile), Input::Reactions(choices)) => {
                let action = self.resolve_reactions(&choices);
                (self.turn, self.phase) = self.apply_reaction(action, tile);
                action.map(GameEvent::Action)
            }
            _ => panic!(
                "Invalid phase-action combination: {:?} {:?}",
                self.phase, input
            ),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::structs::Tile;

    #[test]
    fn test_deal_exact_count_unshuffled() {
        let mut state = State::new(Wind::East, Wind::East);
        let events = state.deal();
        assert_eq!(events.len(), 52, "no flowers in unshuffled wall");
    }

    #[test]
    fn test_deal_alternating_sequence() {
        let mut state = State::new(Wind::East, Wind::East);
        let events = state.deal();
        for round in 0..13 {
            let base = round * 4;
            assert_eq!(
                events[base].seat(),
                Wind::East,
                "round {round}, event 0"
            );
            assert_eq!(
                events[base + 1].seat(),
                Wind::South,
                "round {round}, event 1"
            );
            assert_eq!(
                events[base + 2].seat(),
                Wind::West,
                "round {round}, event 2"
            );
            assert_eq!(
                events[base + 3].seat(),
                Wind::North,
                "round {round}, event 3"
            );
        }
    }

    #[test]
    fn test_deal_all_events_are_draw_tile() {
        let mut state = State::new(Wind::East, Wind::East);
        for (i, event) in state.deal().iter().enumerate() {
            assert!(
                matches!(event, GameEvent::DrawTile { .. }),
                "event {i} should be DrawTile, got {event:?}"
            );
        }
    }

    #[test]
    fn test_deal_rotation_from_each_start_wind() {
        for start in Wind::iter() {
            let mut state = State::new(start, start);
            let events = state.deal();
            assert_eq!(
                events[0].seat(),
                start,
                "first tile should go to {start:?}"
            );
        }
    }

    #[test]
    fn test_deal_flower_replacement_same_seat() {
        let mut state = State::new(Wind::East, Wind::East);

        // Replace the first four tiles in the alternating deal with
        // flowers (one per seat): positions 0 → East, 2 → South,
        // 4 → West, 6 → North. The replacement draws at positions
        // 1, 3, 5, 7 remain as-is (Character1, non-flower).
        let flowers = [
            (0, Tile::Plum),
            (2, Tile::Orchid),
            (4, Tile::Chrysanthemum),
            (6, Tile::BambooF),
        ];
        for &(idx, tile) in &flowers {
            state.wall.set_tile(idx, tile);
        }

        let events = state.deal();

        // 52 non-flower + 4 flower tiles
        assert_eq!(events.len(), 56, "4 flowers → 56 events");

        // Each seat's first tile is a flower, followed immediately
        // by a replacement draw for the same seat.
        let flower_positions = [0, 2, 4, 6];
        for &pos in &flower_positions {
            let flower = &events[pos];
            let replacement = &events[pos + 1];
            match (flower, replacement) {
                (
                    GameEvent::DrawTile { tile, seat },
                    GameEvent::DrawTile { seat: seat2, .. },
                ) => {
                    assert!(
                        tile.is_flower(),
                        "event {pos} should be a flower"
                    );
                    assert_eq!(
                        seat2, seat,
                        "replacement should be same seat as flower at {pos}"
                    );
                }
                _ => panic!("unexpected event at {pos}"),
            }
        }
    }

    #[test]
    fn test_deal_hand_counts() {
        use rand::SeedableRng;
        use rand::rngs::StdRng;

        let mut rng = StdRng::seed_from_u64(42);
        let mut state = State::new_shuffled(Wind::East, Wind::East, &mut rng);
        let events = state.deal();

        // Count non-flower tiles per seat from the event stream.
        let mut counts = [0u16; 4];
        for event in &events {
            if let GameEvent::DrawTile { seat, tile } = event {
                if !tile.is_flower() {
                    counts[*seat as usize] += 1;
                }
            }
        }

        for seat in Wind::iter() {
            assert_eq!(
                counts[seat as usize], 13,
                "{seat:?} should have 13 non-flower tiles"
            );
        }

        let flower_count = events.iter().filter(|e| {
            matches!(e, GameEvent::DrawTile { tile, .. } if tile.is_flower())
        }).count();
        assert_eq!(events.len(), 52 + flower_count);
    }
}
