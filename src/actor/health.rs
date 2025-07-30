use crate::prelude::*;
use bevy::prelude::*;
use rand::Rng;
use serde::{Deserialize, Serialize};
use std::num::NonZero;
use std::ops::DerefMut;

/// Triggered on the actor when their health changes.
/// How the health of an actor was changed since last check.
#[derive(Event, Debug, Hash, PartialEq, Eq, Clone, Copy, Reflect, Serialize, Deserialize)]
#[reflect(Debug, Hash, PartialEq, Clone, Serialize, Deserialize)]
pub enum HealthChange {
    Killed,
    Damaged,
    Healed,
    Revived,
}

/// The typical bundle for health.
/// You shouldn't have one of these without the other
/// as they together are used to properly track health and output
/// health events.
#[derive(Bundle)]
pub struct HealthBundle {
    pub health: Health,
    pub health_old: HealthOld,
}

impl HealthBundle {
    pub fn new(max: NonZero<u32>) -> Self {
        Self {
            health: Health::new(max),
            health_old: HealthOld::new(Some(max)),
        }
    }

    pub fn with_current(current: u32, max: NonZero<u32>) -> Self {
        Self {
            health: Health::with_current(NonZero::new(current), max),
            health_old: HealthOld::new(NonZero::new(current)),
        }
    }

    pub fn from_name(name: ActorName) -> Self {
        use ActorName as A;
        let max = match name {
            A::Warrior => 125,
            A::Priestess => 75,
            A::Theif => 75,
            A::Ogre => 200,
            A::Goblin => 70,
            A::Skeleton => 100,
            A::UnknownJim => 1,
        };

        Self::new(NonZero::new(max).unwrap())
    }
}

/// The health of an actor.
/// This also determines whether that actor is alive or not.
#[derive(Component, Clone, Copy, Reflect, Serialize, Deserialize)]
#[reflect(Component, Clone, Serialize, Deserialize)]
pub struct Health {
    /// When None, the actor is dead.
    /// This should never be above the `max`
    current: Option<NonZero<u32>>,
    max: NonZero<u32>,
}

impl Health {
    /// Makes a new health component with the current health
    /// set to the max.
    pub fn new(max: NonZero<u32>) -> Self {
        Self {
            current: Some(max),
            max,
        }
    }

    /// Makes a new health component with the given current health.
    pub fn with_current(current: Option<NonZero<u32>>, max: NonZero<u32>) -> Self {
        Self { current, max }
    }

    /// Get the current health
    #[inline]
    pub fn current(&self) -> Option<NonZero<u32>> {
        self.current
    }

    /// Get the max health
    #[inline]
    pub fn max(&self) -> NonZero<u32> {
        self.max
    }

    /// Get whether or not the actor is alive.
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

        debug_assert!(self.current.is_none_or(|curr| curr <= self.max));
    }

    /// Heals the actor or revives them if they are dead.
    /// Only revives actors if `amount` > 0
    #[inline]
    pub fn heal_or_revive(&mut self, amount: u32) {
        let Some(amount) = NonZero::<u32>::new(amount) else {
            return;
        };

        self.current = NonZero::new(
            self.current
                .map(|c| c.get())
                .unwrap_or(0)
                .saturating_add(amount.get())
                .min(self.max.get()),
        );

        debug_assert!(self.current.is_none_or(|curr| curr <= self.max));
    }

    /// Damage the actor, killing them if they health would go below one.
    #[inline]
    pub fn damage(&mut self, amount: u32) {
        let (Some(curr), Some(amount)) = (self.current, NonZero::<u32>::new(amount)) else {
            return;
        };

        self.current = NonZero::new(curr.get().saturating_sub(amount.get()));

        debug_assert!(self.current.is_none_or(|curr| curr <= self.max));
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

        debug_assert!(self.current.is_none_or(|curr| curr <= self.max));
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

        debug_assert!(self.current.is_none_or(|curr| curr <= self.max));
    }

    /// Damage the actor but only kill them if they were already at 1 health.
    #[inline]
    pub fn damage_no_one_shot(&mut self, amount: u32) {
        let (Some(curr), Some(amount)) = (self.current, NonZero::<u32>::new(amount)) else {
            return;
        };

        self.current = (curr == self.max)
            .then(|| {
                Some(
                    NonZero::new(curr.get().saturating_sub(amount.get()))
                        .unwrap_or(NonZero::new(1u32).unwrap()),
                )
            })
            .unwrap_or_else(|| NonZero::new(curr.get().saturating_sub(amount.get())));

        debug_assert!(self.current.is_none_or(|curr| curr <= self.max));
    }

    /// Kill the actor nomatter what
    #[inline]
    pub fn kill(&mut self) {
        self.current = None;
    }
}

