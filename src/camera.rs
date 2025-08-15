use crate::generate_map::WORLD_MAP_ORIGIN;
use bevy::prelude::*;
use bevy::render::{
    camera::RenderTarget,
    render_asset::RenderAssetUsages,
    render_resource::{Extent3d, TextureDimension, TextureFormat, TextureUsages},
    view::RenderLayers,
};

pub const CAMERA_DEFAULT_SCALE: f32 = 1.00;
pub const CAMERA_MAP_SCALE: f32 = 2.0;

/// The plugin to enable the camera
pub struct CameraPlugin;

impl Plugin for CameraPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Startup, camera_setup);
    }
}

/// The marker component to signify a camera is the main rendering camera
#[derive(Component)]
pub struct MainCameraMarker;

/// The marker component to signify a camera is the main rendering camera
#[derive(Component)]
pub struct MapCameraMarker;

/// Sets up the main camera and it's settings
fn camera_setup(mut commands: Commands, mut images: ResMut<Assets<Image>>) {
    commands.spawn((
        MainCameraMarker,
        Camera2d,
        Projection::Orthographic(OrthographicProjection {
            scaling_mode: bevy::render::camera::ScalingMode::WindowSize,
            scale: CAMERA_DEFAULT_SCALE,
            ..OrthographicProjection::default_2d()
        }),
        Transform::IDENTITY,
    ));

    let size = Extent3d {
        width: 300,
        height: 300,
        ..default()
    };

    // This is the texture that will be rendered to.
    let mut image = Image::new_fill(
        size,
        TextureDimension::D2,
        &[0, 0, 0, 0],
        TextureFormat::Bgra8UnormSrgb,
        RenderAssetUsages::default(),
    );
    // You need to set these texture usage flags in order to use the image as a render target
    image.texture_descriptor.usage =
        TextureUsages::TEXTURE_BINDING | TextureUsages::COPY_DST | TextureUsages::RENDER_ATTACHMENT;

    let image_handle = images.add(image);

    commands.spawn((
        ImageNode {
            image: image_handle.clone().into(),
            ..default()
        },
        Pickable::IGNORE,
        Node {
            justify_self: JustifySelf::End,
            ..default()
        },
    ));

    commands.spawn((
        MapCameraMarker,
        Camera2d,
        Camera {
            target: RenderTarget::Image(image_handle.clone().into()),
            clear_color: ClearColorConfig::None,
            ..default()
        },
        Projection::Orthographic(OrthographicProjection {
            scaling_mode: bevy::render::camera::ScalingMode::WindowSize,
            scale: CAMERA_MAP_SCALE,
            ..OrthographicProjection::default_2d()
        }),
        Transform::from_translation(WORLD_MAP_ORIGIN),
    ));
}
