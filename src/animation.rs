use crate::embed_asset;
use crate::prelude::*;
use bevy::prelude::*;
use serde::{Deserialize, Serialize};

const NORMAL_TICK_SPEED: usize = 1;

pub struct AnimationPlugin;

impl Plugin for AnimationPlugin {
    fn build(&self, app: &mut App) {
        embed_asset!(app, "assets/sprites/Warrior.png");
        embed_asset!(app, "assets/sprites/Priestess.png");
        embed_asset!(app, "assets/sprites/Theif.png");
        embed_asset!(app, "assets/sprites/Ogre.png");
        embed_asset!(app, "assets/sprites/Goblin.png");
        embed_asset!(app, "assets/sprites/Skeleton.png");
        embed_asset!(app, "assets/sprites/Unknown Jim.png");
        app.init_resource::<AnimationFrameTimer>()
            .add_systems(Update, execute_animations);
    }
}

/// The number of seconds the per AnimationFrameTimer trigger.
pub const ANIMATION_FRAME_TIMER_SECONDS: f32 = 0.5;

#[derive(Resource, Deref, DerefMut, Reflect)]
#[reflect(Resource, Default)]
pub struct AnimationFrameTimer(pub Timer);

impl Default for AnimationFrameTimer {
    fn default() -> Self {
        Self(Timer::from_seconds(
            ANIMATION_FRAME_TIMER_SECONDS,
            TimerMode::Repeating,
        ))
    }
}

#[derive(Bundle)]
pub struct AnimationBundle {
    sprite: Sprite,
    animations: AnimationConfigs,
}

impl AnimationBundle {
    pub fn from_name(asset_server: &AssetServer, name: ActorName) -> Self {
        let sprite = name_to_sprite(asset_server, name);

        let animations = AnimationConfigs::from_name(name);

        Self { sprite, animations }
    }
}

#[derive(Component, Serialize, Deserialize)]
pub struct AnimationConfigs {
    /// The normal animation
    normal: AnimationConfig,
    damaged: AnimationConfig,
    dead: AnimationConfig,
    active: ActiveAnimation,
    tick_count: usize,
    ticks_per_frame: usize,
}

impl AnimationConfigs {
    pub fn from_name(name: ActorName) -> Self {
        Self {
            normal: AnimationConfig::from_name(ActiveAnimation::Normal, name),
            damaged: AnimationConfig::from_name(ActiveAnimation::Damaged, name),
            dead: AnimationConfig::from_name(ActiveAnimation::Dead, name),
            active: ActiveAnimation::Normal,
            tick_count: 0,
            ticks_per_frame: NORMAL_TICK_SPEED,
        }
    }

    pub fn current(&self) -> &AnimationConfig {
        use ActiveAnimation as A;
        match self.active {
            A::Normal => &self.normal,
            A::Damaged => &self.damaged,
            A::Dead => &self.dead,
        }
    }

    /// Ticks the animation counter and
    /// returns true if the animation should progress
    pub fn tick(&mut self) {
        if self.tick_count == self.ticks_per_frame {
            self.tick_count = 0;
        } else {
            self.tick_count += 1;
        }
    }

    pub fn should_progress(&self) -> bool {
        self.tick_count == 0
    }
}

#[derive(Debug, Hash, PartialEq, Eq, Clone, Copy, Serialize, Deserialize)]
pub enum ActiveAnimation {
    Normal,
    Damaged,
    Dead,
}

/// The config for automating animation
#[derive(Clone, Serialize, Deserialize)]
pub struct AnimationConfig {
    first_sprite_index: usize,
    last_sprite_index: usize,
}

impl AnimationConfig {
    pub fn new(first: usize, last: usize) -> Self {
        Self {
            first_sprite_index: first,
            last_sprite_index: last,
        }
    }

