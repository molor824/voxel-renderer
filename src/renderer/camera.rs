use crate::*;
use bytemuck::NoUninit;
use wgpu::*;

#[derive(Component, Clone, Copy)]
pub struct Orthographic {
    pub height_scale: f32,
    pub near: f32,
    pub far: f32,
}
impl Orthographic {
    // ratio is width / height
    pub fn projection(&self, aspect: f32) -> Mat4 {
        let width = aspect * self.height_scale;
        Mat4::orthographic_rh(
            width * -0.5,
            width * 0.5,
            self.height_scale * -0.5,
            self.height_scale * 0.5,
            self.near,
            self.far,
        )
    }
}
impl Default for Orthographic {
    fn default() -> Self {
        Self {
            height_scale: 2.0,
            near: -1.0,
            far: 1.0,
        }
    }
}
#[derive(Component, Clone, Copy)]
pub struct Perspective {
    pub fov: f32,
    pub near: f32,
    pub far: f32,
}
impl Perspective {
    pub fn projection(&self, aspect: f32) -> Mat4 {
        Mat4::perspective_rh(self.fov, aspect, self.near, self.far)
    }
}
impl Default for Perspective {
    fn default() -> Self {
        Self {
            fov: 90.0_f32.to_radians(),
            near: 0.001,
            far: 1000.0,
        }
    }
}

#[derive(Component, Clone, Copy)]
pub enum Camera {
    Orthographic(Orthographic),
    Perspective(Perspective),
}
impl Camera {
    pub fn projection(&self, aspect: f32) -> Mat4 {
        match self {
            Self::Orthographic(ortho) => ortho.projection(aspect),
            Self::Perspective(pers) => pers.projection(aspect),
        }
    }
}

#[derive(Clone, Copy)]
#[repr(C, align(16))]
pub struct CameraBufferValue {
    pub model: ModelBufferValue,
    pub projection: Mat4,
}
unsafe impl NoUninit for CameraBufferValue {}

#[derive(Resource, Deref)]
pub struct MainCameraBuffer(Buffer);
impl MainCameraBuffer {
    pub fn new(renderer: &Renderer) -> Self {
        Self(renderer.device.create_buffer(&BufferDescriptor {
            label: Some("Main camera buffer"),
            size: size_of::<CameraBufferValue>() as u64,
            usage: BufferUsages::UNIFORM | BufferUsages::COPY_DST,
            mapped_at_creation: false,
        }))
    }
    pub fn update(&self, renderer: &Renderer, value: &CameraBufferValue) {
        renderer
            .queue
            .write_buffer(&*self, 0, bytemuck::bytes_of(value));
    }
}
impl FromWorld for MainCameraBuffer {
    fn from_world(world: &mut World) -> Self {
        Self::new(world.resource())
    }
}

fn sync_main_buffer(
    renderer: Res<Renderer>,
    buffer: Res<MainCameraBuffer>,
    main_camera: Res<MainCamera>,
    camera_q: Query<(Ref<Camera>, Ref<GlobalTransform>)>,
    window_q: Query<&Window>,
) {
    let Ok((camera, transform)) = camera_q.get(**main_camera) else {
        return;
    };
    if !camera.is_changed() && !transform.is_changed() && !main_camera.is_changed() {
        return;
    }

    let window = window_q.single();
    let aspect = window.width() / window.height();

    let transform_matrix = transform.compute_matrix();

    buffer.update(
        &*renderer,
        &CameraBufferValue {
            model: ModelBufferValue {
                transform: transform_matrix,
                inv_transform: transform_matrix.inverse(),
            },
            projection: camera.projection(aspect),
        },
    );
}

#[derive(Resource, Deref)]
pub struct MainCamera(Entity);
impl FromWorld for MainCamera {
    fn from_world(world: &mut World) -> Self {
        let camera = world
            .spawn((
                Camera::Perspective(Default::default()),
                TransformBundle::default(),
            ))
            .id();

        Self(camera)
    }
}

pub struct CameraPlugin;
impl Plugin for CameraPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<MainCameraBuffer>();
        app.init_resource::<MainCamera>();

        app.add_systems(PostUpdate, sync_main_buffer.run_if(contains_resource::<Renderer>).before(RenderSystem::Begin));
    }
}
