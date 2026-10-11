use crate::{
    bounds::Bounds,
    collider2d::{Collider2d, CollisionEvent},
    object_components::{Bullet, BulletSpawner, Damagable, Gun, Lifetime, Movable},
    player_config::{PlayerController, PlayerState},
    quad_tree::{CollisionObjectId, QuadTree},
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

pub fn move_update(
    world: &mut World,
    quad_tree: &mut QuadTree,
    delta: f32,
    collisions: &mut Vec<CollisionEvent>,
) {
    let mut query = <(Entity, &Movable, &mut Transform, &mut Collider2d)>::query();
    for (entity, movable, transform, collider) in query.iter_mut(world) {
        let move_delta = movable.get_move_fragment(delta);

        let pos = transform.get_position();
        let new_pos = pos + move_delta;

        let move_result = quad_tree.move_object(*entity, new_pos, collider);

        if move_result.can_move {
            transform.set_position(&new_pos);
        }
        transform.set_direction(movable.get_direction_vec());

        if !move_result.collisions.is_empty() {
            collisions.push(CollisionEvent {
                entity: *entity,
                collide_with: move_result.collisions,
            });
        }
    }
}
