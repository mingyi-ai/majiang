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
    Skip { seat: Wind },
    ConcealedKong { seat: Wind, tile: Tile },
    AddedKong { seat: Wind, tile: Tile },
    Chow { seat: Wind, tile: Tile, from: Wind, start: Tile },
    Pong { seat: Wind, from: Wind, tile: Tile },
    Kong { seat: Wind, from: Wind, tile: Tile },
    Hu { seat: Wind, from: Wind, tile: Tile },
    Discard { seat: Wind, tile: Tile },
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
    pub(crate) fn self_action_options(&self, drawn_tile: Tile) -> Vec<PlayerAction> {
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

        // TODO: self hu check — needs hu solver integration
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
            Some(PlayerAction::Hu { seat, .. }) => (seat, Phase::RequestDrawTile),
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
    fn reaction_options_by_seat(&self, seat: Wind, tile: Tile) -> Vec<PlayerAction> {
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
    pub(crate) fn resolve_reactions(&self, choices: &[PlayerAction]) -> Option<PlayerAction> {
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
