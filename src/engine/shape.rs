use bevy::prelude::*;
use std::mem::transmute;

use bevy::render::render_resource::ShaderType;

#[derive(ShaderType, Clone, Copy)]
pub struct SdShapeUniformInstance {
    pub shape: SdShapeUniform,
    pub material: SdMaterialUniform,
    pub modifier: SdModUniform,
    pub transform: SdTransformUniform,
}

#[derive(Reflect, Debug, Clone, Copy)]
pub struct SdShapeInstance {
    pub shape: SdShape,
    pub material: SdMaterial,
    pub modifier: SdMod,
    pub transform: SdTransform,
}

#[derive(ShaderType, Clone, Copy)]
pub struct SdShapeUniform {
    pub id: u32,
    pub data: Mat3,
}

#[repr(u32)]
#[derive(Reflect, Component, Debug, Clone, Copy)]
#[require(Name::new("SdShape"), SdMod::Empty, Transform)]
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

impl SdShape {
    pub fn uniform(self) -> SdShapeUniform {
        unsafe { transmute(self) }
    }
}

#[derive(ShaderType, Clone, Copy)]
pub struct SdModUniform {
    pub id: u32,
    pub data: Vec4,
}

#[repr(u32)]
#[derive(Reflect, Component, Debug, Clone, Copy)]
#[reflect(Component)]
pub enum SdMod {
    Twist { k: f32 },
    CheapBend { k: f32 },
    SymetryX,
    SymetryY,
    SymetryZ,
    InfArray { c: Vec3 },
    LimArray { c: f32, lim: Vec3 },
    Elongate { h: Vec3 },
    Empty,
}

impl SdMod {
    pub fn uniform(self) -> SdModUniform {
        let bytes: [u32; 5] = unsafe { transmute(self) };
        let pack: Vec4 = unsafe { transmute([bytes[1], bytes[2], bytes[3], bytes[4]]) };
        SdModUniform {
            id: bytes[0],
            data: pack,
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
    pub color: Vec4,
    pub roughness: f32,
    pub fresnel: f32,
    pub metallic: f32,
    pub sss_strength: f32,
    pub sss_radius: Vec3,
}

impl Default for SdMaterial {
    fn default() -> Self {
        Self {
            color: Vec4::W,
            roughness: 0.5,
            fresnel: 0.,
            metallic: 0.,
            sss_strength: 0.,
            sss_radius: Vec3::ONE,
        }
    }
}

impl SdMaterial {
    pub fn uniform(self) -> SdMaterialUniform {
        SdMaterialUniform {
            color: u32::from_ne_bytes(LinearRgba::from_vec4(self.color).to_u8_array()),
            rough_fres_metal: u32::from_ne_bytes(
                LinearRgba::new(self.roughness, self.fresnel, self.metallic, 0.).to_u8_array(),
            ),
            sss_strength_radius: u32::from_ne_bytes(
                LinearRgba::from_vec4(Vec4::new(
                    self.sss_strength,
                    self.sss_radius.x,
                    self.sss_radius.y,
                    self.sss_radius.z,
                ))
                .to_u8_array(),
            ),
        }
    }
}

impl From<StandardMaterial> for SdMaterial {
    fn from(source: StandardMaterial) -> Self {
        Self {
            color: source.base_color.to_linear().to_vec4(),
            roughness: source.perceptual_roughness,
            fresnel: 0.,
            metallic: source.metallic,
            sss_strength: 0.,
            sss_radius: Vec3::ZERO,
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