/// The health of the actor before the latest round of [`kill_heal_revive`]
///
/// This is a separate entity so that changing the old health doesn't
/// re-trigger the event to update itself
#[derive(
    Component, Deref, DerefMut, Debug, Default, Clone, Copy, Reflect, Serialize, Deserialize,
)]
#[reflect(Component, Default, Debug, Clone, Serialize, Deserialize)]
pub struct HealthOld(Option<NonZero<u32>>);

impl HealthOld {
    #[inline]
    pub fn new(val: Option<NonZero<u32>>) -> Self {
        Self(val)
    }

    /// Updates the old health and returns how the actor's health
    /// has changed
    #[inline]
    pub fn update_old_health(&mut self, health: &Health) -> Option<HealthChange> {
        let old_old = **self;
        **self = health.current;

        match (old_old, health.current) {
            (Some(_), Option::None) => Some(HealthChange::Killed),
            (Option::None, Some(_)) => Some(HealthChange::Revived),
            (Some(o), Some(c)) if o > c => Some(HealthChange::Damaged),
            (Some(o), Some(c)) if o < c => Some(HealthChange::Healed),
            _ => None,
        }
    }
}

/// The chance the actor has to heal at the end
/// of the round in combat
/// Should be between 0.0 and 1.0
#[derive(Component, Deref, DerefMut, Clone, Copy, Reflect, Serialize, Deserialize)]
#[reflect(Component, Clone, Serialize, Deserialize)]
#[repr(transparent)]
pub struct HealChance(pub f32);

#[cfg(test)]
mod health_tests {
    use super::*;

    #[test]
    fn test_heal() {
        let mut health = Health::with_current(NonZero::new(5), NonZero::new(10).unwrap());
        health.heal(2);
        assert_eq!(health.current().unwrap().get(), 7);

        let mut health = Health::with_current(NonZero::new(5), NonZero::new(10).unwrap());
        health.heal(10);
        assert_eq!(health.current().unwrap().get(), 10);
        health.heal(10);
        assert_eq!(health.current().unwrap().get(), 10);

        let mut health = Health::with_current(NonZero::new(5), NonZero::new(10).unwrap());
        health.heal(0);
        assert_eq!(health.current().unwrap().get(), 5);

        let mut health = Health::with_current(NonZero::new(0), NonZero::new(10).unwrap());
        health.heal(0);
        assert_eq!(health.current(), None);
        health.heal(1);
        assert_eq!(health.current(), None);
    }

    #[test]
    fn test_heal_or_revive() {
        let mut health = Health::with_current(NonZero::new(5), NonZero::new(10).unwrap());
        health.heal_or_revive(2);
        assert_eq!(health.current().unwrap().get(), 7);

        let mut health = Health::with_current(NonZero::new(5), NonZero::new(10).unwrap());
        health.heal_or_revive(10);
        assert_eq!(health.current().unwrap().get(), 10);
        health.heal_or_revive(10);
        assert_eq!(health.current().unwrap().get(), 10);

        let mut health = Health::with_current(NonZero::new(5), NonZero::new(10).unwrap());
        health.heal_or_revive(0);
        assert_eq!(health.current().unwrap().get(), 5);

        let mut health = Health::with_current(NonZero::new(0), NonZero::new(10).unwrap());
        health.heal_or_revive(0);
        assert_eq!(health.current(), None);
        health.heal_or_revive(1);
        assert_eq!(health.current().unwrap().get(), 1);
    }

    #[test]
    fn test_damage() {
        let mut health = Health::with_current(NonZero::new(5), NonZero::new(10).unwrap());
        health.damage(2);
        assert_eq!(health.current().unwrap().get(), 3);
        health.damage(5);
        assert_eq!(health.current(), None);

        let mut health = Health::with_current(NonZero::new(5), NonZero::new(10).unwrap());
        health.damage(2);
        assert_eq!(health.current().unwrap().get(), 3);
        health.damage(3);
        assert_eq!(health.current(), None);

        let mut health = Health::with_current(NonZero::new(5), NonZero::new(10).unwrap());
        health.damage(10);
        assert_eq!(health.current(), None);
        health.damage(5);
        assert_eq!(health.current(), None);
    }

