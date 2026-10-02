use crate::{
    bounds::Bounds,
    collider2d::{Collider2d, CollisionEvent},
    object_components::{Bullet, BulletSpawner, Damagable, Gun, Lifetime, Movable},
    player_config::{PlayerController, PlayerState},
    quad_tree::QuadTree,
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

pub fn move_update(world: &mut World, quad_tree: &mut QuadTree, delta: f32) {
    let mut query = <(Entity, &Movable, &mut Transform)>::query();

    //let mut move_list = Vec::<(&mut Transform, glm::Vec2)>::new();

    for (entity, movable, transform) in query.iter_mut(world) {
        let move_delta = movable.get_move_fragment(delta);

        let pos = transform.get_position();
        let new_pos = pos + move_delta;

        transform.set_direction(movable.get_direction_vec());
        quad_tree.move_object(*entity, new_pos);
        //println!("move system update {:?}", movable.get_direction_vec());
        //let Some(summary) = quad_tree.move_object(&world, *entity, new_pos) else {
        //    return;
        //};

        //if summary.can_move {
        //    transform.set_position(&new_pos);
        //} else if let Some(collide_ent) = summary.collide_ent {
        //    //println!("Cant move help!!! {:?}", new_pos);
        //    // TODO: entity.push_event(CollisionEvent::OnEntity(collide_ent));
        //}
    }
}
