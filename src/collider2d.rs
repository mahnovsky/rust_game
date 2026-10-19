use std::str;

use crate::{
    bounds::Bounds,
    quad_tree::{CollisionObjectId, Handle},
};
use legion::*;

#[derive(Debug, Clone)]
pub struct CollisionInfo {
    pub entity: Entity,
    pub collide_with: Vec<Entity>,
}

#[derive(Debug, Clone)]
pub enum CollisionEvent {
    Object(CollisionInfo),
    Bullet(CollisionInfo),
    EdgeCrossing(Entity),
}

pub struct ColliderRegisterComponent;

#[derive(Debug, Clone, Default)]
pub struct Collider2d {
    tree_id: CollisionObjectId,
    bounds: Bounds,
    area_handle: Option<Handle>,
    flags: u32,
}

impl Collider2d {
    pub fn new(bounds: Bounds, flags: u32) -> Self {
        Self {
            tree_id: CollisionObjectId::INVALID,
            bounds,
            area_handle: None,
            flags,
        }
    }

    pub fn set_position(&mut self, x: f32, y: f32) {
        self.bounds.set_center_position(x, y);
    }

    pub fn get_bounds(&self) -> Bounds {
        self.bounds
    }

    pub fn set_area_handle(&mut self, handle: Option<Handle>) {
        self.area_handle = handle;
    }

    pub fn get_area_handle(&self) -> Option<Handle> {
        self.area_handle
    }

    pub fn get_tree_id(&self) -> CollisionObjectId {
        self.tree_id
    }

    pub fn set_tree_id(&mut self, tree_id: CollisionObjectId) {
        self.tree_id = tree_id;
    }
}
