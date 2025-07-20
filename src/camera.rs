use crate::prelude::*;
use bevy::prelude::ops::powf;
use bevy::prelude::*;

pub const CAMERA_DEFAULT_SCALE: f32 = 0.5;
pub const CAMERA_DEFAULT_MOVE_SPEED: f32 = 300.0;
pub const CAMERA_DEFAULT_ZOOM_SPEED: f32 = 4.0;
pub const CAMERA_DEFAULT_MOVE_AREA: Mat2 = Mat2::from_cols_array_2d(&[[-200.0, 200.0]; 2]);
pub const CAMERA_DEFAULT_ZOOM_LIMIT: Vec2 = Vec2::new(0.25, 1.0);

/// The plugin to enable the camera
pub struct CameraPlugin;

impl Plugin for CameraPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<CameraMovementSettings>()
            .add_systems(Startup, camera_setup)
            .add_systems(
                PostUpdate,
                (pause_game, (camera_movement, camera_zoom))
                    .chain()
                    //.run_if(in_state(GameState::Game))
                    .after(bevy::render::camera::camera_system),
            );
    }
}

/// The marker component to signify a camera is the main rendering camera
#[derive(Resource)]
pub struct MainCamera(pub Entity);

/// The marker component to signify a camera is the main rendering camera
#[derive(Component)]
pub struct MainCameraMarker;

/// The camera movement settings for the [`MainCamera`]
#[derive(Resource)]
pub struct CameraMovementSettings {
    /// The movement speed of the camera in in-game pixels per second
    pub move_speed: f32,

    /// The zoom speed of the camera defined as `zoom *= speed ^ (delta seconds)`
    /// This uses the inverse of the speed when zooming in.
    pub zoom_speed: f32,

    /// The allowed area the camera can be in, where
    /// each column is the range of movement for a direction.
    /// The first column is the `x` direction, and the second is `y`
    pub move_area: Mat2,

    /// The bounds of the zoom, `x` being the lower bound and `y` being the upper bound.
    pub zoom_limit: Vec2,
}

impl Default for CameraMovementSettings {
    fn default() -> Self {
        Self {
            move_speed: CAMERA_DEFAULT_MOVE_SPEED,
            zoom_speed: CAMERA_DEFAULT_ZOOM_SPEED,
            move_area: CAMERA_DEFAULT_MOVE_AREA,
            zoom_limit: CAMERA_DEFAULT_ZOOM_LIMIT,
        }
    }
}

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

fn pause_game(mut commands: Commands, input: Res<ControlState>) {
    if input.just_pressed(Control::Pause) {
        commands.set_state(GameState::Menu);
    }
}

/// Controls the camera's translational movement based
/// on user input.
fn camera_movement(
    camera_id: Res<MainCamera>,
    mut transform: Query<&mut Transform, With<MainCameraMarker>>,
    settings: Res<CameraMovementSettings>,
    input: Res<ControlState>,
    time: Res<Time>,
) {
    let transform: &mut Transform = &mut transform
        .get_mut(camera_id.0)
        .expect("There should be a camera!");

    let movement = Vec2::Y * input.pressed(Control::MoveUp) as u8 as f32
        + Vec2::NEG_Y * input.pressed(Control::MoveDown) as u8 as f32
        + Vec2::NEG_X * input.pressed(Control::MoveLeft) as u8 as f32
        + Vec2::X * input.pressed(Control::MoveRight) as u8 as f32;

    let movement = movement * time.delta_secs() * settings.move_speed;

    transform.translation = (transform.translation.xy() + movement)
        .clamp(settings.move_area.row(0), settings.move_area.row(1))
        .extend(transform.translation.z);
}

/// Controls the camera's zoom based on user input.
fn camera_zoom(
    camera_id: Res<MainCamera>,
    mut projection: Query<&mut Projection, With<MainCameraMarker>>,
    settings: Res<CameraMovementSettings>,
    input: Res<ControlState>,
    time: Res<Time>,
) {
    let projection = &mut projection
        .get_mut(camera_id.0)
        .expect("There should be a camera!");
    let Projection::Orthographic(ref mut projection2d) = **projection else {
        unreachable!("Only Orthographic Projection is supported!");
    };

    let scale = projection2d.scale
        * powf(
            powf(settings.zoom_speed, time.delta_secs()),
            input.pressed(Control::ZoomIn) as u8 as f32,
        )
        * powf(
            powf(1.0 / settings.zoom_speed, time.delta_secs()),
            input.pressed(Control::ZoomOut) as u8 as f32,
        );

    projection2d.scale = scale.clamp(settings.zoom_limit.x, settings.zoom_limit.y);
}
