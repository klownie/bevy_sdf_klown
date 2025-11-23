use bevy::math::VectorSpace;
use bevy::prelude::*;
use bevy::render::render_resource::ShaderType;
use bevy_sdf_klown_derive::EnumVariantGpuFields;
use std::mem::transmute;

#[derive(ShaderType, Clone, Copy)]
pub struct SdObjectUniform {
    pub shape: SdShapeUniform,
    pub material: SdMaterialUniform,
    pub modifier_stack: SdModStackUniform,
    pub transform: SdTransformUniform,
}

#[derive(Reflect, Debug, Clone)]
pub struct SdObject {
    pub shape: SdShape,
    pub material: SdMaterial,
    pub modifier_stack: SdModStack,
    pub transform: SdTransform,
}

impl SdObject {
    pub fn uniform(&self, start_mod_index: usize, start_shape_index: usize) -> SdObjectUniform {
        SdObjectUniform {
            shape: self.shape.uniform(start_shape_index),
            material: self.material.uniform(),
            modifier_stack: self.modifier_stack.clone().uniform(start_mod_index),
            transform: self.transform.uniform(),
        }
    }
}

#[repr(C)]
#[repr(u8)]
#[derive(Reflect, Component, Debug, Copy, Clone, EnumVariantGpuFields)]
#[require(Name::new("SdObject"), SdModStack, Transform, GlobalTransform)]
#[reflect(Component)]
pub enum SdShape {
    Sphere {
        radius: f32,
    },
    Ellipsoid {
        radius: Vec3,
    },
    Box {
        bounds: Vec3,
    },
    RoundBox {
        bounds: Vec3,
        radius: f32,
    },
    BoxFrame {
        bounds: Vec3,
        edge: f32,
    },
    Gyroid {
        height: f32,
    },
    Torus {
        major_radius: f32,
        minor_radius: f32,
    },
    CappedTorus {
        major_radius: f32,
        minor_radius: f32,
        sincos: Vec2,
    },
    Link {
        major_radius: f32,
        minor_radius: f32,
        length: f32,
    },
    VerticalCapsule {
        height: f32,
        radius: f32,
    },
    Capsule {
        a: Vec3,
        b: Vec3,
        radius: f32,
    },
    Cylinder {
        a: Vec3,
        b: Vec3,
        radius: f32,
    },
    VerticalCylinder {
        height: f32,
        radius: f32,
    },
    RoundedCylinder {
        height: f32,
        radius: f32,
        edge_radius: f32,
    },
    InfiniteCylinder {
        center: Vec3,
    },
    Cone {
        height: f32,
        sincos: Vec2,
    },
    ConeBound {
        height: f32,
        sincos: Vec2,
    },
    InfiniteCone {
        sincos: Vec2,
    },
    CappedVerticalCone {
        height: f32,
        r1: f32,
        r2: f32,
    },
    CappedCone {
        a: Vec3,
        b: Vec3,
        ra: f32,
        rb: f32,
    },
    RoundVerticalCone {
        height: f32,
        r1: f32,
        r2: f32,
    },
    RoundCone {
        a: Vec3,
        b: Vec3,
        r1: f32,
        r2: f32,
    },
    SolidAngle {
        sincos: Vec2,
        radius: f32,
    },
    Plane {
        normal: Vec3,
        height: f32,
    },
    Octahedron {
        size: f32,
    },
    OctahedronBound {
        size: f32,
    },
    Pyramid {
        height: f32,
    },
    HexPrism {
        bound: Vec2,
    },
    TriPrism {
        bound: Vec2,
    },
    Triangle {
        a: Vec3,
        b: Vec3,
        c: Vec3,
    },
    Bunny {
        s: f32,
    },
    MandelBulb {
        scale: f32,
        iter: f32,
        expo: f32,
        b_offset: f32,
    },
    JuliaQuaternion {
        scale: f32,
        iter: f32,
    },
    MengerSponge {
        scale: f32,
        iter: f32,
    },
}

#[repr(C)]
#[derive(ShaderType, Clone, Copy)]
pub struct SdShapeUniform {
    pub type_id_index_len: u32,
}

