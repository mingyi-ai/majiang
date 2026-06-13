use std::collections::HashMap;

use crate::structs::{Hand, Tile};
use rand::seq::SliceRandom;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
#[repr(u8)]
pub enum Wind {
    East,
    South,
    West,
    North,
}

impl Wind {
    pub fn next(self) -> Self {
        match self {
            Wind::East => Wind::South,
            Wind::South => Wind::West,
            Wind::West => Wind::North,
            Wind::North => Wind::East,
        }
    }

    pub fn iter() -> impl Iterator<Item = Wind> {
        [Wind::East, Wind::South, Wind::West, Wind::North].into_iter()
    }
}

pub(crate) const WALL_SIZE: usize = 144;

#[derive(Clone, Copy)]
pub struct Wall {
    tiles: [Tile; WALL_SIZE],
    pointer: usize,
}

impl Wall {
    #[inline]
    pub fn new_mcr() -> Self {
        let mut tiles: [Tile; WALL_SIZE] = [Tile::Character1; WALL_SIZE];
        let mut idx = 0;
        for tile in Tile::iter() {
            let count = if tile.is_flower() { 1 } else { 4 };
            for _ in 0..count {
                tiles[idx] = tile;
                idx += 1;
            }
        }
        // No shuffling for now; deterministic wall
        Self { tiles, pointer: 0 }
    }

    pub(crate) fn shuffle(&mut self) {
        let mut rng = rand::rng();
        self.tiles.shuffle(&mut rng);
    }

    /// Yields the next tile from the wall, if available.
    /// Advances the wall pointer.
    pub(crate) fn yield_tile(&mut self) -> Option<Tile> {
        if self.pointer >= WALL_SIZE {
            return None;
        }
        let tile = self.tiles[self.pointer];
        self.pointer += 1;
        Some(tile)
    }

    pub(crate) fn get_last_drawn_tile(&self) -> Option<Tile> {
        if self.pointer == 0 {
            return None;
        }
        Some(self.tiles[self.pointer - 1])
    }
}

pub struct SeatState {
    pub hand: Hand,
    pub discards: [Option<Tile>; 32],
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Phase {
    RequestDrawTile,
    RequestSelfAction,
    RequestDiscard,
    RequestReaction(Tile),
    RoundEnded,
}

pub struct RoundState {
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

    // Self-actions (player chooses after drawing)
    ConcealedKong {
        seat: Wind,
        tile: Tile,
    },
    AddedKong {
        seat: Wind,
        tile: Tile,
    },
    SelfHu {
        seat: Wind,
        tile: Tile,
    },
    SelfSkip,

    // Reactions (other players respond to a discard)
    Chow {
        seat: Wind,
        tile: Tile,
        from: Wind,
        chow: [Tile; 2],
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
    ReactionSkip {
        seat: Wind,
    },

    // Discard
    Discard {
        seat: Wind,
        tile: Tile,
    },
}

impl RoundState {
    /// Draw the next tile from the wall and add to the current turn's hand.
    /// If flower: increments flower count, phase stays `RequestDrawTile`.
    /// If non-flower: adds to concealed hand, phase becomes `RequestSelfAction`
    ///   or `RequestDiscard` depending on whether self-actions exist.
    pub(crate) fn draw_tile(&mut self) -> Tile {
        unimplemented!()
    }

    /// Apply a self-action for the current turn player.
    /// Transitions phase:
    ///   `ConcealedKong` / `AddedKong` → `RequestDrawTile`
    ///   `SelfHu` → `RoundEnded`
    ///   `SelfSkip` → `RequestDiscard`
    pub(crate) fn apply_self_action(&mut self, event: Event) {
        unimplemented!()
    }

    /// Remove a tile from the current turn's hand and add to their discard river.
    /// Does NOT advance turn or check reactions (Engine handles that).
    pub(crate) fn add_discard(&mut self, tile: Tile) {
        unimplemented!()
    }

    /// Apply the winning reaction. Sets turn to the winner.
    /// Transitions phase:
    ///   `Chow` / `Pong` → `RequestDiscard`
    ///   `Kong` → `RequestDrawTile`
    ///   `Hu` → `RoundEnded`
    pub(crate) fn apply_reaction(&mut self, event: Event) {
        unimplemented!()
    }

    /// Advance the turn to the next player in order.
    pub(crate) fn advance_turn(&mut self) {
        self.turn = self.turn.next();
    }

