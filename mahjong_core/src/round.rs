use std::collections::{HashMap, HashSet};

use crate::hand::{Hand, HandError};
use crate::tile::{Tile, Wall};

use crate::event::{
    PlayerDiscard, PlayerDraw, ProceedToNextTurn, Reaction, ReactionRequests,
    SelfAction,
};

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

#[derive(Debug, Clone)]
pub struct Seat {
    pub hand: Hand,
    pub discards: Vec<Tile>,
}

#[derive(Debug, Clone)]
pub struct Seats {
    pub east: Seat,
    pub south: Seat,
    pub west: Seat,
    pub north: Seat,
}

impl Seats {
    pub fn get(&self, wind: Wind) -> &Seat {
        match wind {
            Wind::East => &self.east,
            Wind::South => &self.south,
            Wind::West => &self.west,
            Wind::North => &self.north,
        }
    }
    pub fn get_mut(&mut self, wind: Wind) -> &mut Seat {
        match wind {
            Wind::East => &mut self.east,
            Wind::South => &mut self.south,
            Wind::West => &mut self.west,
            Wind::North => &mut self.north,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Phase {
    RoundStarted,
    ExecuteDraw,
    RequestSelfAction,
    ExecuteSelfAction,
    ExecuteReaction,
    ExecuteKongRepl,
    RequestDiscard,
    ExecuteDiscard,
    RequestReaction,
    HandleReactions,
    RoundEnded,
}

#[derive(Debug, Clone)]
pub struct RoundState {
    pub round_wind: Wind,
    pub seats: Seats,
    pub wall: Wall,
    pub turn: Wind,
    pub phase: Phase,
    selfaction_request_buffer: Option<Vec<SelfAction>>,
    last_drawn_tile_ref: Option<Tile>,
    pub discard_buffer: Option<Tile>,
    reaction_request_buffer: Option<ReactionRequests>,
    pending_reaction: Option<Reaction>,
    pub winner: Option<Wind>,
}

impl RoundState {
    pub fn new(round_wind: Wind) -> Self {
        let wall: Wall = Wall::new_mcr();

        let seats: Seats = Seats {
            east: Seat {
                hand: Hand::default(),
                discards: Vec::new(),
            },
            south: Seat {
                hand: Hand::default(),
                discards: Vec::new(),
            },
            west: Seat {
                hand: Hand::default(),
                discards: Vec::new(),
            },
            north: Seat {
                hand: Hand::default(),
                discards: Vec::new(),
            },
        };

        Self {
            round_wind,
            seats,
            wall,
            turn: Wind::East,
            phase: Phase::RoundStarted,
            last_drawn_tile_ref: None,
            discard_buffer: None,
            selfaction_request_buffer: None,
            reaction_request_buffer: None,
            pending_reaction: None,
            winner: None,
        }
    }

    pub fn deal_initial_hands(
        &mut self,
    ) -> Result<Vec<PlayerDraw>, HandError> {
        let mut dealt_tiles: Vec<PlayerDraw> = Vec::new();
        for seat_wind in Wind::iter() {
            let seat = self.seats.get_mut(seat_wind);
            let drawn = seat.hand.draw_initial_hand(&mut self.wall)?;
            for tile in drawn {
                dealt_tiles.push(PlayerDraw {
                    seat: seat_wind,
                    tile,
                });
            }
        }

        self.phase = Phase::ExecuteDraw;

        Ok(dealt_tiles)
    }

    pub fn execute_draw(&mut self) -> Result<Vec<PlayerDraw>, HandError> {
        let seat = &mut self.seats.get_mut(self.turn);
        let wall = &mut self.wall;

        let drawn: Vec<Tile> = seat.hand.draw(wall)?;

        if drawn.is_empty() {
            return Err(HandError::DrawError("empty draw"));
        }
        let (last, preceding) = drawn.split_last().unwrap();
        // unwrap is safe: drawn is non-empty, preceding is allowed to be empty
        if last.is_flower() {
            return Err(HandError::DrawError("final drawn tile is a flower"));
        }
        if preceding.iter().any(|t| !t.is_flower()) {
            return Err(HandError::DrawError("non-flower before final tile"));
        }

        self.last_drawn_tile_ref = Some(*last);

        let output = drawn
            .into_iter()
            .map(|tile| PlayerDraw {
                seat: self.turn,
                tile,
            })
            .collect::<Vec<PlayerDraw>>();

        let self_action_options =
            self.possible_player_self_actions(self.turn)?;

        if self_action_options.is_empty() {
            self.selfaction_request_buffer = None;
            self.phase = Phase::RequestDiscard;
            return Ok(output);
        }

        self.selfaction_request_buffer = Some(self_action_options);
        self.phase = Phase::RequestSelfAction;

        Ok(output)
    }

    pub fn possible_player_self_actions(
        &mut self,
        seat: Wind,
    ) -> Result<Vec<SelfAction>, HandError> {
        let mut actions: Vec<SelfAction> = Vec::new();

        for tile in self.seats.get(seat).hand.possible_concealed_kong() {
            actions.push(SelfAction::ConcealedKong { seat, tile });
        }

        for tile in self.seats.get(seat).hand.possible_kong_from_pong() {
            actions.push(SelfAction::AddedKong { seat, tile });
        }

        if Hand::can_hu_self(&self.seats.get(seat).hand)? {
            let drawn_tile = self.last_drawn_tile_ref.ok_or(
                HandError::ActionError("Last drawn tile reference is None"),
            )?;
            if drawn_tile.is_flower() {
                return Err(HandError::ActionError(
                    "Last drawn tile reference is a flower",
                ));
            }
            actions.push(SelfAction::HuSelf {
                seat,
                tile: drawn_tile,
            });
        }

        if actions.is_empty() {
            return Ok(actions);
        }

        actions.push(SelfAction::Skip);

        Ok(actions)
    }

    pub fn request_self_action(
        &mut self,
    ) -> Result<Vec<SelfAction>, HandError> {
        let actions = self.selfaction_request_buffer.take().ok_or(
            HandError::ActionError("Self action request buffer is None"),
        )?;
        if actions.len() <= 1 {
            return Err(HandError::ActionError(
                "Incorrect self action count, expected > 1",
            ));
        }
        self.phase = Phase::ExecuteSelfAction;
        Ok(actions)
    }

    pub fn execute_self_action(
        &mut self,
        action: SelfAction,
    ) -> Result<SelfAction, HandError> {
        match action {
            SelfAction::ConcealedKong { seat, tile } => {
                self.seats.get_mut(seat).hand.concealed_kong(tile);
                self.phase = Phase::ExecuteKongRepl;
            }
            SelfAction::AddedKong { seat, tile } => {
                self.seats.get_mut(seat).hand.added_kong(tile);
                self.phase = Phase::ExecuteKongRepl;
            }
            SelfAction::HuSelf { seat, tile: _ } => {
                self.winner = Some(seat);
                self.phase = Phase::RoundEnded;
            }
            SelfAction::Skip => {
                self.phase = Phase::RequestDiscard;
            }
        }

        Ok(action)
    }

    pub fn execute_pending_reaction(&mut self) -> Result<Reaction, HandError> {
        // take() will set pending_reaction to None
        let action = self
            .pending_reaction
            .take()
            .ok_or(HandError::ActionError("No pending reaction"))?;

        match action {
            Reaction::Chow {
                seat,
                tile,
                from: _,
                chow: tiles,
            } => {
                self.seats.get_mut(seat).hand.chow(tile, tiles);
                self.phase = Phase::RequestDiscard;
            }
            Reaction::Pong {
                seat,
                from: _,
                tile,
            } => {
                self.seats.get_mut(seat).hand.pong(tile);
                self.phase = Phase::RequestDiscard;
            }
            Reaction::Kong {
                seat,
                from: _,
                tile,
            } => {
                self.seats.get_mut(seat).hand.kong(tile);
                self.phase = Phase::ExecuteKongRepl;
            }
            Reaction::Hu {
                seat,
                from: _,
                tile: _,
            } => {
                self.winner = Some(seat);
                self.phase = Phase::RoundEnded;
            }
            Reaction::Skip { .. } => {
                return Err(HandError::ActionError(
                    "Skip reaction should not be executed",
                ));
            }
        }

        Ok(action)
    }

    pub fn execute_kong_repl(&mut self) -> Result<Vec<PlayerDraw>, HandError> {
        let seat = self.seats.get_mut(self.turn);
        let wall = &mut self.wall;

        let drawn = seat.hand.draw(wall)?;

        self.phase = Phase::RequestDiscard;

        Ok(drawn
            .into_iter()
            .map(|tile| PlayerDraw {
                seat: self.turn,
                tile,
            })
            .collect())
    }

    pub fn request_discard(
        &mut self,
    ) -> Result<Vec<PlayerDiscard>, HandError> {
        let unique_tiles: HashSet<Tile> = HashSet::from_iter(
            self.seats.get(self.turn).hand.tiles.iter().cloned(),
        );

        let actions: Vec<PlayerDiscard> = unique_tiles
            .into_iter()
            .map(|tile| PlayerDiscard {
                seat: self.turn,
                tile,
            })
            .collect();

        self.phase = Phase::ExecuteDiscard;

        Ok(actions)
    }

    pub fn execute_discard(
        &mut self,
        action: PlayerDiscard,
    ) -> Result<PlayerDiscard, HandError> {
        let seat = self.seats.get_mut(action.seat);

        let _ = seat.hand.discard(action.tile)?;

        self.discard_buffer = Some(action.tile);

        let reaction_options = self.possible_reactions_all_seats()?;

        if reaction_options.is_empty() {
            self.seats.get_mut(self.turn).discards.push(
                self.discard_buffer
                    .take()
                    .ok_or(HandError::ActionError("No tile to discard"))?,
            );
            self.turn = self.turn.next();
            self.phase = Phase::ExecuteDraw;
            return Ok(action);
        }

        self.reaction_request_buffer = Some(reaction_options);
        self.phase = Phase::RequestReaction;

        Ok(action)
    }

    fn possible_reactions_by_seat(
        &self,
        seat: Wind,
        tile: Tile,
    ) -> Result<Vec<Reaction>, HandError> {
        let mut actions: Vec<Reaction> = Vec::new();

        if seat == self.turn {
            return Err(HandError::ActionError(
                "Reaction player cannot be the active player",
            ));
        }

        // below is bug prone: no compiler check that all Reaction variants are covered
        // consider iterating over Reaction variants instead
        if seat == self.turn.next() {
            for tiles in self.seats.get(seat).hand.possible_chows(tile) {
                actions.push(Reaction::Chow {
                    seat,
                    tile,
                    from: self.turn,
                    chow: tiles,
                });
            }
        }

        if self.seats.get(seat).hand.can_pong(tile) {
            actions.push(Reaction::Pong {
                seat,
                from: self.turn,
                tile,
            });
        }

        if self.seats.get(seat).hand.can_kong(tile) {
            actions.push(Reaction::Kong {
                seat,
                from: self.turn,
                tile,
            });
        }

        if self.seats.get(seat).hand.can_hu(tile)? {
            actions.push(Reaction::Hu {
                seat,
                from: self.turn,
                tile,
            });
        }

        if actions.is_empty() {
            return Ok(actions);
        }

        actions.push(Reaction::Skip { seat });

        Ok(actions)
    }

    fn possible_reactions_all_seats(
        &self,
    ) -> Result<ReactionRequests, HandError> {
        let mut output: ReactionRequests = HashMap::new();

        let tile = self
            .discard_buffer
            .ok_or(HandError::ActionError("No tile to react to"))?;

        for seat in Wind::iter() {
            if seat == self.turn {
                continue;
            }

            let seat_actions = self.possible_reactions_by_seat(seat, tile)?;

            if seat_actions.is_empty() {
                continue;
            }

            output.insert(seat, seat_actions);
        }

        Ok(output)
    }

    pub fn request_reaction(&mut self) -> Result<ReactionRequests, HandError> {
        let actions = self.reaction_request_buffer.clone().ok_or(
            HandError::ActionError("Reaction request buffer is None"),
        )?;
        if actions.is_empty() {
            return Err(HandError::ActionError(
                "No reactions available, but request_reaction was called",
            ));
        }
        for (_, v) in actions.iter() {
            if v.len() <= 1 {
                return Err(HandError::ActionError(
                    "Incorrect reaction action count for a seat, expected > 1",
                ));
            }
        }

        self.phase = Phase::HandleReactions;

        Ok(actions)
    }

    pub fn handle_reactions(
        &mut self,
        reactions: Vec<Reaction>,
    ) -> Result<ProceedToNextTurn, HandError> {
        let seen_seats: HashSet<Wind> =
            HashSet::from_iter(reactions.iter().map(|r| r.seat()));

        if seen_seats.len() != reactions.len() {
            return Err(HandError::ActionError(
                "Reactions contain duplicate seats",
            ));
        }

        let expected_seats = self
            .reaction_request_buffer
            .as_ref()
            .ok_or(HandError::ActionError("Reaction request buffer is None"))?
            .keys()
            .cloned()
            .collect::<HashSet<Wind>>();

        if expected_seats != seen_seats {
            return Err(HandError::ActionError(
                "Reactions do not match expected seats",
            ));
        }

        self.reaction_request_buffer = None;
        // Potential flaws: caller can construct invalid Reaction variants
        // that do not correspond to the possible reactions previously requested.
        // Consider adding validation here.
        // But why bother? This is not a public API.

        let len = reactions.len();

        let mut weights: Vec<(u8, u8)> = vec![(0, 0); len];
        for (i, reaction) in reactions.iter().enumerate() {
            let seat_order = (reaction.seat() as u8 + 4 - self.turn as u8) % 4;
            weights[i] = (reaction.as_int(), seat_order);
        }

        let idx = weights
            .iter()
            .enumerate()
            .max_by_key(|(_, (x, y))| (*x, std::cmp::Reverse(*y)))
            .map(|(i, _)| i)
            .ok_or(HandError::ActionError("Reactions may be empty"))?;

        let to_do = match reactions[idx] {
            Reaction::Skip { .. } => None,
            reaction => Some(reaction),
        };

        if let Some(seat) = to_do.as_ref().map(Reaction::seat) {
            self.turn = seat;
            self.pending_reaction = to_do;
            self.discard_buffer = None;
            self.phase = Phase::ExecuteReaction;
        } else {
            self.pending_reaction = None;
            self.seats.get_mut(self.turn).discards.push(
                self.discard_buffer
                    .take()
                    .ok_or(HandError::ActionError("No tile to discard"))?,
            );
            self.turn = self.turn.next();
            self.phase = Phase::ExecuteDraw;
        }
        Ok(ProceedToNextTurn { seat: self.turn })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::tile::Tile::*;
    #[test]
    fn test_possible_reactions_by_seat() {
        let mut round = RoundState::new(Wind::East);
        // set up hands for testing
        round.seats.get_mut(Wind::South).hand.tiles =
            vec![Dot2, Dot3, Dot5, Dot6];
        round.turn = Wind::East;
        let tile = Dot4;
        let reactions_south =
            round.possible_reactions_by_seat(Wind::South, tile).unwrap();

        assert_eq!(reactions_south.len(), 4);
        assert!(reactions_south.contains(&Reaction::Chow {
            seat: Wind::South,
            tile,
            from: Wind::East,
            chow: [Dot2, Dot3]
        }));
        assert!(reactions_south.contains(&Reaction::Chow {
            seat: Wind::South,
            tile,
            from: Wind::East,
            chow: [Dot3, Dot5]
        }));
        assert!(reactions_south.contains(&Reaction::Chow {
            seat: Wind::South,
            tile,
            from: Wind::East,
            chow: [Dot5, Dot6]
        }));
    }
}
