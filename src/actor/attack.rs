use crate::prelude::*;
use bevy::prelude::*;
use rand::Rng;
use serde::{Deserialize, Serialize};
use std::num::NonZero;
use std::ops::Range;

#[derive(Component, Clone, Reflect, Serialize, Deserialize)]
#[reflect(Component, Clone, Serialize, Deserialize)]
pub struct Attack {
    /// The range of damage they can do.
    damage: Range<u32>,
    /// Determines the order of turns in combat. Higher numbers means they will go sooner.
    speed: NonZero<u32>,
    /// The chance the actor has to hit when they attack.
    /// Should be between 0.0 and 1.0
    hit_chance: f32,
}

impl Attack {
    pub fn new(damage: Range<u32>, speed: NonZero<u32>, hit_chance: f32) -> Self {
        Self {
            damage,
            speed,
            hit_chance,
        }
    }

    /// Simulates an attack using the rng and returns the
    /// amount of damage done, or if the attack missed.
    pub fn conduct(&self, rng: &mut impl Rng) -> AttackDamage {
        rng.random_bool(self.hit_chance as f64)
            .then(|| rng.random_range(self.damage.clone()))
            .and_then(|d| NonZero::<u32>::new(d))
            .map(|d| AttackDamage::Hit(d))
            .unwrap_or(AttackDamage::Miss)
    }
}

/// The damage done by an attack. An attack that does 0 damage is considered a miss.
#[derive(Debug, Hash, PartialEq, Eq, Clone, Copy, Serialize, Deserialize)]
pub enum AttackDamage {
    Hit(NonZero<u32>),
    Miss,
}

/// The chance the actor has to block in combat.
/// Should be between 0.0 and 1.0
#[derive(Component, Deref, DerefMut, Clone, Copy, Reflect, Serialize, Deserialize)]
#[reflect(Component, Clone, Serialize, Deserialize)]
#[repr(transparent)]
pub struct BlockChance(pub f32);