    pub fn from_name(active: ActiveAnimation, name: ActorName) -> Self {
        use ActiveAnimation as A;
        use ActorName as C;
        match (active, name) {
            // TODO: Make Real stats Self stats accurate (I copied it from Theif)
            (A::Normal, C::Warrior) => Self::new(0, 1),
            (A::Damaged, C::Warrior) => Self::new(0, 1),
            (A::Dead, C::Warrior) => Self::new(0, 1),

            (A::Normal, C::Priestess) => Self::new(0, 1),
            (A::Damaged, C::Priestess) => Self::new(0, 1),
            (A::Dead, C::Priestess) => Self::new(0, 1),

            (A::Normal, C::Theif) => Self::new(0, 1),
            (A::Damaged, C::Theif) => Self::new(0, 1),
            (A::Dead, C::Theif) => Self::new(0, 1),

            (A::Normal, C::Ogre) => Self::new(0, 1),
            (A::Damaged, C::Ogre) => Self::new(0, 1),
            (A::Dead, C::Ogre) => Self::new(0, 1),

            (A::Normal, C::Goblin) => Self::new(0, 1),
            (A::Damaged, C::Goblin) => Self::new(0, 1),
            (A::Dead, C::Goblin) => Self::new(0, 1),

            (A::Normal, C::Skeleton) => Self::new(0, 1),
            (A::Damaged, C::Skeleton) => Self::new(0, 1),
            (A::Dead, C::Skeleton) => Self::new(0, 1),

            (A::Normal, C::UnknownJim) => Self::new(0, 3),
            (A::Damaged, C::UnknownJim) => Self::new(4, 4),
            (A::Dead, C::UnknownJim) => Self::new(8, 8),
        }
    }
}

pub fn name_to_sprite(asset_server: &AssetServer, name: ActorName) -> Sprite {
    let asset = asset_server.load(name_to_sprite_path(name));
    let atlas_layout = name_to_atlas_layout(name);
    let atlas_layout = asset_server.add(atlas_layout);

    let atlas = TextureAtlas {
        layout: atlas_layout,
        index: 0,
    };

    Sprite::from_atlas_image(asset, atlas)
}

pub fn name_to_sprite_path(name: ActorName) -> String {
    format!("embedded://assets/sprites/{}.png", name)
}

pub fn name_to_sprite_size(name: ActorName) -> UVec2 {
    use ActorName as A;
    match name {
        A::Warrior => UVec2::new(32, 60),
        A::Priestess => UVec2::new(32, 60),
        A::Theif => UVec2::new(34, 60),
        A::Ogre => UVec2::new(32, 60),
        A::Goblin => UVec2::new(32, 60),
        A::Skeleton => UVec2::new(32, 60),
        A::UnknownJim => UVec2::new(32, 60),
    }
}

pub fn name_to_atlas_layout(name: ActorName) -> TextureAtlasLayout {
    use ActorName as A;
    let (columns, rows) = match name {
        A::Warrior => (2, 1),
        A::Priestess => (2, 1),
        A::Theif => (2, 1),
        A::Ogre => (2, 1),
        A::Goblin => (2, 1),
        A::Skeleton => (2, 1),
        A::UnknownJim => (4, 2),
    };

    TextureAtlasLayout::from_grid(name_to_sprite_size(name), columns, rows, None, None)
}

pub fn execute_animations(
    time: Res<Time>,
    mut frame_timer: ResMut<AnimationFrameTimer>,
    mut query: Query<(&mut AnimationConfigs, &mut Sprite)>,
) {
    frame_timer.tick(time.delta());

    if !frame_timer.just_finished() {
        return;
    }

    for (mut config, mut sprite) in &mut query {
        config.tick();
        if !config.should_progress() {
            continue;
        }

        let config = config.current();

        let Some(atlas) = &mut sprite.texture_atlas else {
            continue;
        };

        if atlas.index == config.last_sprite_index {
            atlas.index = config.first_sprite_index;
        } else {
            atlas.index += 1;
        }
    }
}
