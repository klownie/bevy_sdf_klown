use crate::engine::hierarchy::SdOperatedBy;
use bevy::ecs::lifecycle::HookContext;
use bevy::ecs::world::DeferredWorld;
use bevy::prelude::*;
use bevy::render::render_resource::ShaderType;

#[derive(Reflect, Debug, Clone, Copy)]
pub struct SdOperator {
    pub op: SdBlend,
    pub lhs: u16,
    pub rhs: u16,
}

#[derive(ShaderType, Clone, Copy)]
pub struct SdOperatorUniform {
    pub op: SdBlendUniform,
    pub lhs_rhs: u32,
}

impl SdOperator {
    pub fn uniform(self) -> SdOperatorUniform {
        // Pack lhs (lower 16 bits) and rhs (upper 16 bits) into u32
        let lhs_rhs = (self.lhs as u32) | ((self.rhs as u32) << 16);

        // Pack the operation into a u32
        let op = self.op.uniform();

        SdOperatorUniform { op, lhs_rhs }
    }
}

#[derive(ShaderType, Clone, Copy)]
pub struct SdBlendUniform {
    pub type_id_data: u32,
}

#[repr(u32)]
#[derive(Reflect, Component, Debug, Clone, Copy, Default)]
#[require(Name::new("SdOp"), SdIndex)]
#[reflect(Component)]
pub enum SdBlend {
    #[default]
    Union,
    Subtract {
        rev: bool,
    },
    Intersect,
    ChamferUnion {
        radius: f32,
    },
    ChamferSubtract {
        rev: bool,
        radius: f32,
    },
    ChamferIntersect {
        radius: f32,
    },
    SmoothUnion {
        k: f32,
    },
    SmoothSubtract {
        rev: bool,
        k: f32,
    },
    SmoothIntersect {
        k: f32,
    },
    Displace {
        rev: bool,
        strength: f32,
    },
}

impl SdBlend {
    pub fn uniform(self) -> SdBlendUniform {
        use SdBlend::*;
        let (disc, rev, extra): (u8, u8, u16) = match self {
            Union => (0, 0, 0),
            Subtract { rev } => (1, rev as u8, 0),
            Intersect => (2, 0, 0),
            ChamferUnion { radius } => (3, 0, (radius * 255.0) as u16), // approximate
            ChamferSubtract { rev, radius } => (4, rev as u8, (radius * 255.0) as u16),
            ChamferIntersect { radius } => (5, 0, (radius * 255.0) as u16),
            SmoothUnion { k } => (6, 0, (k * 255.0) as u16),
            SmoothSubtract { rev, k } => (7, rev as u8, (k * 255.0) as u16),
            SmoothIntersect { k } => (8, 0, (k * 255.0) as u16),
            Displace { rev, strength } => (9, rev as u8, (strength * 255.0) as u16),
        };

        let id_data = (disc as u32) | ((rev as u32) << 8) | ((extra as u32) << 16);

        SdBlendUniform {
            type_id_data: id_data,
        }
    }
}

#[derive(Reflect, Component, Ord, PartialOrd, PartialEq, Eq, Default, Debug, Clone, Copy)]
#[component(on_add = update_sd_index)]
#[reflect(Component)]
pub struct SdIndex(pub u32);

fn update_sd_index(mut world: DeferredWorld, HookContext { entity, .. }: HookContext) {
    let mut depth = 0;
    let mut current_entity = entity;

    while let Some(parent) = world.get::<SdOperatedBy>(current_entity) {
        depth += 1;
        current_entity = parent.0;
    }

    set_index(&mut world, entity, depth);
}

#[inline]
fn set_index(world: &mut DeferredWorld, entity: Entity, index: u32) {
    if let Some(mut sd_index) = world.get_mut::<SdIndex>(entity) {
        sd_index.0 = index;
    }
}
