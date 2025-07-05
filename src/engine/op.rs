use crate::engine::SdIndex;
use bevy::ecs::component::HookContext;
use bevy::ecs::world::DeferredWorld;
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

#[derive(Component)]
#[relationship(relationship_target = SdOperatingOn)]
pub struct SdOperatedBy(pub Entity);

#[derive(Component, Clone)]
#[relationship_target(relationship = SdOperatedBy)]
pub struct SdOperatingOn(Vec<Entity>);

impl SdOperatingOn {
    #[inline]
    pub fn get_sd_argunments(self) -> (Entity, Entity) {
        (self.0[0], self.0[1])
    }
}

//NOTE: A helper for skein that will automatically setup the SDF relationships
// This must be added to every SdOp and SdShape in the skein scene
#[derive(Component, Reflect)]
#[reflect(Component)]
#[component(on_add = detect_op)]
pub struct InitSkeinSdRelatinShip;

fn detect_op(mut world: DeferredWorld, HookContext { entity, .. }: HookContext) {
    {
        let mut command = world.commands();
        command.entity(entity).remove::<InitSkeinSdRelatinShip>();
    }

    let parent = match world.get::<ChildOf>(entity) {
        Some(p) => p.parent(),
        None => return,
    };
    match world.get::<SdOp>(parent) {
        Some(_sd_op) => {
            let mut command = world.commands();
            command.entity(entity).insert(SdOperatedBy(parent));
            return;
        }
        None => (),
    }

    let grand_parent = match world.get::<ChildOf>(parent) {
        Some(p) => p.parent(),
        None => return,
    };
    match world.get::<SdOp>(grand_parent) {
        Some(_sd_op) => {
            let mut command = world.commands();
            command.entity(entity).insert(SdOperatedBy(grand_parent));
        }
        None => (),
    }
}
