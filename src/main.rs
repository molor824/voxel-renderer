use std::f32::consts::PI;
use bevy::ecs::query::*;
use bevy::input::mouse::MouseMotion;
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
    voxel_pipeline: Res<Pipeline>,
    main_camera: Res<MainCamera>,
    mut camera_q: Query<&mut Transform>,
    mut window_q: Query<&mut Window, With<PrimaryWindow>>
) {
    let mut camera = camera_q.get_mut(**main_camera).unwrap();

    camera.translation.z += 2.0;
    camera.translation.y += 1.0;
    camera.translation *= 0.5;
    camera.look_at(Vec3::ZERO, Vec3::Y);

    commands.spawn(VoxelBundle::new(
        UVec3::splat(64),
        &renderer,
        &voxel_pipeline,
    ));

    let mut window = window_q.single_mut();

    window.cursor.grab_mode = CursorGrabMode::Locked;
    window.cursor.visible = false;
}
fn set_voxel(mut voxel_q: Query<&mut Voxel>) {
    let Some(mut voxel) = voxel_q.iter_mut().next() else {
        return;
    };
    voxel.for_each_mut(|v, position| {
        let scaled_position = position.as_vec3() * (16.0 / 64.0);
        let height = (scaled_position.x.cos() + scaled_position.z.sin()) + 7.0;
        let scaled_position1 = scaled_position / 3.0;
        *v = if scaled_position.y > (height + (
            scaled_position1.x.cos() + scaled_position1.z.cos() + scaled_position1.y.sin()
        ) * 2.0) {
            0x0
        } else {
            0b11000100
        };
    });
}
fn camera_movement(mut camera_q: Query<&mut Transform, With<Camera>>, time: Res<Time>, input: Res<ButtonInput<KeyCode>>, mut mouse_motion: EventReader<MouseMotion>) {
    const SENSITIVITY: f32 = 0.01;
    const SPEED: f32 = 1.0;

    let delta = time.delta_seconds();
    let mut camera = camera_q.single_mut();
    let mouse_delta = mouse_motion.read().fold(Vec2::ZERO, |acc, motion| acc + motion.delta);

    let mut euler = camera.rotation.to_euler(EulerRot::YXZ);
    euler.0 -= mouse_delta.x * SENSITIVITY;
    euler.1 -= mouse_delta.y * SENSITIVITY;
    euler.1 = euler.1.clamp(-PI * 0.5, PI * 0.5);

    camera.rotation = Quat::from_euler(EulerRot::YXZ, euler.0, euler.1, euler.2);

    let mut direction = Vec3::ZERO;
    if input.pressed(KeyCode::KeyW) {
        direction.z -= 1.0;
    }
    if input.pressed(KeyCode::KeyS) {
        direction.z += 1.0;
    }
    if input.pressed(KeyCode::KeyA) {
        direction.x -= 1.0;
    }
    if input.pressed(KeyCode::KeyD) {
        direction.x += 1.0;
    }
    if input.pressed(KeyCode::KeyE) {
        direction.y += 1.0;
    }
    if input.pressed(KeyCode::KeyQ) {
        direction.y -= 1.0;
    }

    let rotation = camera.rotation;
    camera.translation += rotation * direction * delta * SPEED;
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
        .add_plugins((DefaultPlugins.set(window_plugin), RenderPlugin, VoxelPlugin))
        .insert_resource(ClearColor(wgpu::Color::BLACK))
        .add_systems(Startup, (setup, set_voxel.after(setup)))
        .add_systems(Update, camera_movement)
        .run();
}
