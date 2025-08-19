use super::*;
use bevy::prelude::*;

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

pub fn add_pillar(pouch_q: Query<&mut Pouch>) {
    for mut pillar in pouch_q {
        pillar.count += 1;
    }
}

pub fn pillar_count(pouch_q: Query<&mut Pouch>, mut next_state: ResMut<NextState<GameState>>) {
    for pillar in pouch_q {
        if pillar.count == 4 {
            next_state.set(GameState::Victory);
        }
    }
}
