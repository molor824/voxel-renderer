use bevy::prelude::*;
use bevy::window::{Window, WindowPlugin, WindowResolution};
use renderer::*;

mod renderer;

fn contains_resource<T: Resource>(world: &World) -> bool {
    world.contains_resource::<T>()
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
        .add_plugins((DefaultPlugins.set(window_plugin), RenderPlugin))
        .insert_resource(ClearColor(wgpu::Color::RED))
        .run();
}
