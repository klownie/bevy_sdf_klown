#![feature(stmt_expr_attributes)]

pub mod engine;
use crate::engine::RayMarchEnginePlugin;
use bevy::prelude::*;
pub struct RayMarchingPlugin;

impl Plugin for RayMarchingPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins(RayMarchEnginePlugin);
    }
}
