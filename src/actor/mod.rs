mod attack;
mod health;

pub use attack::*;
pub use health::*;

use crate::prelude::*;
use bevy::prelude::*;
use rand::Rng;
use serde::{Deserialize, Serialize};
use std::fmt::Debug;
#[cfg(feature = "sqlite")]
use std::num::NonZero;
use strum::{Display, EnumIter};

pub const ACTOR_LAYER: f32 = 1.0;

/// The typical components for any given actor.
#[derive(Bundle)]
pub struct ActorBundle {
    pub actor: Actor,
    pub name: ActorName,
    pub team: Team,
    pub health: HealthBundle,
    pub attack: Attack,
    pub speed: AttackSpeed,
    pub transform: Transform,
    pub animation: AnimationBundle,
    pub block_chance: BlockChance,
}

impl ActorBundle {
    pub fn from_name(
        asset_server: &AssetServer,
        name: ActorName,
        team: Team,
        transform: Transform,
        alive: bool,
    ) -> Self {
        let mut health = HealthBundle::from_name(name);

        if !alive {
            health.health.kill();
        }

        Self {
            actor: Actor,
            name,
            team,
            health,
            attack: Attack::from_name(name),
            speed: AttackSpeed::from_name(name),
            transform,
            animation: AnimationBundle::from_name(asset_server, name),
            block_chance: BlockChance::from_name(name),
        }
    }
}

#[derive(Component)]
pub struct Actor;

#[cfg(feature = "sqlite")]
pub fn save_actors(
    components: Query<(&ActorName, &Team, &Health, &Attack, &AttackSpeed)>,
    save_info: Res<SaveGame>,
    db: NonSend<Database>,
) -> Result<(), DatabaseError> {
    let game_id = save_info.game_id;

    db.connection.execute(
        "DELETE FROM PlayerActor WHERE game_id = :game_id",
        (game_id.0,),
    )?;

    let query = r#"
        INSERT INTO PlayerActor(
            name,
            game_id,
            health_max,
            health_curr,
            attack_damage_min,
            attack_damage_max,
            hit_chance,
            attack_speed
        )
        VALUES(
            :name,
            :game,
            :health_max,
            :health_curr,
            :attack_damage_min,
            :attack_damage_max,
            :hit_chance,
            :attack_speed
        );
    "#;
    let mut statement = db.connection.prepare(query)?;

    for (name, team, health, attack, speed) in components {
        let Team::Player = team else {
            continue;
        };
        statement.execute((
            name.to_string(),
            *game_id,
            health.max(),
            health.current(),
            attack.damage.start,
            attack.damage.end,
            attack.hit_chance,
            speed.0,
        ))?;
    }

    Ok(())
}

#[cfg(feature = "sqlite")]
pub fn load_actors(
    mut commands: Commands,
    db: NonSend<Database>,
    save_game: Res<SaveGame>,
    asset_server: Res<AssetServer>,
) -> Result<(), DatabaseError> {
    let game_id = save_game.game_id;
    let query = r#"
            SELECT
                name,
                health_curr,
                health_max,
                attack_damage_max,
                attack_damage_min,
                attack_speed,
                hit_chance
            FROM PlayerActor WHERE PlayerActor.game_id = :game;
        "#;

    db.connection
        .prepare(query)?
        .query_map((game_id.0,), |row| {
            let name = row.get::<_, String>("name")?;
            let name = ron::from_str(&name).unwrap_or(ActorName::UnknownJim);

            let health = HealthBundle::with_current(
                row.get("health_curr")?,
                NonZero::new(row.get("health_max")?).unwrap_or(NonZero::new(1).unwrap()),
            );
            let attack = Attack::new(
                row.get("attack_damage_min")?..row.get("attack_damage_max")?,
                row.get("hit_chance")?,
            );
            let speed = AttackSpeed::new(row.get("attack_speed")?);
            // the actor will be placed after this
            let transform = Transform::IDENTITY;
            let animation = AnimationBundle::from_name(&asset_server, name);
            let block_chance = BlockChance::from_name(name);

            Ok(ActorBundle {
                actor: Actor,
                name,
                team: Team::Player,
                health,
                attack,
                speed,
                transform,
                animation,
                block_chance,
            })
        })?
        .for_each(|actor| {
            commands.spawn(actor.unwrap());
        });

    Ok(())
}

/// The team the actor is in for combat.
#[derive(
    Component, Debug, Hash, PartialEq, Eq, Clone, Copy, Serialize, Deserialize, EnumIter, Display,
)]
pub enum Team {
    /// The player controls this actor and
    /// decides their moves.
    Player,
    /// The game decides what these actors do
    /// on a given turn.
    Enemy,
}

/// The team the actor, both in combat and for the sprite image.
#[derive(
    Component, Debug, Hash, PartialEq, Eq, Clone, Copy, Serialize, Deserialize, EnumIter, Display,
)]
pub enum ActorName {
    Warrior,
    Priestess,
    Theif,
    Goblin,
    Ogre,
    Skeleton,
    #[strum(to_string = "Unknown Jim")]
    UnknownJim,
}

impl ActorName {
    pub fn get_enemies(rng: &mut impl Rng) -> Box<[ActorName]> {
        let mon = rng.random_range(0..10);
        let mut enemies: Vec<ActorName> = Vec::new();

        if mon < 1 {
            // I know that's not how you do it but I'll fix it laterElijah. Ok I'm sorry
            for _ in 0..3 {
                enemies.push(Self::get_rand_enemy(rng));
            }
        } else if mon < 3 {
            for _ in 0..2 {
                enemies.push(Self::get_rand_enemy(rng));
            }
        } else {
            enemies.push(Self::get_rand_enemy(rng));
        }

        enemies.into()
    }

    pub fn get_rand_enemy(rng: &mut impl Rng) -> ActorName {
        let idx = rng.random_range(0..3);

        match idx {
            0 => ActorName::Goblin,
            1 => ActorName::Ogre,
            2 => ActorName::Skeleton,
            _ => unreachable!(),
        }
    }
}

#[derive(Component, Debug, Hash, PartialEq, Eq, Clone, Copy, Serialize, Deserialize, Display)]
pub enum SpecialAction {
    #[strum(to_string = "Heal")]
    HealTarget,
    #[strum(to_string = "Crushing Blow")]
    CrushingBlow,
    #[strum(to_string = "Surprise Attack")]
    SurpriseAttack,
}
