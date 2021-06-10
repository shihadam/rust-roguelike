//! Components that make up entities.

pub use crate::prelude::*;

/// Component for entities that should be rendered.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Renderable {
    pub color: ColorPair,
    pub glyph: FontCharType,
}

/// Player tag.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Player;

/// Enemy tag.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Enemy;

/// Entity stats used for combat.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Stats {
    pub max_health: i32,
    pub health: i32,
    pub damage: i32,
}

/// Tag for random movement system.
#[derive(Copy, Clone, Debug, PartialEq)]
pub struct RandomMovement;

/// Tag for player-targeted movement system.
#[derive(Copy, Clone, Debug, PartialEq)]
pub struct TargetedMovement;

/// Message for entities who want to move.
#[derive(Copy, Clone, Debug, PartialEq)]
pub struct WantsToMove {
    pub entity: Entity,
    pub destination: Point,
}

/// Message for entities who want to attack.
#[derive(Copy, Clone, Debug, PartialEq)]
pub struct WantsToAttack {
    pub entity: Entity,
    pub target: Entity,
}

/// Message for entities who want to mine.
#[derive(Copy, Clone, Debug, PartialEq)]
pub struct WantsToMine {
    pub entity: Entity,
    pub target: Point,
}

/// Name storage.
#[derive(Clone, Debug, PartialEq)]
pub struct Name(pub String);

/// Mining range.
#[derive(Copy, Clone, Debug, PartialEq)]
pub struct MineRange(pub i32);
