use bevy::{
    ecs::{component::HookContext, world::DeferredWorld},
    prelude::*,
};

use super::op::SdOp;

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

#[macro_export]
macro_rules! op_patients {
    // Match exactly two patients
    [ $a:expr, $b:expr $(,)? ] => {
        $crate::engine::hierarchy::SdOperatingOn::spawn((
            bevy::ecs::spawn::Spawn($a),
            bevy::ecs::spawn::Spawn($b),
        ))
    };

    // Match anything else (fallback error)
    [ $($anything:tt)* ] => {
        compile_error!("SdOp can only opereate on tow patients")
    };
}

// NOTE: A helper for skein that will automatically setup the SDF relationships
// This must be added to every SdOp and SdShape in the skein scene
#[derive(Component, Reflect)]
#[reflect(Component)]
#[component(on_add = detect_op)]
pub struct InitSkeinSdRelationShip;

fn detect_op(mut world: DeferredWorld, HookContext { entity, .. }: HookContext) {
    {
        let mut commands = world.commands();
        commands.entity(entity).remove::<InitSkeinSdRelationShip>();
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
