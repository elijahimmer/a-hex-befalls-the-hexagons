use super::*;
use bevy::prelude::*;

pub use imp::*;

#[cfg(feature = "sqlite")]
mod imp {
    use super::*;

    pub struct PouchPlugin;

    impl Plugin for PouchPlugin {
        fn build(&self, _app: &mut App) {}
    }

    pub fn add_pillar(mut save_game: ResMut<SaveGame>) {
        save_game.pillar_count += 1;
    }

    pub fn pillar_count(save_game: Res<SaveGame>, mut next_state: ResMut<NextState<GameState>>) {
        if save_game.pillar_count == 4 {
            next_state.set(GameState::Victory);
        }
    }
}

#[cfg(not(feature = "sqlite"))]
mod imp {
    use super::*;

    pub struct PouchPlugin;

    impl Plugin for PouchPlugin {
        fn build(&self, app: &mut App) {
            app.init_resource::<PillarCount>();
        }
    }

    #[derive(Resource, Deref, DerefMut, Default)]
    pub struct PillarCount(pub usize);

    pub fn add_pillar(mut pillars: ResMut<PillarCount>) {
        **pillars += 1;
    }

    pub fn pillar_count(pillars: Res<PillarCount>, mut next_state: ResMut<NextState<GameState>>) {
        if **pillars == 4 {
            next_state.set(GameState::Victory);
        }
    }
}
