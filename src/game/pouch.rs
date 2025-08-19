use super::*;
use bevy::prelude::*;

pub fn add_pillar(mut save_game: ResMut<SaveGame>) {
    save_game.pillar_count += 1;
}

pub fn pillar_count(save_game: Res<SaveGame>, mut next_state: ResMut<NextState<GameState>>) {
    if save_game.pillar_count == 4 {
        next_state.set(GameState::Victory);
    }
}
