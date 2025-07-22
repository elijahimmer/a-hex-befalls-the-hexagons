use bevy::prelude::*;
use rand::Rng;
use serde::{Deserialize, Serialize};
use std::num::NonZero;
use std::ops::{DerefMut, Range};

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

/// The config for automating animation
#[derive(Component, Clone, Reflect, Serialize, Deserialize)]
#[reflect(Component, Clone, Serialize, Deserialize)]
pub struct AnimationConfig {
    first_sprite_index: usize,
    last_sprite_index: usize,
    tick_count: usize,
    ticks_per_frame: usize,
}

#[expect(dead_code)]
impl AnimationConfig {
    pub fn new(first: usize, last: usize, ticks_per_frame: usize) -> Self {
        Self {
            first_sprite_index: first,
            last_sprite_index: last,
            tick_count: 0,
            ticks_per_frame,
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

#[expect(dead_code)]
pub fn execute_animations(
    time: Res<Time>,
    mut frame_timer: ResMut<AnimationFrameTimer>,
    mut query: Query<(&mut AnimationConfig, &mut Sprite)>,
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
