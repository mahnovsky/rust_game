use crate::bounds::Bounds;
use crate::quad_tree::Handle;
use ::ecs::*;
use ecs_derive::component_impl;

#[component_impl]
#[derive(Debug, Clone)]
pub struct Collider2d {
    bounds: Bounds,
    area_handle: Option<Handle>,
    reached_border: bool,
    collision_ignore: Option<Box<[EntityId]>>,
}

impl Collider2d {
    pub fn new(entity: &EntityWeak, bounds: Bounds) -> Self {
        Self {
            entity: entity.clone(),
            bounds,
            area_handle: None,
            reached_border: false,
            collision_ignore: None,
        }
    }

    pub fn with_ignore(entity: &EntityWeak, bounds: Bounds, ignores: Vec<EntityId>) -> Self {
        Self {
            entity: entity.clone(),
            bounds,
            area_handle: None,
            reached_border: false,
            collision_ignore: Some(ignores.into_boxed_slice()),
        }
    }

    pub fn is_ignored_entity(&self, id: EntityId) -> bool {
        if let Some(ignores) = &self.collision_ignore {
            return ignores.contains(&id);
        }
        false
    }

    pub fn set_position(&mut self, x: f32, y: f32) {
        self.bounds.set_center_position(x, y);
    }

    pub fn get_bounds(&self) -> Bounds {
        self.bounds.clone()
    }

    pub fn is_reached_border(&self) -> bool {
        self.reached_border
    }

    pub fn set_reached_border(&mut self, reached: bool) {
        self.reached_border = reached;
    }

    pub fn set_area_handle(&mut self, handle: Option<Handle>) {
        self.area_handle = handle;
    }

    pub fn get_area_handle(&self) -> Option<Handle> {
        self.area_handle
    }
}
