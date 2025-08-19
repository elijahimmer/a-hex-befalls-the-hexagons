use super::*;
use crate::prelude::*;
use bevy::prelude::*;
use rand::Rng;
use serde::{Deserialize, Serialize};
use std::ops::Range;

pub struct PouchPlugin;

impl Plugin for PouchPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Startup, spawn_pouch);
    }
}

#[derive(Component)]
pub struct Pouch {
    count: u32,
}

pub fn spawn_pouch(mut commands: Commands) {
    commands.spawn(Pouch { count: 0 });
}

pub fn add_pillar(mut commands: Commands, pouch_q: Query<&mut Pouch>) {
    info!("working");
    for mut pillar in pouch_q {
        info!(pillar.count);
        pillar.count += 1;
        info!(pillar.count);
    }
}

pub fn pillar_count(
    mut commands: Commands,
    pouch_q: Query<&mut Pouch>,
    mut next_state: ResMut<NextState<GameState>>,
) {
    for mut pillar in pouch_q {
        if pillar.count == 4 {
            next_state.set(GameState::Victory);
        }
    }
}
