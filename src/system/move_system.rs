use crate::{
    bounds::Bounds,
    collider2d::{Collider2d, CollisionEvent},
    object_components::{Bullet, BulletSpawner, Damagable, Gun, Lifetime, Movable},
    player_config::{PlayerController, PlayerState},
    quad_tree::{QuadTree, TreeId},
    sprite::Sprite,
    transform::Transform,
};
use legion::*;
use std::{cell::RefCell, rc::Rc};
pub struct MoveSystem {
    quad_tree: Rc<RefCell<QuadTree>>,
}

impl MoveSystem {
    pub fn new(quad_tree: Rc<RefCell<QuadTree>>) -> Self {
        Self { quad_tree }
    }
}

pub fn move_update(world: &mut World, quad_tree: &mut QuadTree, delta: f32) -> Vec<TreeId> {
    let mut query = <(Entity, &Movable, &mut Transform, &mut Collider2d)>::query();
    let mut collisions = Vec::new();
    for (entity, movable, transform, collider) in query.iter_mut(world) {
        let move_delta = movable.get_move_fragment(delta);

        let pos = transform.get_position();
        let new_pos = pos + move_delta;

        let summary = quad_tree.move_object(*entity, new_pos, collider);

        if summary.can_move {
            transform.set_position(&new_pos);
        }
        transform.set_direction(movable.get_direction_vec());

        if !summary.collisions.is_empty() {
            collisions.extend(summary.collisions);
        }
    }

    collisions
}