impl SdShape {
    #[inline]
    pub fn uniform(self, index: usize) -> SdShapeUniform {
        let self_bytes: [u8; 40] = unsafe { transmute(self) };
        let index_bytes: [u8; 2] = (index as u16).to_ne_bytes();
        let len_byte: u8 = self.gpu_field_count() as u8;

        unsafe { transmute([self_bytes[0], index_bytes[0], index_bytes[1], len_byte]) }
    }
}

#[derive(ShaderType, Default, Clone, Debug, Copy)]
pub struct SdModUniform {
    pub type_id: u32,
    pub data: Vec4,
}

#[repr(u32)]
#[derive(Reflect, Debug, Clone, Copy)]
#[reflect(Default)]
pub enum SdMod {
    Translate { t: Vec3 },
    OrthogonalRotateX,
    OrthogonalRotateY,
    OrthogonalRotateZ,
    RotateX { a: f32 },
    RotateY { a: f32 },
    RotateZ { a: f32 },
    RotateEuleur { a: Vec3 },
    Twist { k: f32 },
    CheapBend { k: f32 },
    SymetryX,
    SymetryY,
    SymetryZ,
    InfArray { c: Vec3 },
    LimArray { c: f32, lim: Vec3 },
    Elongate { h: Vec3 },
}

impl Default for SdMod {
    fn default() -> Self {
        Self::Translate { t: Vec3::ZERO }
    }
}

impl SdMod {
    #[inline]
    pub fn uniform(self) -> SdModUniform {
        let bytes: [u32; 5] = unsafe { transmute(self) };
        let pack: Vec4 = unsafe { transmute([bytes[1], bytes[2], bytes[3], bytes[4]]) };
        SdModUniform {
            type_id: bytes[0],
            data: pack,
        }
    }
}

#[derive(Reflect, Component, Default, Debug, Clone)]
#[reflect(Component, Default)]
pub struct SdModStack {
    pub modifiers: Vec<SdMod>,
}

#[derive(ShaderType, Clone, Copy)]
pub struct SdModStackUniform {
    pub data_index_and_lenght: u32,
}

impl SdModStack {
    #[inline]
    pub fn uniform(self, start_index: usize) -> SdModStackUniform {
        let start = start_index as u16 as u32;
        let len = self.modifiers.len() as u16 as u32;

        SdModStackUniform {
            data_index_and_lenght: (start << 16) | len,
        }
    }
}

#[derive(Reflect, ShaderType, Clone, Copy)]
pub struct SdMaterialUniform {
    pub color: u32,
    pub rough_fres_metal: u32,
    pub sss_strength_radius: u32,
}

#[derive(Component, Reflect, Debug, Clone, Copy)]
#[reflect(Component, Default)]
pub struct SdMaterial {
    pub color: Color,
    pub roughness: f32,
    pub fresnel: f32,
    pub metallic: f32,
    pub sss_strength: f32,
    pub sss_radius: Color,
}

impl Default for SdMaterial {
    fn default() -> Self {
        Self {
            color: LinearRgba::ZERO.into(),
            roughness: 0.5,
            fresnel: 0.,
            metallic: 0.,
            sss_strength: 0.,
            sss_radius: LinearRgba::ZERO.into(),
        }
    }
}

impl SdMaterial {
    #[inline]
    pub fn uniform(self) -> SdMaterialUniform {
        SdMaterialUniform {
            color: u32::from_ne_bytes(self.color.to_linear().to_u8_array()),
            rough_fres_metal: u32::from_ne_bytes(
                LinearRgba::new(self.roughness, self.fresnel, self.metallic, 0.).to_u8_array(),
            ),
            sss_strength_radius: u32::from_ne_bytes(self.sss_radius.to_linear().to_u8_array()),
        }
    }
}

impl From<StandardMaterial> for SdMaterial {
    fn from(source: StandardMaterial) -> Self {
        Self {
            color: source.base_color,
            roughness: source.perceptual_roughness,
            fresnel: 0.,
            metallic: source.metallic,
            sss_strength: 0.,
            sss_radius: LinearRgba::ZERO.into(),
        }
    }
}

#[derive(Reflect, Debug, Clone, Copy)]
pub struct SdTransform {
    pub pos: Vec3,
    pub rot: Vec3,
}

#[derive(ShaderType, Clone, Copy)]
pub struct SdTransformUniform {
    pub pos: Vec3,
    pub rot: Vec3,
}

impl SdTransform {
    pub fn uniform(self) -> SdTransformUniform {
        unsafe { transmute(self) }
    }
}