    /// Compute possible reactions from all other seats for a given discard tile.
    pub(crate) fn possible_reactions(
        &self,
        tile: Tile,
    ) -> HashMap<Wind, Vec<Event>> {
        unimplemented!()
    }

    /// Resolve a set of reaction choices to find the winning one.
    /// Returns `None` if all players chose to skip.
    pub(crate) fn resolve_reactions(
        &self,
        choices: &HashMap<Wind, Event>,
    ) -> Option<Event> {
        unimplemented!()
    }

    /// Legitimate events for the current phase.
    pub(crate) fn legit_events(&self) -> Vec<Event> {
        match self.phase {
            Phase::RequestDrawTile => {
                // Tile value is unknown until drawn; query() handles this phase directly
                vec![]
            }
            Phase::RequestSelfAction => {
                unimplemented!()
            }
            Phase::RequestDiscard => {
                unimplemented!()
            }
            Phase::RequestReaction(_) => {
                // query() handles this phase directly via possible_reactions(tile)
                vec![]
            }
            Phase::RoundEnded => {
                vec![]
            }
        }
    }
}

// impl RoundState {
//     pub fn new(round_wind: Wind) -> Self {
//         let wall: Wall = Wall::new_mcr();

//         let seats: Seats = Seats {
//             east: Seat {
//                 hand: Hand::default(),
//                 discards: BitArray::default(),
//             },
//             south: Seat {
//                 hand: Hand::default(),
//                 discards: BitArray::default(),
//             },
//             west: Seat {
//                 hand: Hand::default(),
//                 discards: BitArray::default(),
//             },
//             north: Seat {
//                 hand: Hand::default(),
//                 discards: BitArray::default(),
//             },
//         };

//         Self {
//             round_wind,
//             seats,
//             wall,
//             turn: Wind::East,
//             phase: Phase::RoundStarted,
//             last_drawn_tile_ref: None,
//             discard_buffer: None,
//             selfaction_request_buffer: None,
//             reaction_request_buffer: None,
//             pending_reaction: None,
//             winner: None,
//         }
//     }

//     pub fn draw_tile_to_seat(
//         &mut self,
//         seat: Wind,
//     ) -> Result<Tile, HandError> {
//         let tile = self
//             .wall
//             .yield_tile()
//             .ok_or(HandError::DrawError("Wall is empty"))?;
//         let success = self.seats.get_mut(seat).hand.tiles.add_tile(tile.id());
//         if !success {
//             return Err(HandError::DrawError("Failed to add tile to hand"));
//         }
//         Ok(tile)
//     }

//     pub fn draw_initial_hand(
//         &mut self,
//         seat: Wind,
//     ) -> Result<Vec<Tile>, HandError> {
//         let mut drawn_tiles: Vec<Tile> = Vec::new();
//         let mut non_flower_count = 13;
//         while non_flower_count > 0 {
//             let tile = self.draw_tile_to_seat(seat)?;
//             drawn_tiles.push(tile);
//             if !tile.is_flower() {
//                 non_flower_count -= 1;
//             }
//         }
//         Ok(drawn_tiles)
//     }

//     pub fn deal_initial_hands(
//         &mut self,
//     ) -> Result<Vec<PlayerDraw>, HandError> {
//         let mut dealt_tiles: Vec<PlayerDraw> = Vec::new();
//         for seat_wind in Wind::iter() {
//             let drawn = self.draw_initial_hand(seat_wind)?;
//             for tile in drawn {
//                 dealt_tiles.push(PlayerDraw {
//                     seat: seat_wind,
//                     tile,
//                 });
//             }
//         }

//         self.phase = Phase::ExecuteDraw;

//         Ok(dealt_tiles)
//     }

//     pub fn execute_draw(&mut self) -> Result<Vec<PlayerDraw>, HandError> {
//         let mut drawn: Vec<Tile> = vec![];
//         loop {
//             let tile = self.draw_tile_to_seat(self.turn)?;
//             drawn.push(tile);
//             if !tile.is_flower() {
//                 break;
//             }
//         }

//         if drawn.is_empty() {
//             return Err(HandError::DrawError("empty draw"));
//         }
//         let (last, preceding) = drawn.split_last().unwrap();
//         // unwrap is safe: drawn is non-empty, preceding is allowed to be empty
//         if last.is_flower() {
//             return Err(HandError::DrawError("final drawn tile is a flower"));
//         }
//         if preceding.iter().any(|t| !t.is_flower()) {
//             return Err(HandError::DrawError("non-flower before final tile"));
//         }

//         self.last_drawn_tile_ref = Some(*last);

//         let output = drawn
//             .into_iter()
//             .map(|tile| PlayerDraw {
//                 seat: self.turn,
//                 tile,
//             })
//             .collect::<Vec<PlayerDraw>>();

//         let self_action_options =
//             self.possible_player_self_actions(self.turn)?;

//         if self_action_options.is_empty() {
//             self.selfaction_request_buffer = None;
//             self.phase = Phase::RequestDiscard;
//             return Ok(output);
//         }

//         self.selfaction_request_buffer = Some(self_action_options);
//         self.phase = Phase::RequestSelfAction;

//         Ok(output)
//     }

//     pub fn possible_player_self_actions(
//         &mut self,
//         seat: Wind,
//     ) -> Result<Vec<SelfAction>, HandError> {
//         let mut actions: Vec<SelfAction> = Vec::new();

//         let concealed_kong_mask =
//             self.seats.get(seat).hand.possible_concealed_kong();
//         if concealed_kong_mask != [0; 4] {
//             let tiles = concealed_kong_mask.to_unique_tiles();
//             for tile in tiles {
//                 actions.push(SelfAction::ConcealedKong { seat, tile });
//             }
//         }

//         let added_kong_mask =
//             self.seats.get(seat).hand.possible_kong_from_pong();
//         if added_kong_mask != [0; 4] {
//             let tiles = added_kong_mask.to_unique_tiles();
//             for tile in tiles {
//                 actions.push(SelfAction::AddedKong { seat, tile });
//             }
//         }

//         if self.seats.get(seat).hand.can_hu_self() {
//             let drawn_tile = self.last_drawn_tile_ref.ok_or(
//                 HandError::ActionError("Last drawn tile reference is None"),
//             )?;
//             if drawn_tile.is_flower() {
//                 return Err(HandError::ActionError(
//                     "Last drawn tile reference is a flower",
//                 ));
//             }
//             actions.push(SelfAction::HuSelf {
//                 seat,
//                 tile: drawn_tile,
//             });
//         }

//         if actions.is_empty() {
//             return Ok(actions);
//         }

//         actions.push(SelfAction::Skip);

//         Ok(actions)
//     }

//     pub fn request_self_action(
//         &mut self,
//     ) -> Result<Vec<SelfAction>, HandError> {
//         let actions = self.selfaction_request_buffer.take().ok_or(
//             HandError::ActionError("Self action request buffer is None"),
//         )?;
//         if actions.len() <= 1 {
//             return Err(HandError::ActionError(
//                 "Incorrect self action count, expected > 1",
//             ));
//         }
//         self.phase = Phase::ExecuteSelfAction;
//         Ok(actions)
//     }

//     pub fn execute_self_action(
//         &mut self,
//         action: SelfAction,
//     ) -> Result<SelfAction, HandError> {
//         match action {
//             SelfAction::ConcealedKong { seat, tile } => {
//                 self.seats.get_mut(seat).hand.concealed_kong(tile);
//                 self.phase = Phase::ExecuteDraw;
//             }
//             SelfAction::AddedKong { seat, tile } => {
//                 self.seats.get_mut(seat).hand.added_kong(tile);
//                 self.phase = Phase::ExecuteDraw;
//             }
//             SelfAction::HuSelf { seat, tile: _ } => {
//                 self.winner = Some(seat);
//                 self.phase = Phase::RoundEnded;
//             }
//             SelfAction::Skip => {
//                 self.phase = Phase::RequestDiscard;
//             }
//         }

//         Ok(action)
//     }

//     pub fn execute_pending_reaction(&mut self) -> Result<Reaction, HandError> {
//         // take() will set pending_reaction to None
//         let action = self
//             .pending_reaction
//             .take()
//             .ok_or(HandError::ActionError("No pending reaction"))?;

//         match action {
//             Reaction::Chow {
//                 seat,
//                 tile,
//                 from: _,
//                 chow: tiles,
//             } => {
//                 let start_tile = std::cmp::min(tile, tiles[0]);
//                 self.seats.get_mut(seat).hand.chow(tile, start_tile);
//                 self.phase = Phase::RequestDiscard;
//             }
//             Reaction::Pong {
//                 seat,
//                 from: _,
//                 tile,
//             } => {
//                 self.seats.get_mut(seat).hand.pong(tile);
//                 self.phase = Phase::RequestDiscard;
//             }
//             Reaction::Kong {
//                 seat,
//                 from: _,
//                 tile,
//             } => {
//                 self.seats.get_mut(seat).hand.kong(tile);
//                 self.phase = Phase::ExecuteDraw;
//             }
//             Reaction::Hu {
//                 seat,
//                 from: _,
//                 tile: _,
//             } => {
//                 self.winner = Some(seat);
//                 self.phase = Phase::RoundEnded;
//             }
//             Reaction::Skip { .. } => {
//                 return Err(HandError::ActionError(
//                     "Skip reaction should not be executed",
//                 ));
//             }
//         }

//         Ok(action)
//     }

//     pub fn request_discard(
//         &mut self,
//     ) -> Result<Vec<PlayerDiscard>, HandError> {
//         let mut tiles_array = self.seats.get(self.turn).hand.tiles;

//         let unique_tiles = tiles_array
//             .bit_and(BitArray::NON_FLOWER_MASK)
//             .to_unique_tiles();

//         let actions: Vec<PlayerDiscard> = unique_tiles
//             .into_iter()
//             .map(|tile| PlayerDiscard {
//                 seat: self.turn,
//                 tile,
//             })
//             .collect();

//         self.phase = Phase::ExecuteDiscard;

//         Ok(actions)
//     }

//     pub fn execute_discard(
//         &mut self,
//         action: PlayerDiscard,
//     ) -> Result<PlayerDiscard, HandError> {
//         let seat = self.seats.get_mut(action.seat);

//         let _ = seat.hand.tiles.remove_tile(action.tile.id());

//         self.discard_buffer = Some(action.tile);

//         let reaction_options = self.possible_reactions_all_seats()?;

//         if reaction_options.is_empty() {
//             self.seats.get_mut(self.turn).discards.add_tile(
//                 self.discard_buffer
//                     .take()
//                     .ok_or(HandError::ActionError("No tile to discard"))?
//                     .id(),
//             );
//             self.turn = self.turn.next();
//             self.phase = Phase::ExecuteDraw;
//             return Ok(action);
//         }

//         self.reaction_request_buffer = Some(reaction_options);
//         self.phase = Phase::RequestReaction;

//         Ok(action)
//     }

//     fn possible_reactions_by_seat(
//         &self,
//         seat: Wind,
//         tile: Tile,
//     ) -> Result<Vec<Reaction>, HandError> {
//         let mut actions: Vec<Reaction> = Vec::new();

//         if seat == self.turn {
//             return Err(HandError::ActionError(
//                 "Reaction player cannot be the active player",
//             ));
//         }

//         // below is bug prone: no compiler check that all Reaction variants are covered
//         // consider iterating over Reaction variants instead
//         if seat == self.turn.next() {
//             let chow_masks = self.seats.get(seat).hand.possible_chows(tile);
//             let chow_starting_tiles = chow_masks.to_unique_tiles();

//             for start_tile in chow_starting_tiles {
//                 let id = start_tile.id() as usize;
//                 let row = id >> 6;
//                 let shift = id & 0x3F;

//                 // We know these exist because possible_chows returned them
//                 let t0 = start_tile;
//                 let t1 = Tile::from_indices(row, shift + 4).unwrap();
//                 let t2 = Tile::from_indices(row, shift + 8).unwrap();

//                 let chow_tiles = if tile == t0 {
//                     [t1, t2]
//                 } else if tile == t1 {
//                     [t0, t2]
//                 } else {
//                     [t0, t1]
//                 };

//                 actions.push(Reaction::Chow {
//                     seat,
//                     tile,
//                     from: self.turn,
//                     chow: chow_tiles,
//                 });
//             }
//         }

//         if self.seats.get(seat).hand.can_pong(tile) {
//             actions.push(Reaction::Pong {
//                 seat,
//                 from: self.turn,
//                 tile,
//             });
//         }

//         if self.seats.get(seat).hand.can_kong(tile) {
//             actions.push(Reaction::Kong {
//                 seat,
//                 from: self.turn,
//                 tile,
//             });
//         }

//         if self.seats.get(seat).hand.can_hu(tile) {
//             actions.push(Reaction::Hu {
//                 seat,
//                 from: self.turn,
//                 tile,
//             });
//         }

//         if actions.is_empty() {
//             return Ok(actions);
//         }

//         actions.push(Reaction::Skip { seat });

//         Ok(actions)
//     }

//     fn possible_reactions_all_seats(
//         &self,
//     ) -> Result<ReactionRequests, HandError> {
//         let mut output: ReactionRequests = HashMap::new();

//         let tile = self
//             .discard_buffer
//             .ok_or(HandError::ActionError("No tile to react to"))?;

//         for seat in Wind::iter() {
//             if seat == self.turn {
//                 continue;
//             }

//             let seat_actions = self.possible_reactions_by_seat(seat, tile)?;

//             if seat_actions.is_empty() {
//                 continue;
//             }

//             output.insert(seat, seat_actions);
//         }

//         Ok(output)
//     }

//     pub fn request_reaction(&mut self) -> Result<ReactionRequests, HandError> {
//         let actions = self.reaction_request_buffer.clone().ok_or(
//             HandError::ActionError("Reaction request buffer is None"),
//         )?;
//         if actions.is_empty() {
//             return Err(HandError::ActionError(
//                 "No reactions available, but request_reaction was called",
//             ));
//         }
//         for (_, v) in actions.iter() {
//             if v.len() <= 1 {
//                 return Err(HandError::ActionError(
//                     "Incorrect reaction action count for a seat, expected > 1",
//                 ));
//             }
//         }

//         self.phase = Phase::HandleReactions;

//         Ok(actions)
//     }

//     pub fn handle_reactions(
//         &mut self,
//         reactions: Vec<Reaction>,
//     ) -> Result<ProceedToNextTurn, HandError> {
//         let seen_seats: HashSet<Wind> =
//             HashSet::from_iter(reactions.iter().map(|r| r.seat()));

//         if seen_seats.len() != reactions.len() {
//             return Err(HandError::ActionError(
//                 "Reactions contain duplicate seats",
//             ));
//         }

//         let expected_seats = self
//             .reaction_request_buffer
//             .as_ref()
//             .ok_or(HandError::ActionError("Reaction request buffer is None"))?
//             .keys()
//             .cloned()
//             .collect::<HashSet<Wind>>();

//         if expected_seats != seen_seats {
//             return Err(HandError::ActionError(
//                 "Reactions do not match expected seats",
//             ));
//         }

//         self.reaction_request_buffer = None;
//         // Potential flaws: caller can construct invalid Reaction variants
//         // that do not correspond to the possible reactions previously requested.
//         // Consider adding validation here.
//         // But why bother? This is not a public API.

//         let len = reactions.len();

//         let mut weights: Vec<(u8, u8)> = vec![(0, 0); len];
//         for (i, reaction) in reactions.iter().enumerate() {
//             let seat_order = (reaction.seat() as u8 + 4 - self.turn as u8) % 4;
//             weights[i] = (reaction.as_int(), seat_order);
//         }

//         let idx = weights
//             .iter()
//             .enumerate()
//             .max_by_key(|(_, (x, y))| (*x, std::cmp::Reverse(*y)))
//             .map(|(i, _)| i)
//             .ok_or(HandError::ActionError("Reactions may be empty"))?;

//         let to_do = match reactions[idx] {
//             Reaction::Skip { .. } => None,
//             reaction => Some(reaction),
//         };

//         if let Some(seat) = to_do.as_ref().map(Reaction::seat) {
//             self.turn = seat;
//             self.pending_reaction = to_do;
//             self.discard_buffer = None;
//             self.phase = Phase::ExecuteReaction;
//         } else {
//             self.pending_reaction = None;
//             self.seats.get_mut(self.turn).discards.add_tile(
//                 self.discard_buffer
//                     .take()
//                     .ok_or(HandError::ActionError("No tile to discard"))?
//                     .id(),
//             );
//             self.turn = self.turn.next();
//             self.phase = Phase::ExecuteDraw;
//         }
//         Ok(ProceedToNextTurn { seat: self.turn })
//     }
// }

// #[cfg(test)]
// mod tests {
//     use super::*;
//     use crate::bits::MahjongBitArray;
//     use crate::tile::Tile::*;

//     #[test]
//     fn test_possible_reactions_by_seat() {
//         let mut round = RoundState::new(Wind::East);
//         // set up hands for testing
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
