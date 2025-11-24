use std::collections::HashMap;

use crate::round::Wind;
use crate::tile::Tile;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct PlayerDraw {
    pub seat: Wind,
    pub tile: Tile,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct PlayerDiscard {
    pub seat: Wind,
    pub tile: Tile,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SelfAction {
    ConcealedKong { seat: Wind, tile: Tile },
    AddedKong { seat: Wind, tile: Tile },
    HuSelf { seat: Wind, tile: Tile },
    Skip,
}

impl SelfAction {
    pub fn describe(&self) -> String {
        match self {
            SelfAction::ConcealedKong { seat: _, tile } => {
                format!("Concealed Kong {:?}", tile)
            }
            SelfAction::AddedKong { seat: _, tile } => {
                format!("Added Kong {:?}", tile)
            }
            SelfAction::HuSelf { seat: _, tile } => {
                format!("Hu (self-draw) {:?}", tile)
            }
            SelfAction::Skip => "Skip".to_string(),
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Reaction {
    Skip {
        seat: Wind,
    },
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
}

impl Reaction {
    #[inline]
    pub fn seat(&self) -> Wind {
        match self {
            Reaction::Chow { seat, .. } => *seat,
            Reaction::Pong { seat, .. } => *seat,
            Reaction::Kong { seat, .. } => *seat,
            Reaction::Hu { seat, .. } => *seat,
            Reaction::Skip { seat } => *seat,
        }
    }

    pub fn as_int(&self) -> u8 {
        match self {
            Reaction::Skip { .. } => 0,
            Reaction::Chow { .. } => 1,
            Reaction::Pong { .. } => 2,
            Reaction::Kong { .. } => 3,
            Reaction::Hu { .. } => 4,
        }
    }
}

#[derive(Debug, Clone)]
pub enum PlayerAction {
    Draw(Vec<PlayerDraw>),
    Discard(PlayerDiscard),
    Active(SelfAction),
    Reaction(Reaction),
}

impl From<Vec<PlayerDraw>> for PlayerAction {
    fn from(actions: Vec<PlayerDraw>) -> Self {
        PlayerAction::Draw(actions)
    }
}

impl From<PlayerDiscard> for PlayerAction {
    fn from(action: PlayerDiscard) -> Self {
        PlayerAction::Discard(action)
    }
}

impl From<SelfAction> for PlayerAction {
    fn from(action: SelfAction) -> Self {
        PlayerAction::Active(action)
    }
}

impl From<Reaction> for PlayerAction {
    fn from(action: Reaction) -> Self {
        PlayerAction::Reaction(action)
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ProceedToNextTurn {
    pub seat: Wind,
}

pub type ReactionRequests = HashMap<Wind, Vec<Reaction>>;

#[derive(Debug, Clone)]
pub enum EngineEvent {
    RoundStarted { round_wind: Wind },
    WallShuffled,
    InitialHandsDealt(Vec<PlayerDraw>),

    PlayerAction(PlayerAction),

    RequestAction(Vec<SelfAction>),
    RequestDiscard(Vec<PlayerDiscard>),
    RequestReaction(ReactionRequests),
    ProceedToNextTurn(ProceedToNextTurn),

    RoundEnded { winner: Option<Wind> },
    EngineExited,
}

impl From<PlayerAction> for EngineEvent {
    fn from(action: PlayerAction) -> Self {
        EngineEvent::PlayerAction(action)
    }
}

impl From<ProceedToNextTurn> for EngineEvent {
    fn from(action: ProceedToNextTurn) -> Self {
        EngineEvent::ProceedToNextTurn(action)
    }
}

pub struct LoggedEvent {
    pub index: usize,
    pub time: u64,
    pub event: EngineEvent,
}
