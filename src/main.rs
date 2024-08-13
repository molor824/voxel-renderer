use bevy::ecs::query::*;
use bevy::math::*;
use bevy::prelude::*;
use bevy::window::*;
use camera::*;
use model::*;
use renderer::*;
use voxel::*;

mod renderer;
mod voxel;

fn contains_resource<T: Resource>(resource: Option<Res<T>>) -> bool {
    resource.is_some()
}

fn setup(
    mut commands: Commands,
    renderer: Res<Renderer>,
    voxel_pipeline: Res<voxel::Pipeline>,
    main_camera: Res<MainCamera>,
    mut camera_q: Query<&mut Transform>,
) {
    let mut camera = camera_q.get_mut(**main_camera).unwrap();

    camera.translation.z += 2.0;
    camera.translation.y += 1.0;
    camera.look_at(Vec3::ZERO, Vec3::Y);

    commands.spawn(VoxelBundle::new(
        UVec3::splat(16),
        &renderer,
        &voxel_pipeline,
    ));
}
fn set_voxel(mut voxel_q: Query<&mut Voxel>) {
    let Some(mut voxel) = voxel_q.iter_mut().next() else {
        return;
    };
    let center = voxel.dimension().as_vec3() * 0.5;
    let radius = 6.0;
    voxel.for_each_mut(|v, position| {
        let position = position.as_vec3();
        *v = if (center - position).length() < radius {
            0xff
        } else {
            0x0
        };
    });
}
fn rotate_camera(mut camera_q: Query<&mut Transform, With<Camera>>, time: Res<Time>) {
    let delta = time.delta_seconds();
    let mut transform = camera_q.single_mut();

    transform.rotate_around(
        Vec3::ZERO,
        Quat::from_rotation_y(45.0_f32.to_radians() * delta),
    );
}

fn main() {
    let window_plugin = WindowPlugin {
        primary_window: Some(Window {
            focused: true,
            title: "Voxel renderer".into(),
            resolution: WindowResolution::new(800.0, 600.0),
            present_mode: PresentMode::AutoVsync,
            ..Default::default()
        }),
        ..Default::default()
    };
    App::new()
        .add_plugins((DefaultPlugins.set(window_plugin), RenderPlugin, VoxelPlugin))
        .insert_resource(ClearColor(wgpu::Color::BLACK))
        .add_systems(Startup, (setup, set_voxel.after(setup)))
        .add_systems(Update, rotate_camera)
        .run();
}
