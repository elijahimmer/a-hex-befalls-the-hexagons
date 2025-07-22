use bevy::prelude::*;
use rand::Rng;
use serde::{Deserialize, Serialize};
use std::num::NonZero;
use std::ops::{DerefMut, Range};

/// The typical components for any given
/// actor.
#[derive(Bundle)]
pub struct Actor {
    pub name: Name,
    pub team: Team,
    pub health: Health,
    pub attack: Attack,
    pub attack_speed: AttackSpeed,
    pub hit_chance: HitChance,
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

/// The health of an actor.
/// This also determines whether that
/// actor is alive or not.
#[derive(Component, Clone, Copy, Reflect, Serialize, Deserialize)]
#[reflect(Component, Clone, Serialize, Deserialize)]
pub struct Health {
    /// When None, the actor is dead.
    /// This should never be above the `max`
    pub current: Option<NonZero<u32>>,
    pub max: NonZero<u32>,
}

#[expect(dead_code)]
impl Health {
    #[inline]
    pub fn is_alive(&self) -> bool {
        self.current.is_some()
    }

    /// Heals the actor if they are not already dead
    #[inline]
    pub fn heal(&mut self, amount: u32) {
        let Some(amount) = NonZero::<u32>::new(amount) else {
            return;
        };

        if let Some(ref mut curr) = self.current {
            *curr = curr.saturating_add(amount.get()).min(self.max)
        }
    }

    /// Heals the actor or revives them if they are dead.
    /// Only revives actors if `amount` > 0
    #[inline]
    pub fn heal_or_revive(&mut self, amount: u32) {
        let Some(amount) = NonZero::<u32>::new(amount) else {
            return;
        };

        match self.current {
            Some(ref mut curr) => {
                *curr = curr.saturating_add(amount.get()).min(self.max);
            }
            Option::None => {
                self.current = Some(amount.min(self.max));
            }
        }
    }

    /// Damage the actor, killing them if they health would go below one.
    #[inline]
    pub fn damage(&mut self, amount: u32) {
        let (Some(curr), Some(amount)) = (self.current, NonZero::<u32>::new(amount)) else {
            return;
        };

        self.current = NonZero::new(curr.get().saturating_sub(amount.get()));
    }

    /// Damage the actor yet don't kill them
    #[inline]
    pub fn damage_no_kill(&mut self, amount: u32) {
        let (Some(curr), Some(amount)) = (self.current, NonZero::<u32>::new(amount)) else {
            return;
        };

        self.current = Some(
            NonZero::new(curr.get().saturating_sub(amount.get()))
                .unwrap_or(NonZero::new(1u32).unwrap()),
        );
    }

    /// Damage the actor but only kill them if they were already at 1 health.
    #[inline]
    pub fn damage_endurence(&mut self, amount: u32) {
        let (Some(curr), Some(amount)) = (self.current, NonZero::<u32>::new(amount)) else {
            return;
        };

        self.current = (curr.get() > 1).then(|| {
            NonZero::new(curr.get().saturating_sub(amount.get()))
                .unwrap_or(NonZero::new(1u32).unwrap())
        });
    }
}

/// The range of damage they can do.
#[derive(Component, Deref, DerefMut, Clone, Reflect, Serialize, Deserialize)]
#[reflect(Component, Clone, Serialize, Deserialize)]
pub struct Attack(pub Range<u16>);

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

/// The chance the actor has to heal at the end
/// of the round in combat
/// Should be between 0.0 and 1.0
#[derive(Component, Deref, DerefMut, Clone, Copy, Reflect, Serialize, Deserialize)]
#[reflect(Component, Clone, Serialize, Deserialize)]
#[repr(transparent)]
pub struct HealChance(pub f32);

/// Heals all actors that end of round
/// based on their [`HealChance`]
#[expect(dead_code)]
pub fn end_of_turn_healing<Rand: Resource + DerefMut<Target: Rng>>(
    mut actor_q: Query<(&HealChance, &mut Health)>,
    mut rng: ResMut<Rand>,
) {
    actor_q
        .iter_mut()
        .filter_map(|(chance, health)| rng.random_bool(**chance as f64).then_some(health))
        .map(|health| (health.max.get().div_ceil(10), health))
        .for_each(|(additional, mut health)| health.heal(additional))
}

///// Gives all dead entities non-player entities [`DeathDespawnTimer`]s if they don't have them already.
//pub fn kill_all_dead(
//    mut commands: Commands,
//    actor_q: Query<(Entity, &Team, &Health), (Without<DeathDespawnTimer>, Changed<Health>)>,
//) {
//    for (entity, team, health) in actor_q.iter() {
//        if *team == Team::Player || health.is_alive() {
//            continue;
//        }
//
//        commands.entity(entity).insert(DeathDespawnTimer::default());
//    }
//}

///// Ticks up all [`DeathDespawnTimer`]s and despawns all who have been dead long enough.
/////
///// TODO: Implement a [`OnInsert`]<[`DeathDespawnTimer`]> for all actors
/////       to change their sprite to a dead version.
/////
///// TODO: Implement a [`OnDespawn`] for actors to play a little animation
/////       the moment they despawn
//pub fn tick_dead_despawn_timers(
//    mut commands: Commands,
//    time: Time,
//    mut timer_q: Query<(Entity, &mut DeathDespawnTimer)>,
//) {
//    for (entity, mut timer) in timer_q.iter_mut() {
//        **timer += 1;
//
//        if **timer > DEATH_ANIMATION_TIME {
//            commands.entity(entity).despawn();
//        }
//    }
//}
