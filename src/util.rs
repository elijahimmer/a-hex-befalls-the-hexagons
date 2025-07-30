use bevy::ecs::query::QueryFilter;
use bevy::ecs::schedule::ScheduleConfigs;
use bevy::ecs::system::ScheduleSystem;
use bevy::input_focus::InputFocus;
use bevy::prelude::*;
use bevy::state::state::FreelyMutableState;
use std::time::Duration;

/// TODO: Replace with `std::f32::consts::SQRT_3` when that is stable.
//pub const SQRT_3: f32 = 1.732050807568877293527446341505872367_f32;
pub const SQRT_3_2: f32 = 0.866025403784438646763723170752936183_f32;

/// The full hex size
pub const FLOOR_TILE_SIZE: IVec2 = IVec2 { x: 24, y: 26 };

#[cfg(feature = "debug")]
pub const FPS_COUNTER_COLOR: Color = Color::srgb(0.0, 1.0, 0.0);

//#[derive(Resource)]
//pub struct GlobalRandom(RandomSource);

#[macro_export]
macro_rules! embed_asset {
    ($app: ident, $path: expr) => {{
        let embedded = $app
            .world_mut()
            .resource_mut::<::bevy::asset::io::embedded::EmbeddedAssetRegistry>();

        embedded.insert_asset(
            concat!(env!("CARGO_MANIFEST_DIR"), "/", $path).into(),
            ::std::path::Path::new($path),
            include_bytes!(concat!(env!("CARGO_MANIFEST_DIR"), "/", $path)),
        );
    }};
}

/// Helper method to despawn all of the entities with a given component.
/// This is used with the `On*` Components to easily destroy all of the components
/// on specific screens
pub fn despawn_filtered<T: QueryFilter>(mut commands: Commands, to_despawn: Query<Entity, T>) {
    for entity in &to_despawn {
        commands.entity(entity).despawn();
    }
}

pub fn remove_component<T: Component>(mut commands: Commands, to_despawn: Query<Entity, With<T>>) {
    for entity in &to_despawn {
        commands.entity(entity).remove::<T>();
    }
}

pub fn remove_resource<T: Resource>(mut commands: Commands) {
    commands.remove_resource::<T>();
}

pub fn init_resource<T: Resource + FromWorld>(mut commands: Commands) {
    commands.init_resource::<T>();
}

pub fn set_state<T: States + FreelyMutableState + Clone>(
    state: T,
) -> impl Fn(ResMut<NextState<T>>) {
    move |mut next| next.set(state.clone())
}

pub fn stop_event_propagate<T: Event>(mut event: Trigger<T>) {
    event.propagate(false);
}

pub fn change_state_on_click<State: FreelyMutableState + Clone>(
    click: PointerButton,
    state: State,
) -> impl Fn(Trigger<Pointer<Click>>, ResMut<NextState<State>>) {
    move |mut event, mut next_state| {
        if event.button != click {
            return;
        }

        next_state.set(state.clone());
        event.propagate(false);
    }
}

pub fn change_state<State: FreelyMutableState + Clone>(
    state: State,
) -> ScheduleConfigs<ScheduleSystem> {
    (move |mut next_state: ResMut<NextState<State>>| next_state.set(state.clone())).into_configs()
}

pub fn log_event<T: Event>(_event: Trigger<T>) {
    let name = core::any::type_name::<T>();
    info!("Event {name} sent!");
}

pub fn clear_focus_on_click(
    mut click: Trigger<Pointer<Click>>,
    mut input_focus: ResMut<InputFocus>,
) {
    input_focus.clear();
    click.propagate(false);
}

#[derive(Resource)]
pub struct OldFixedDuration(pub Duration);

/// Change the fixed update timer so that this section will go much faster.
pub fn set_fixed_update_time(frequency: f64) -> ScheduleConfigs<ScheduleSystem> {
    (move |mut commands: Commands, mut time: ResMut<Time<Fixed>>| {
        commands.insert_resource(OldFixedDuration(time.timestep()));
        time.set_timestep_hz(frequency);
    })
    .into_configs()
}

/// Change back the time to not affect any other fixed update things.
pub fn restore_fixed_update_time(
    mut commands: Commands,
    mut time: ResMut<Time<Fixed>>,
    old_duration: Res<OldFixedDuration>,
) {
    time.set_timestep(old_duration.0);
    commands.remove_resource::<OldFixedDuration>();
}
