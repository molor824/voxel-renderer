use bevy::ecs::query::*;
use bevy::math::*;
use bevy::prelude::*;
use bevy::window::*;
use camera::*;
use model::*;
use renderer::*;
use voxel_pipeline::*;

mod renderer;

fn contains_resource<T: Resource>(resource: Option<Res<T>>) -> bool {
    resource.is_some()
}

fn setup(
    mut commands: Commands,
    renderer: Res<Renderer>,
    voxel_layout: Res<VoxelBindLayout>,
    model_layout: Res<ModelBindLayout>,
    main_camera: Res<MainCamera>,
    mut camera_q: Query<&mut Transform>,
) {
    let mut camera = camera_q.get_mut(**main_camera).unwrap();

    camera.translation.z += 10.0;
    camera.translation.y += 5.0;
    camera.look_at(Vec3::ZERO, Vec3::Y);

    for z in -2..=2 {
        for y in -2..=2 {
            for x in -2..=2 {
                commands.spawn(VoxelBundle {
                    transform: TransformBundle::from_transform(Transform::from_translation(vec3(x as f32, y as f32, z as f32))),
                    ..VoxelBundle::new(&renderer, &voxel_layout, &model_layout, uvec3(32, 32, 32))
                });
            }
        }
    }
}
fn rotate_camera(mut camera_q: Query<&mut Transform, With<Camera>>, time: Res<Time>) {
    let delta = time.delta_seconds();
    let mut transform = camera_q.single_mut();

    transform.rotate_around(Vec3::ZERO, Quat::from_rotation_y(30.0_f32.to_radians() * delta));
}

fn main() {
    let window_plugin = WindowPlugin {
        primary_window: Some(Window {
            focused: true,
            title: "Voxel renderer".into(),
            resolution: WindowResolution::new(800.0, 600.0),
            ..Default::default()
        }),
        ..Default::default()
    };
    App::new()
        .add_plugins((
            DefaultPlugins.set(window_plugin),
            RenderPlugin,
            ModelPlugin,
            CameraPlugin,
            VoxelPlugin,
        ))
        .insert_resource(ClearColor(wgpu::Color::BLACK))
        .add_systems(Startup, setup.after(camera::setup))
        .add_systems(Update, rotate_camera)
        .run();
}
