use crate::*;
use wgpu::*;

use std::mem::MaybeUninit;

#[derive(Component, Deref)]
pub struct VoxelBuffer {
    #[deref]
    buffer: Buffer,
    dimension: UVec3,
}
impl VoxelBuffer {
    pub fn new(renderer: &Renderer, dimension: UVec3) -> Self {
        let buffer = renderer.device.create_buffer(&BufferDescriptor {
            label: Some("Voxel buffer"),
            size: (dimension.x * dimension.y * dimension.z) as u64 + size_of::<UVec4>() as u64, // despite dimension being uvec3, on the gpu side, its alignment is 16 so there is an extra padding.
            usage: BufferUsages::STORAGE | BufferUsages::COPY_DST,
            mapped_at_creation: false,
        });

        renderer
            .queue
            .write_buffer(&buffer, 0, bytemuck::bytes_of(&dimension));

        Self { buffer, dimension }
    }
    pub fn update(&self, renderer: &Renderer, voxel: &Voxel) {
        if self.dimension != voxel.dimension() {
            panic!("Cannot update buffer with voxel whose dimension does not match the buffer's dimension. Resize the buffer with the matching dimension and then update.");
        }
        renderer.queue.write_buffer(
            &*self,
            size_of::<UVec4>() as u64,
            bytemuck::cast_slice(&voxel.data),
        );
    }
}
#[derive(Component, Clone)]
pub struct Voxel {
    dimension: UVec3,
    data: Box<[u8]>,
}
impl Voxel {
    pub fn new(dimension: UVec3) -> Self {
        let UVec3 { x, y, z } = dimension;
        Self {
            dimension,
            data: vec![0; x as usize * y as usize * z as usize].into_boxed_slice(),
        }
    }
    pub const fn len(&self) -> usize {
        self.data.len()
    }
    pub const fn dimension(&self) -> UVec3 {
        self.dimension
    }
    pub fn get_index(dimension: UVec3, position: UVec3) -> Option<usize> {
        let UVec3 { x, y, z } = position;
        let UVec3 {
            x: dx,
            y: dy,
            z: dz,
        } = dimension;
        if x >= dx || y >= dy || z >= dz {
            return None;
        }
        Some(x as usize + y as usize * dx as usize + z as usize * dx as usize * dy as usize)
    }
    pub fn get_position(dimension: UVec3, index: usize) -> Option<UVec3> {
        let UVec3 {
            x: dx,
            y: dy,
            z: dz,
        } = dimension;
        if index >= dx as usize * dy as usize * dz as usize {
            return None;
        }
        Some(uvec3(
            index as u32 % dx,
            index as u32 / dx % dy,
            index as u32 / dx / dy,
        ))
    }
    pub fn for_each_mut(&mut self, mut callback: impl FnMut(&mut u8, UVec3)) {
        let dimension = self.dimension;
        for (i, v) in self.data.iter_mut().enumerate() {
            let position = Self::get_position(dimension, i).unwrap();
            callback(v, position);
        }
    }
    pub fn get(&self, position: UVec3) -> Option<&u8> {
        unsafe { Self::get_index(self.dimension, position).map(|i| self.data.get_unchecked(i)) }
    }
    pub fn get_mut(&mut self, position: UVec3) -> Option<&mut u8> {
        unsafe { Self::get_index(self.dimension, position).map(|i| self.data.get_unchecked_mut(i)) }
    }
}

#[derive(Component, Deref, Clone, Copy)]
pub struct VoxelColors([[u8; 4]; 256]);
impl VoxelColors {
    // Color palette that contains every color of RGBA channels where each channel has 2bits
    pub fn all_color() -> Self {
        #[allow(invalid_value)]
        let mut itself: Self = unsafe { MaybeUninit::uninit().assume_init() };
        for (i, color) in itself.0.iter_mut().enumerate() {
            color[0] = (i as u8 & 0b11) * 85;
            color[1] = (i as u8 >> 2 & 0b11) * 85;
            color[2] = (i as u8 >> 4 & 0b11) * 85;
            color[3] = (i as u8 >> 6 & 0b11) * 85;
        }
        itself
    }
}

#[derive(Resource, Deref)]
pub struct MainVoxelColors(Entity);
impl FromWorld for MainVoxelColors {
    fn from_world(world: &mut World) -> Self {
        let palette = world.spawn(VoxelColors::all_color()).id();
        Self(palette)
    }
}

#[derive(Resource, Deref)]
pub struct MainColorBuffer(Buffer);
impl MainColorBuffer {
    pub fn new(renderer: &Renderer) -> Self {
        let buffer = renderer.device.create_buffer(&BufferDescriptor {
            label: Some("Main color buffer"),
            size: size_of::<[[u8; 4]; 256]>() as u64,
            usage: BufferUsages::UNIFORM | BufferUsages::COPY_DST,
            mapped_at_creation: false,
        });

        Self(buffer)
    }
    pub fn update(&self, renderer: &Renderer, color: &VoxelColors) {
        renderer
            .queue
            .write_buffer(&*self, 0, bytemuck::bytes_of(&**color));
    }
}
impl FromWorld for MainColorBuffer {
    fn from_world(world: &mut World) -> Self {
        Self::new(world.resource())
    }
}
pub(super) fn sync_voxel_buffers(
    renderer: Res<Renderer>,
    voxel_q: Query<(&Voxel, &VoxelBuffer), Changed<Voxel>>,
) {
    for (voxel, buffer) in voxel_q.iter() {
        buffer.update(&renderer, voxel);
    }
}
pub(super) fn sync_color_buffer(
    renderer: Res<Renderer>,
    color_q: Query<Ref<VoxelColors>>,
    main_color: Res<MainVoxelColors>,
    color_buffer: Res<MainColorBuffer>,
) {
    let Ok(color) = color_q.get(**main_color) else {
        return;
    };
    if !color.is_changed() && !main_color.is_changed() {
        return;
    }

    color_buffer.update(&renderer, &color);
}
