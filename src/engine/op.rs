use crate::engine::SdIndex;
use bevy::prelude::*;
use bevy::render::render_resource::ShaderType;
use std::mem::transmute;

#[derive(Reflect, Debug, Clone, Copy)]
pub struct SdOpInstance {
    pub op: SdOp,
    pub lhs: u16,
    pub rhs: u16,
}

#[derive(ShaderType, Clone, Copy)]
pub struct SdOpUniformInstance {
    pub op: SdOpUniform,
    pub lhs_rhs: u32,
}

impl SdOpInstance {
    pub fn uniform(self) -> SdOpUniformInstance {
        unsafe { transmute(self) }
    }
}

#[derive(ShaderType, Clone, Copy)]
pub struct SdOpUniform {
    pub id_rev: u32,
    pub data: f32,
}

#[repr(u8)]
#[derive(Reflect, Component, Debug, Clone, Copy)]
#[require(SdIndex)]
#[reflect(Component)]
pub enum SdOp {
    Union,
    Subtract {
        rev: bool,
    },
    Intersect,
    ChamferUnion {
        #[reflect(ignore)]
        pad: [u8; 3],
        radius: f32,
    },
    ChamferSubtract {
        rev: bool,
        #[reflect(ignore)]
        pad: u16,
        radius: f32,
    },
    ChamferIntersect {
        #[reflect(ignore)]
        pad: [u8; 3],
        radius: f32,
    },
    SmoothUnion {
        #[reflect(ignore)]
        pad: [u8; 3],
        k: f32,
    },
    SmoothSubtract {
        rev: bool,
        #[reflect(ignore)]
        pad: u16,
        k: f32,
    },
    SmoothIntersect {
        #[reflect(ignore)]
        pad: [u8; 3],
        k: f32,
    },
    Displace {
        rev: bool,
        #[reflect(ignore)]
        pad: u16,
        strength: f32,
    },
}
