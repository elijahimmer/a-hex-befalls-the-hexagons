use bevy::input_focus::InputFocus;
use bevy::prelude::*;
use bevy::state::state::FreelyMutableState;

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
pub fn despawn_all_with<T: Component>(to_despawn: Query<Entity, With<T>>, mut commands: Commands) {
    for entity in &to_despawn {
        commands.entity(entity).despawn();
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
