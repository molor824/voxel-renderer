use crate::*;

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
        app.init_resource::<MainCamera>();
    }
}
