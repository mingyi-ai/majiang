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

/// A choice a player can make. These appear in `Output` options and are
/// returned by `Player::decide()`. Every variant has a clear actor.
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum PlayerAction {
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
}

impl PlayerAction {
    pub fn seat(&self) -> Wind {
        match self {
            PlayerAction::Skip { seat }
            | PlayerAction::ConcealedKong { seat, .. }
            | PlayerAction::AddedKong { seat, .. }
            | PlayerAction::Hu { seat, .. }
            | PlayerAction::Chow { seat, .. }
            | PlayerAction::Pong { seat, .. }
            | PlayerAction::Kong { seat, .. }
            | PlayerAction::Discard { seat, .. } => *seat,
        }
    }
}

/// A notification from the engine to the caller. These are the events
/// visible at the table: tiles drawn and actions taken. Internal
/// mechanics (skips, turn advances) are not reported.
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum GameEvent {
    DrawTile { seat: Wind, tile: Tile },
    Action(PlayerAction),
}

impl GameEvent {
    pub fn seat(&self) -> Wind {
        match self {
            GameEvent::DrawTile { seat, .. } => *seat,
            GameEvent::Action(action) => action.seat(),
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
    pub(crate) fn self_action_options(
        &self,
        drawn_tile: Tile,
    ) -> Vec<PlayerAction> {
        let hand = &self.seats[self.turn as usize].hand;
        let mut actions: Vec<PlayerAction> = Vec::new();

        for tile in hand.concealed_kong_options() {
            actions.push(PlayerAction::ConcealedKong {
                seat: self.turn,
                tile,
            });
        }

        for tile in hand.added_kong_options() {
            actions.push(PlayerAction::AddedKong {
                seat: self.turn,
                tile,
            });
        }

        if hand.can_hu() {
            actions.push(PlayerAction::Hu {
                seat: self.turn,
                from: self.turn,
                tile: drawn_tile,
            });
        }

        if !actions.is_empty() {
            actions.push(PlayerAction::Skip { seat: self.turn });
        }

        actions
    }

    /// Apply a self-action for the current turn player.
    /// Returns the resulting phase:
    ///   `ConcealedKong` / `AddedKong` / `Hu` → `RequestDrawTile`
    ///   `Skip` → `RequestDiscard`
    pub(crate) fn apply_self_action(&mut self, action: PlayerAction) -> Phase {
        let hand = &mut self.seats[self.turn as usize].hand;
        match action {
            PlayerAction::ConcealedKong { tile, .. } => {
                hand.kong(tile, true);
                Phase::RequestDrawTile
            }
            PlayerAction::AddedKong { tile, .. } => {
                hand.kong_from_pong(tile);
                Phase::RequestDrawTile
            }
            PlayerAction::Hu { .. } => Phase::RequestDrawTile,
            PlayerAction::Skip { seat } => {
                debug_assert_eq!(
                    seat, self.turn,
                    "Skip action seat must match current turn"
                );
                Phase::RequestDiscard
            }
            _ => panic!("Invalid action for self-action: {:?}", action),
        }
    }

    pub(crate) fn discard_options(&self) -> Vec<PlayerAction> {
        let concealed = &self.seats[self.turn as usize].hand.concealed;
        Tile::iter()
            .filter(|t| !t.is_flower() && concealed.count(*t) > 0)
            .map(|tile| PlayerAction::Discard {
                seat: self.turn,
                tile,
            })
            .collect()
    }

    /// Remove the discarded tile from hand. Returns the resulting phase
    /// (`RequestReaction` with the tile) for the caller to assign.
    pub(crate) fn apply_discard(&mut self, action: PlayerAction) -> Phase {
        let tile = match action {
            PlayerAction::Discard { tile, .. } => tile,
            _ => panic!("Invalid action for discard: {:?}", action),
        };
        self.seats[self.turn as usize].hand.remove_tile(tile, 1);
        Phase::RequestReaction(tile)
    }

    /// Apply a resolved reaction action. Returns the resulting (seat, phase).
    ///
    /// `None` means all players skipped — the discard is placed in the river
    /// and the turn advances.
    pub(crate) fn apply_reaction(
        &mut self,
        action: Option<PlayerAction>,
        discard: Tile,
    ) -> (Wind, Phase) {
        match action {
            Some(PlayerAction::Chow {
                seat, tile, start, ..
            }) => {
                self.seats[seat as usize].hand.chow(start, tile);
                (seat, Phase::RequestDiscard)
            }
            Some(PlayerAction::Pong { seat, tile, .. }) => {
                self.seats[seat as usize].hand.pong(tile);
                (seat, Phase::RequestDiscard)
            }
            Some(PlayerAction::Kong { seat, tile, .. }) => {
                self.seats[seat as usize].hand.kong(tile, false);
                (seat, Phase::RequestDrawTile)
            }
            Some(PlayerAction::Hu { seat, .. }) => {
                (seat, Phase::RequestDrawTile)
            }
            None => {
                // All players skipped; place discard in river and advance turn
                self.seats[self.turn as usize].push_discard(discard);
                (self.turn.next(), Phase::RequestDrawTile)
            }
            _ => panic!("Invalid action for reaction: {:?}", action),
        }
    }

    /// Compute possible reactions from all other seats for a given discard tile.
    /// Returns a flat list of all available reaction actions across all seats.
    pub(crate) fn reaction_options(&self, tile: Tile) -> Vec<PlayerAction> {
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
    fn reaction_options_by_seat(
        &self,
        seat: Wind,
        tile: Tile,
    ) -> Vec<PlayerAction> {
        let hand = &self.seats[seat as usize].hand;
        let mut actions: Vec<PlayerAction> = Vec::new();

        // Only the next player in turn order can chow
        if seat == self.turn.next() {
            for start in hand.chow_start_options(tile) {
                actions.push(PlayerAction::Chow {
                    seat,
                    tile,
                    from: self.turn,
                    start,
                });
            }
        }

        if hand.can_pong(tile) {
            actions.push(PlayerAction::Pong {
                seat,
                from: self.turn,
                tile,
            });
        }

        if hand.can_kong(tile) {
            actions.push(PlayerAction::Kong {
                seat,
                from: self.turn,
                tile,
            });
        }

        if hand.can_hu_on(tile) {
            actions.push(PlayerAction::Hu {
                seat,
                from: self.turn,
                tile,
            });
        }

        if !actions.is_empty() {
            actions.push(PlayerAction::Skip { seat });
        }

        actions
    }

    /// Resolve a set of reaction choices to find the winning one.
    ///
    /// Priority: Hu(4) > Kong(3) > Pong(2) > Chow(1).
    /// Tiebreaker: closest seat clockwise from the discarder wins.
    /// Returns `None` if all players skipped (or choices is empty).
    pub(crate) fn resolve_reactions(
        &self,
        choices: &[PlayerAction],
    ) -> Option<PlayerAction> {
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

        let priority = |action: &PlayerAction| -> u8 {
            match action {
                PlayerAction::Hu { .. } => 4,
                PlayerAction::Kong { .. } => 3,
                PlayerAction::Pong { .. } => 2,
                PlayerAction::Chow { .. } => 1,
                _ => 0,
            }
        };

        let distance =
            |seat: Wind| -> u8 { (seat as u8 + 4 - self.turn as u8) % 4 };

        let winner = choices
            .iter()
            .max_by_key(|action| {
                (priority(action), std::cmp::Reverse(distance(action.seat())))
            })
            .copied();

        match winner {
            Some(PlayerAction::Skip { .. }) | None => None,
            Some(action) => Some(action),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::structs::{Meld, Triplet};

    // ── Helpers ──

    fn state_with_turn(turn: Wind) -> State {
        State {
            wind: Wind::East,
            wall: Wall::new_mcr(),
            turn,
            phase: Phase::RequestDrawTile,
            seats: [SeatState::default(); 4],
        }
    }

    fn hand_with(tiles: &[Tile]) -> Hand {
        let mut h = Hand::default();
        for &t in tiles {
            h.concealed.insert(t);
        }
        h
    }

    // ── PlayerAction::seat ──

    #[test]
    fn player_action_seat_returns_correct_wind() {
        let seat = Wind::South;
        assert_eq!(PlayerAction::Skip { seat }.seat(), seat);
        assert_eq!(
            PlayerAction::ConcealedKong {
                seat,
                tile: Tile::Character1
            }
            .seat(),
            seat
        );
        assert_eq!(
            PlayerAction::AddedKong {
                seat,
                tile: Tile::Character1
            }
            .seat(),
            seat
        );
        assert_eq!(
            PlayerAction::Chow {
                seat,
                tile: Tile::Character1,
                from: Wind::East,
                start: Tile::Character1
            }
            .seat(),
            seat
        );
        assert_eq!(
            PlayerAction::Pong {
                seat,
                from: Wind::East,
                tile: Tile::Character1
            }
            .seat(),
            seat
        );
        assert_eq!(
            PlayerAction::Kong {
                seat,
                from: Wind::East,
                tile: Tile::Character1
            }
            .seat(),
            seat
        );
        assert_eq!(
            PlayerAction::Hu {
                seat,
                from: Wind::East,
                tile: Tile::Character1
            }
            .seat(),
            seat
        );
        assert_eq!(
            PlayerAction::Discard {
                seat,
                tile: Tile::Character1
            }
            .seat(),
            seat
        );
    }

    // ── GameEvent::seat ──

    #[test]
    fn game_event_seat_returns_correct_wind() {
        assert_eq!(
            GameEvent::DrawTile {
                seat: Wind::West,
                tile: Tile::Dot3
            }
            .seat(),
            Wind::West
        );
        assert_eq!(
            GameEvent::Action(PlayerAction::Skip { seat: Wind::North }).seat(),
            Wind::North
        );
    }

    // ── SeatState::push_discard ──

    #[test]
    fn push_discard_fills_consecutive_slots() {
        let mut seat = SeatState::default();
        seat.push_discard(Tile::Character1);
        seat.push_discard(Tile::Character2);
        seat.push_discard(Tile::Character3);

        assert_eq!(seat.discards[0], Some(Tile::Character1));
        assert_eq!(seat.discards[1], Some(Tile::Character2));
        assert_eq!(seat.discards[2], Some(Tile::Character3));
        assert!(seat.discards[3].is_none());
    }

    // ── State::draw_tile ──

    #[test]
    fn draw_tile_takes_from_wall_and_adds_to_hand() {
        let mut state = state_with_turn(Wind::East);
        // Fresh wall starts with Character1
        let tile = state.draw_tile();
        assert_eq!(tile, Tile::Character1);
        assert_eq!(state.seats[0].hand.concealed.count(Tile::Character1), 1);
    }

    #[test]
    fn draw_tile_uses_current_turn_seat() {
        let mut state = state_with_turn(Wind::South);
        let tile = state.draw_tile();
        assert_eq!(tile, Tile::Character1);
        assert_eq!(state.seats[1].hand.concealed.count(Tile::Character1), 1);
        // Other seats remain empty
        assert!(state.seats[0].hand.concealed.rows.iter().all(|&r| r == 0));
    }

    // ── State::self_action_options ──

    #[test]
    fn self_action_options_returns_empty_when_no_options() {
        let state = state_with_turn(Wind::East);
        let options = state.self_action_options(Tile::Character1);
        assert!(options.is_empty());
    }

    #[test]
    fn self_action_options_includes_concealed_kong() {
        let mut state = state_with_turn(Wind::East);
        // 4 copies of a tile → concealed kong available
        state.seats[0].hand =
            hand_with(&[Tile::Red, Tile::Red, Tile::Red, Tile::Red]);
        let options = state.self_action_options(Tile::Red);
        assert!(options.contains(&PlayerAction::ConcealedKong {
            seat: Wind::East,
            tile: Tile::Red,
        }));
        // Skip should also be present
        assert!(options.contains(&PlayerAction::Skip { seat: Wind::East }));
    }

    #[test]
    fn self_action_options_includes_added_kong() {
        let mut state = state_with_turn(Wind::East);
        // Exposed Pung(Dot5) + 1 extra Dot5 in concealed
        state.seats[0].hand = hand_with(&[Tile::Dot5]);
        state.seats[0].hand.melds[0] =
            Some(Meld::Pung(Triplet::new(Tile::Dot5, false)));

        let options = state.self_action_options(Tile::Dot5);
        assert!(options.contains(&PlayerAction::AddedKong {
            seat: Wind::East,
            tile: Tile::Dot5,
        }));
        assert!(options.contains(&PlayerAction::Skip { seat: Wind::East }));
    }

    #[test]
    fn self_action_options_includes_hu() {
        let mut state = state_with_turn(Wind::East);
        // Complete hand: 4 triplets + 1 pair
        state.seats[0].hand = hand_with(&[
            Tile::Character1,
            Tile::Character1,
            Tile::Character1,
            Tile::Character2,
            Tile::Character2,
            Tile::Character2,
            Tile::Character3,
            Tile::Character3,
            Tile::Character3,
            Tile::Character5,
            Tile::Character5,
            Tile::Character5,
            Tile::Character7,
            Tile::Character7,
        ]);
        let options = state.self_action_options(Tile::Character7);
        assert!(options.contains(&PlayerAction::Hu {
            seat: Wind::East,
            from: Wind::East,
            tile: Tile::Character7,
        }));
    }

    // ── State::apply_self_action ──

    #[test]
    fn apply_self_action_concealed_kong_returns_draw_phase() {
        let mut state = state_with_turn(Wind::East);
        state.seats[0].hand =
            hand_with(&[Tile::Red, Tile::Red, Tile::Red, Tile::Red]);
        let phase = state.apply_self_action(PlayerAction::ConcealedKong {
            seat: Wind::East,
            tile: Tile::Red,
        });
        assert_eq!(phase, Phase::RequestDrawTile);
        // All 4 copies consumed
        assert_eq!(state.seats[0].hand.concealed.count(Tile::Red), 0);
    }

    #[test]
    fn apply_self_action_added_kong_returns_draw_phase() {
        let mut state = state_with_turn(Wind::East);
        // Exposed Pung(Bamboo5) + 1 extra Bamboo5 in concealed for the upgrade
        state.seats[0].hand = hand_with(&[Tile::Bamboo5]);
        state.seats[0].hand.melds[0] =
            Some(Meld::Pung(Triplet::new(Tile::Bamboo5, false)));

        let phase = state.apply_self_action(PlayerAction::AddedKong {
            seat: Wind::East,
            tile: Tile::Bamboo5,
        });
        assert_eq!(phase, Phase::RequestDrawTile);
        // 1 tile consumed by the upgrade
        assert_eq!(state.seats[0].hand.concealed.count(Tile::Bamboo5), 0);
        // Meld is now a Kong
        assert!(matches!(
            state.seats[0].hand.melds[0],
            Some(Meld::Kong(q)) if q.tile() == Tile::Bamboo5
        ));
    }

    #[test]
    fn apply_self_action_skip_returns_discard_phase() {
        let mut state = state_with_turn(Wind::West);
        let phase =
            state.apply_self_action(PlayerAction::Skip { seat: Wind::West });
        assert_eq!(phase, Phase::RequestDiscard);
    }

    // ── State::discard_options ──

    #[test]
    fn discard_options_returns_non_flower_tiles_in_concealed() {
        let mut state = state_with_turn(Wind::East);
        state.seats[0].hand = hand_with(&[Tile::Dot3, Tile::Bamboo5]);
        let options = state.discard_options();
        assert_eq!(options.len(), 2);
        assert!(options.contains(&PlayerAction::Discard {
            seat: Wind::East,
            tile: Tile::Dot3,
        }));
        assert!(options.contains(&PlayerAction::Discard {
            seat: Wind::East,
            tile: Tile::Bamboo5,
        }));
    }

    #[test]
    fn discard_options_excludes_flowers() {
        let mut state = state_with_turn(Wind::East);
        state.seats[0].hand = hand_with(&[Tile::Character1]);
        state.seats[0].hand.add_tile(Tile::Plum); // flower in hand
        let options = state.discard_options();
        assert_eq!(
            options,
            vec![PlayerAction::Discard {
                seat: Wind::East,
                tile: Tile::Character1,
            }]
        );
    }

    #[test]
    fn discard_options_empty_when_no_concealed_tiles() {
        let state = state_with_turn(Wind::East);
        let options = state.discard_options();
        assert!(options.is_empty());
    }

    // ── State::apply_discard ──

    #[test]
    fn apply_discard_removes_tile_and_returns_reaction_phase() {
        let mut state = state_with_turn(Wind::East);
        state.seats[0].hand = hand_with(&[Tile::Character3]);
        let phase = state.apply_discard(PlayerAction::Discard {
            seat: Wind::East,
            tile: Tile::Character3,
        });
        assert_eq!(phase, Phase::RequestReaction(Tile::Character3));
        assert_eq!(state.seats[0].hand.concealed.count(Tile::Character3), 0);
    }

    // ── State::apply_reaction ──

    #[test]
    fn apply_reaction_pong_gives_turn_to_responder() {
        let mut state = state_with_turn(Wind::East);
        // South has 2 copies of Dot3 to pong
        state.seats[1].hand = hand_with(&[Tile::Dot3, Tile::Dot3]);

        let (seat, phase) = state.apply_reaction(
            Some(PlayerAction::Pong {
                seat: Wind::South,
                from: Wind::East,
                tile: Tile::Dot3,
            }),
            Tile::Dot3,
        );
        assert_eq!(seat, Wind::South);
        assert_eq!(phase, Phase::RequestDiscard);
        // Pong consumed 2 copies
        assert_eq!(state.seats[1].hand.concealed.count(Tile::Dot3), 0);
    }

    #[test]
    fn apply_reaction_kong_gives_draw_phase() {
        let mut state = state_with_turn(Wind::East);
        state.seats[2].hand = hand_with(&[Tile::Red, Tile::Red, Tile::Red]);

        let (seat, phase) = state.apply_reaction(
            Some(PlayerAction::Kong {
                seat: Wind::West,
                from: Wind::East,
                tile: Tile::Red,
            }),
            Tile::Red,
        );
        assert_eq!(seat, Wind::West);
        assert_eq!(phase, Phase::RequestDrawTile);
    }

    #[test]
    fn apply_reaction_none_pushes_discard_and_advances_turn() {
        let mut state = state_with_turn(Wind::East);
        let (seat, phase) = state.apply_reaction(None, Tile::Character5);
        // Discard goes to East's river
        assert_eq!(state.seats[0].discards[0], Some(Tile::Character5));
        // Turn advances to next player
        assert_eq!(seat, Wind::South);
        assert_eq!(phase, Phase::RequestDrawTile);
    }

    // ── State::reaction_options_by_seat ──

    #[test]
    fn reaction_options_by_seat_only_next_player_can_chow() {
        let mut state = state_with_turn(Wind::East);
        state.seats[1].hand = hand_with(&[Tile::Character1, Tile::Character3]);
        // Discard Character2 — South should have chow options
        let south_options =
            state.reaction_options_by_seat(Wind::South, Tile::Character2);
        assert!(
            south_options
                .iter()
                .any(|a| matches!(a, PlayerAction::Chow { .. }))
        );

        // West (seat 2) should NOT have chow options
        let west_options =
            state.reaction_options_by_seat(Wind::West, Tile::Character2);
        assert!(
            !west_options
                .iter()
                .any(|a| matches!(a, PlayerAction::Chow { .. }))
        );
    }

    #[test]
    fn reaction_options_by_seat_includes_skip_when_options_exist() {
        let mut state = state_with_turn(Wind::East);
        state.seats[1].hand = hand_with(&[Tile::Dot3, Tile::Dot3]);
        let options = state.reaction_options_by_seat(Wind::South, Tile::Dot3);
        assert!(options.contains(&PlayerAction::Skip { seat: Wind::South }));
    }

    // ── State::reaction_options ──

    #[test]
    fn reaction_options_excludes_current_turn() {
        let mut state = state_with_turn(Wind::East);
        // Give pong capability to all other seats
        for i in 1..4 {
            state.seats[i].hand = hand_with(&[Tile::Bamboo5, Tile::Bamboo5]);
        }
        let options = state.reaction_options(Tile::Bamboo5);
        // No reaction from East (current turn)
        assert!(!options.iter().any(|a| a.seat() == Wind::East));
        // All other seats have pong
        for w in [Wind::South, Wind::West, Wind::North] {
            assert!(options.iter().any(|a| a.seat() == w));
        }
    }

    // ── State::resolve_reactions ──

    #[test]
    fn resolve_reactions_hu_wins_over_lower_priority() {
        let state = state_with_turn(Wind::East);
        let choices = vec![
            PlayerAction::Pong {
                seat: Wind::South,
                from: Wind::East,
                tile: Tile::Character1,
            },
            PlayerAction::Hu {
                seat: Wind::West,
                from: Wind::East,
                tile: Tile::Character1,
            },
            PlayerAction::Chow {
                seat: Wind::North,
                tile: Tile::Character1,
                from: Wind::East,
                start: Tile::Character1,
            },
        ];
        assert_eq!(
            state.resolve_reactions(&choices),
            Some(PlayerAction::Hu {
                seat: Wind::West,
                from: Wind::East,
                tile: Tile::Character1,
            })
        );
    }

    #[test]
    fn resolve_reactions_kong_beats_pong_and_chow() {
        let state = state_with_turn(Wind::East);
        let choices = vec![
            PlayerAction::Chow {
                seat: Wind::South,
                tile: Tile::Character1,
                from: Wind::East,
                start: Tile::Character1,
            },
            PlayerAction::Kong {
                seat: Wind::West,
                from: Wind::East,
                tile: Tile::Character1,
            },
            PlayerAction::Pong {
                seat: Wind::North,
                from: Wind::East,
                tile: Tile::Character1,
            },
        ];
        assert_eq!(
            state.resolve_reactions(&choices),
            Some(PlayerAction::Kong {
                seat: Wind::West,
                from: Wind::East,
                tile: Tile::Character1,
            })
        );
    }

    #[test]
    fn resolve_reactions_closer_seat_wins_tiebreaker() {
        // With East discarding, South (dist 1) beats West (dist 2) for same priority
        let state = state_with_turn(Wind::East);
        let choices = vec![
            PlayerAction::Pong {
                seat: Wind::West,
                from: Wind::East,
                tile: Tile::Character1,
            },
            PlayerAction::Pong {
                seat: Wind::South,
                from: Wind::East,
                tile: Tile::Character1,
            },
        ];
        assert_eq!(
            state.resolve_reactions(&choices),
            Some(PlayerAction::Pong {
                seat: Wind::South,
                from: Wind::East,
                tile: Tile::Character1,
            })
        );
    }

    #[test]
    fn resolve_reactions_all_skip_returns_none() {
        let state = state_with_turn(Wind::East);
        let choices = vec![
            PlayerAction::Skip { seat: Wind::South },
            PlayerAction::Skip { seat: Wind::West },
            PlayerAction::Skip { seat: Wind::North },
        ];
        assert_eq!(state.resolve_reactions(&choices), None);
    }

    #[test]
    fn resolve_reactions_empty_returns_none() {
        let state = state_with_turn(Wind::East);
        assert_eq!(state.resolve_reactions(&[]), None);
    }
}
