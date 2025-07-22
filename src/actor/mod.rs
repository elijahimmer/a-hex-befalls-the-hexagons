mod health;

pub use health::*;

use crate::animation::AnimationConfig;
use bevy::prelude::*;
use rand::Rng;
use serde::{Deserialize, Serialize};
use std::num::NonZero;
use std::ops::{DerefMut, Range};

/// The typical components for any given actor.
#[derive(Bundle)]
pub struct Actor {
    pub name: Name,
    pub team: Team,
    pub health: HealthBundle,
    pub attack_damage: AttackDamage,
    pub attack_speed: AttackSpeed,
    pub hit_chance: HitChance,
    pub transform: Transform,
    pub sprite: Sprite,
    pub animation: AnimationConfig,
}

/// The team the actor is in for combat.
#[derive(Component, Debug, Hash, PartialEq, Eq, Clone, Copy, Reflect, Serialize, Deserialize)]
#[reflect(Component, Debug, Hash, PartialEq, Clone, Serialize, Deserialize)]
pub enum Team {
    /// The player controls this actor and
    /// decides their moves.
    Player,
    /// The game decides what these actors do
    /// on a given turn.
    Enemy,
}

/// The range of damage they can do.
#[derive(Component, Deref, DerefMut, Clone, Reflect, Serialize, Deserialize)]
#[reflect(Component, Clone, Serialize, Deserialize)]
pub struct AttackDamage(pub Range<u32>);

/// Determines the order of turns in combat.
/// Higher numbers means they will go sooner.
#[derive(Component, Deref, DerefMut, Clone, Copy, Reflect, Serialize, Deserialize)]
#[reflect(Component, Clone, Serialize, Deserialize)]
#[repr(transparent)]
pub struct AttackSpeed(pub NonZero<u32>);

/// The chance the actor has to hit when they attack.
/// Should be between 0.0 and 1.0
#[derive(Component, Deref, DerefMut, Clone, Copy, Reflect, Serialize, Deserialize)]
#[reflect(Component, Clone, Serialize, Deserialize)]
#[repr(transparent)]
pub struct HitChance(pub f32);

/// The chance the actor has to block in combat.
/// Should be between 0.0 and 1.0
#[derive(Component, Deref, DerefMut, Clone, Copy, Reflect, Serialize, Deserialize)]
#[reflect(Component, Clone, Serialize, Deserialize)]
#[repr(transparent)]
pub struct BlockChance(pub f32);