    #[test]
    fn test_damage_no_kill() {
        let mut health = Health::with_current(NonZero::new(5), NonZero::new(10).unwrap());
        health.damage_no_kill(2);
        assert_eq!(health.current().unwrap().get(), 3);
        health.damage_no_kill(5);
        assert_eq!(health.current().unwrap().get(), 1);
        health.damage_no_kill(1);
        assert_eq!(health.current().unwrap().get(), 1);

        let mut health = Health::with_current(NonZero::new(5), NonZero::new(10).unwrap());
        health.damage_no_kill(2);
        assert_eq!(health.current().unwrap().get(), 3);
        health.damage_no_kill(3);
        assert_eq!(health.current().unwrap().get(), 1);
        health.damage_no_kill(5);
        assert_eq!(health.current().unwrap().get(), 1);

        let mut health = Health::with_current(NonZero::new(5), NonZero::new(10).unwrap());
        health.damage_no_kill(10);
        assert_eq!(health.current().unwrap().get(), 1);
        health.damage_no_kill(5);
        assert_eq!(health.current().unwrap().get(), 1);
    }

    #[test]
    fn test_damage_endurence() {
        let mut health = Health::with_current(NonZero::new(5), NonZero::new(10).unwrap());
        health.damage_endurence(2);
        assert_eq!(health.current().unwrap().get(), 3);
        health.damage_endurence(5);
        assert_eq!(health.current().unwrap().get(), 1);
        health.damage_endurence(5);
        assert_eq!(health.current(), None);
        health.damage_endurence(1);
        assert_eq!(health.current(), None);

        let mut health = Health::with_current(NonZero::new(5), NonZero::new(10).unwrap());
        health.damage_endurence(2);
        assert_eq!(health.current().unwrap().get(), 3);
        health.damage_endurence(3);
        assert_eq!(health.current().unwrap().get(), 1);
        health.damage_endurence(5);
        assert_eq!(health.current(), None);
        health.damage_endurence(3);
        assert_eq!(health.current(), None);

        let mut health = Health::with_current(NonZero::new(5), NonZero::new(10).unwrap());
        health.damage_endurence(10);
        assert_eq!(health.current().unwrap().get(), 1);
        health.damage_endurence(5);
        assert_eq!(health.current(), None);
        health.damage_endurence(2);
        assert_eq!(health.current(), None);
    }
}

/// Heals all actors that end of round
/// based on their [`HealChance`]
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

/// Runs after the damage step before you want to trigger any animations.
/// Also updates the [`Health`]'s old health
pub fn kill_heal_revive(
    mut commands: Commands,
    mut actor_q: Query<(Entity, &Health, &mut HealthOld), Changed<Health>>,
) {
    actor_q
        .iter_mut()
        .filter_map(|(entity, health, mut old_health)| {
            old_health.update_old_health(health).zip(Some(entity))
        })
        .for_each(|(health_change, entity)| {
            commands.entity(entity).trigger(health_change);
        });
}

#[cfg(test)]
mod kill_heal_revive_tests {
    use super::*;

    #[test]
    fn test_kill_heal_revive() {
        kill_heal_revive_helper(10, 10, None);
        kill_heal_revive_helper(10, 5, Some(HealthChange::Healed));
        kill_heal_revive_helper(5, 10, Some(HealthChange::Damaged));
        kill_heal_revive_helper(0, 10, Some(HealthChange::Killed));
        kill_heal_revive_helper(10, 0, Some(HealthChange::Revived));
    }

    fn kill_heal_revive_helper(health: u32, old: u32, event: Option<HealthChange>) {
        // Setup app
        let mut app = App::new();

        // Add our two systems
        app.add_systems(Update, kill_heal_revive);

        // Setup test entities
        let health_id = app
            .world_mut()
            .spawn((
                Health::with_current(NonZero::new(health), NonZero::new(100).unwrap()),
                HealthOld::new(NonZero::new(old)),
            ))
            .observe(move |t: Trigger<HealthChange>| assert_eq!(Some(*t.event()), event))
            .id();

        // Run systems
        app.update();

        // Check resulting changes
        assert!(app.world().get::<HealthOld>(health_id).is_some());
        assert_eq!(
            **app.world().get::<HealthOld>(health_id).unwrap(),
            NonZero::new(health)
        );
    }
}
