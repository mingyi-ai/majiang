use crate::structs::{Hand, Tile, Wall, Wind};

#[derive(Clone, Copy, Default)]
pub struct SeatState {
    pub hand: Hand,
    pub discards: [Option<Tile>; 32],
}

impl SeatState {
    fn push_discard(&mut self, tile: Tile) {
        for slot in self.discards.iter_mut() {
            if slot.is_none() {
                *slot = Some(tile);
                return;
            }
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Phase {
    RequestDrawTile,
    RequestSelfAction(Tile),
    RequestDiscard,
    RequestReaction(Tile),
}

#[derive(Clone, Copy)]
pub struct State {
    pub wind: Wind,
    pub wall: Wall,
    pub turn: Wind,
    pub phase: Phase,
    pub seats: [SeatState; 4],
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum Event {
    DrawTile {
        seat: Wind,
        tile: Tile,
    },

    Skip {
        seat: Wind,
    },

    ConcealedKong {
        seat: Wind,
        tile: Tile,
    },
    AddedKong {
        seat: Wind,
        tile: Tile,
    },

    Chow {
        seat: Wind,
        tile: Tile,
        from: Wind,
        start: Tile,
    },
    Pong {
        seat: Wind,
        from: Wind,
        tile: Tile,
    },
    Kong {
        seat: Wind,
        from: Wind,
        tile: Tile,
    },
    Hu {
        seat: Wind,
        from: Wind,
        tile: Tile,
    },

    Discard {
        seat: Wind,
        tile: Tile,
    },

    AdvanceTurnTo {
        seat: Wind,
    },
}

impl Event {
    /// Extract the seat from any event variant.
    pub fn seat(&self) -> Wind {
        match self {
            Event::DrawTile { seat, .. }
            | Event::Skip { seat }
            | Event::ConcealedKong { seat, .. }
            | Event::AddedKong { seat, .. }
            | Event::Hu { seat, .. }
            | Event::Chow { seat, .. }
            | Event::Pong { seat, .. }
            | Event::Kong { seat, .. }
            | Event::Discard { seat, .. }
            | Event::AdvanceTurnTo { seat } => *seat,
        }
    }
}

impl State {
    /// Draw the next tile from the wall and add to the current turn's hand.
    ///
    /// Atomic: draws exactly one tile. Does not decide phase — the caller
    /// (`apply`) inspects the tile and sets `RequestSelfAction` or keeps
    /// `RequestDrawTile` for flowers.
    pub(crate) fn draw_tile(&mut self) -> Tile {
        let tile = self
            .wall
            .yield_tile()
            .expect("Wall is empty during normal play");

        self.seats[self.turn as usize].hand.add_tile(tile);
        tile
    }

    /// Returns the self-actions available for the current turn player
    /// given the tile just drawn.
    pub(crate) fn self_action_options(&self, drawn_tile: Tile) -> Vec<Event> {
        let hand = &self.seats[self.turn as usize].hand;
        let mut actions: Vec<Event> = Vec::new();

        for tile in hand.concealed_kong_options() {
            actions.push(Event::ConcealedKong {
                seat: self.turn,
                tile,
            });
        }

        for tile in hand.added_kong_options() {
            actions.push(Event::AddedKong {
                seat: self.turn,
                tile,
            });
        }

        // TODO: self hu check — needs hu solver integration
        if hand.can_hu() {
            actions.push(Event::Hu {
                seat: self.turn,
                from: self.turn,
                tile: drawn_tile,
            });
        }

        if !actions.is_empty() {
            actions.push(Event::Skip { seat: self.turn });
        }

        actions
    }

    /// Apply a self-action for the current turn player.
    /// Returns the resulting phase:
    ///   `ConcealedKong` / `AddedKong` / `Hu` → `RequestDrawTile`
    ///   `Skip` → `RequestDiscard`
    pub(crate) fn apply_self_action(&mut self, event: Event) -> Phase {
        let hand = &mut self.seats[self.turn as usize].hand;
        match event {
            Event::ConcealedKong { tile, .. } => {
                hand.kong(tile, true);
                Phase::RequestDrawTile
            }
            Event::AddedKong { tile, .. } => {
                hand.kong_from_pong(tile);
                Phase::RequestDrawTile
            }
            Event::Hu { .. } => Phase::RequestDrawTile,
            Event::Skip { seat } => {
                debug_assert_eq!(
                    seat, self.turn,
                    "Skip event seat must match current turn"
                );
                Phase::RequestDiscard
            }
            _ => panic!("Invalid event for self-action: {:?}", event),
        }
    }

    pub(crate) fn discard_options(&self) -> Vec<Event> {
        let concealed = &self.seats[self.turn as usize].hand.concealed;
        Tile::iter()
            .filter(|t| !t.is_flower() && concealed.count(*t) > 0)
            .map(|tile| Event::Discard {
                seat: self.turn,
                tile,
            })
            .collect()
    }

    /// Remove the discarded tile from hand. Returns the resulting phase
    /// (`RequestReaction` with the tile) for the caller to assign.
    pub(crate) fn apply_discard(&mut self, event: Event) -> Phase {
        let tile = match event {
            Event::Discard { tile, .. } => tile,
            _ => panic!("Invalid event for discard: {:?}", event),
        };
        self.seats[self.turn as usize].hand.remove_tile(tile, 1);
        Phase::RequestReaction(tile)
    }

    /// Apply resolved reaction event. Returns the resulting (seat, phase) for the caller to assign.
    pub(crate) fn apply_reaction(
        &mut self,
        event: Event,
        discard: Tile,
    ) -> (Wind, Phase) {
        match event {
            Event::Chow {
                seat, tile, start, ..
            } => {
                self.seats[seat as usize].hand.chow(start, tile);
                (seat, Phase::RequestDiscard)
            }
            Event::Pong { seat, tile, .. } => {
                self.seats[seat as usize].hand.pong(tile);
                (seat, Phase::RequestDiscard)
            }
            Event::Kong { seat, tile, .. } => {
                self.seats[seat as usize].hand.kong(tile, false);
                (seat, Phase::RequestDrawTile)
            }
            Event::Hu { seat, .. } => (seat, Phase::RequestDrawTile),
            Event::AdvanceTurnTo { seat } => {
                // All players skipped; place discard in river and advance turn
                self.seats[self.turn as usize].push_discard(discard);
                (seat, Phase::RequestDrawTile)
            }
            _ => panic!("Invalid event for reaction: {:?}", event),
        }
    }

    /// Compute possible reactions from all other seats for a given discard tile.
    /// Returns a flat list of all available reaction events across all seats.
    pub(crate) fn reaction_options(&self, tile: Tile) -> Vec<Event> {
        let mut all = Vec::new();
        for seat in Wind::iter() {
            if seat == self.turn {
                continue;
            }
            all.extend(self.reaction_options_by_seat(seat, tile));
        }
        all
    }

    /// Compute possible reactions for a single seat given a discard tile.
    /// Returns an empty vec if no reactions are possible for this seat.
    fn reaction_options_by_seat(&self, seat: Wind, tile: Tile) -> Vec<Event> {
        let hand = &self.seats[seat as usize].hand;
        let mut actions: Vec<Event> = Vec::new();

        // Only the next player in turn order can chow
        if seat == self.turn.next() {
            for start in hand.chow_start_options(tile) {
                actions.push(Event::Chow {
                    seat,
                    tile,
                    from: self.turn,
                    start,
                });
            }
        }

        if hand.can_pong(tile) {
            actions.push(Event::Pong {
                seat,
                from: self.turn,
                tile,
            });
        }

        if hand.can_kong(tile) {
            actions.push(Event::Kong {
                seat,
                from: self.turn,
                tile,
            });
        }

        if hand.can_hu_on(tile) {
            actions.push(Event::Hu {
                seat,
                from: self.turn,
                tile,
            });
        }

        if !actions.is_empty() {
            actions.push(Event::Skip { seat });
        }

        actions
    }

    /// Resolve a set of reaction choices to find the winning one.
    ///
    /// Priority: Hu(4) > Kong(3) > Pong(2) > Chow(1).
    /// Tiebreaker: closest seat clockwise from the discarder wins.
    /// If all players skipped (or choices is empty), returns `AdvanceTurnTo`.
    pub(crate) fn resolve_reactions(&self, choices: &[Event]) -> Event {
        debug_assert!(
            {
                let mut seen = 0u8;
                choices.iter().all(|e| {
                    let bit = 1u8 << (e.seat() as u8);
                    if seen & bit != 0 {
                        false
                    } else {
                        seen |= bit;
                        true
                    }
                })
            },
            "duplicate seat in reaction choices"
        );

        let priority = |event: &Event| -> u8 {
            match event {
                Event::Hu { .. } => 4,
                Event::Kong { .. } => 3,
                Event::Pong { .. } => 2,
                Event::Chow { .. } => 1,
                _ => 0,
            }
        };

        let distance =
            |seat: Wind| -> u8 { (seat as u8 + 4 - self.turn as u8) % 4 };

        let winner = choices
            .iter()
            .max_by_key(|event| {
                (priority(event), std::cmp::Reverse(distance(event.seat())))
            })
            .copied();

        match winner {
            Some(Event::Skip { .. }) | None => Event::AdvanceTurnTo {
                seat: self.turn.next(),
            },
            Some(event) => event,
        }
    }
}

// ── Legacy commented-out code (reference only) ──

// #[cfg(test)]
// mod tests {
//     use super::*;
//     use crate::bits::MahjongBitArray;
//     use crate::tile::Tile::*;

//     #[test]
//     fn test_possible_reactions_by_seat() {
//         let mut round = State::new(Wind::East);
//         {
//             let hand = &mut round.seats.get_mut(Wind::South).hand;
//             hand.tiles.add_tile(Dot2.id());
//             hand.tiles.add_tile(Dot3.id());
//             hand.tiles.add_tile(Dot5.id());
//             hand.tiles.add_tile(Dot6.id());
//         }
//         round.turn = Wind::East;
//         let tile = Dot4;
//         let reactions_south =
//             round.possible_reactions_by_seat(Wind::South, tile).unwrap();

//         assert_eq!(reactions_south.len(), 4);
//         assert!(reactions_south.contains(&Reaction::Chow {
//             seat: Wind::South,
//             tile,
//             from: Wind::East,
//             chow: [Dot2, Dot3]
//         }));
//         assert!(reactions_south.contains(&Reaction::Chow {
//             seat: Wind::South,
//             tile,
//             from: Wind::East,
//             chow: [Dot3, Dot5]
//         }));
//         assert!(reactions_south.contains(&Reaction::Chow {
//             seat: Wind::South,
//             tile,
//             from: Wind::East,
//             chow: [Dot5, Dot6]
//         }));
//     }
// }
