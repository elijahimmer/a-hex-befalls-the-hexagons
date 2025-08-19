use crate::prelude::*;
use bevy::prelude::*;
use rand::Rng;
use serde::{Deserialize, Serialize};
use std::num::NonZero;
use std::ops::Range;

#[derive(Component, Clone, Serialize, Deserialize)]
pub struct Attack {
    /// The range of damage they can do.
    pub(super) damage: Range<u32>,
    /// The chance the actor has to hit when they attack.
    /// Should be between 0.0 and 1.0
    pub(super) hit_chance: f32,
}

impl Attack {
    pub fn new(damage: Range<u32>, hit_chance: f32) -> Self {
        Self { damage, hit_chance }
    }

    pub fn from_name(name: ActorName) -> Self {
        use ActorName as A;
        let (damage, hit_chance) = match name {
            A::Warrior => (35..61, 0.8),
            A::Priestess => (25..46, 0.7),
            A::Theif => (20..41, 0.8),
            A::Ogre => (3000..6100, 0.6),
            A::Goblin => (1500..3100, 0.8),
            A::Skeleton => (3000..5100, 0.8),
            A::UnknownJim => (0..1, 0.0),
        };

        Self::new(damage, hit_chance)
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

/// The chance the actor has to block an attack in combat.
/// Should be between 0.0 and 1.0
#[derive(Component, Deref, DerefMut, Clone, Copy, Serialize, Deserialize)]
#[repr(transparent)]
pub struct BlockChance(pub f32);

/// Determines the order of turns in combat. Higher numbers means they will go sooner.
#[derive(Component, Deref, DerefMut, Clone, Copy, Serialize, Deserialize)]
pub struct AttackSpeed(pub u32);

impl AttackSpeed {
    pub fn new(speed: u32) -> Self {
        Self(speed)
    }

    pub fn from_name(name: ActorName) -> Self {
        use ActorName as A;
        Self(match name {
            A::Warrior => 4,
            A::Priestess => 5,
            A::Theif => 6,
            A::Ogre => 2,
            A::Goblin => 5,
            A::Skeleton => 3,
            A::UnknownJim => 1,
        })
    }
}

impl BlockChance {
    pub fn from_name(name: ActorName) -> Self {
        use ActorName as A;
        Self(match name {
            A::Warrior => 0.5,
            A::Priestess => 0.1,
            A::Theif => 0.3,
            A::Ogre => 0.2,
            A::Goblin => 0.4,
            A::Skeleton => 0.2,
            A::UnknownJim => 0.1,
        })
    }
}
