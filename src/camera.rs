use bevy::prelude::*;

pub const CAMERA_DEFAULT_SCALE: f32 = 0.35;

/// The plugin to enable the camera
pub struct CameraPlugin;

impl Plugin for CameraPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Startup, camera_setup);
        //.add_systems(
        //    PostUpdate,
        //    (pause_game, (camera_movement, camera_zoom))
        //        .chain()
        //        .run_if(in_state(GameState::Game))
        //        .after(bevy::render::camera::camera_system),
        //);
    }
}

/// The marker component to signify a camera is the main rendering camera
#[derive(Resource)]
pub struct MainCamera(pub Entity);

/// The marker component to signify a camera is the main rendering camera
#[derive(Component)]
pub struct MainCameraMarker;

/// Sets up the main camera and it's settings
fn camera_setup(mut commands: Commands) {
    let camera_id = commands
        .spawn((
            MainCameraMarker,
            Camera2d,
            Projection::Orthographic(OrthographicProjection {
                scaling_mode: bevy::render::camera::ScalingMode::WindowSize,
                scale: CAMERA_DEFAULT_SCALE,
                ..OrthographicProjection::default_2d()
            }),
            Transform::IDENTITY,
        ))
        .id();
    commands.insert_resource(MainCamera(camera_id));
}
