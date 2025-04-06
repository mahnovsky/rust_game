use std::{cell::RefCell, rc::Rc};
use super::system_trait::System;
use crate::{
    bounds::Bounds, 
    collider2d::{Collider2d, CollisionEvent}, 
    object_components::{Bullet, BulletSpawner, Damagable, Gun, Lifetime, Movable}, 
    player_config::{PlayerController, PlayerState}, 
    quad_tree::QuadTree, 
    sprite::Sprite, 
    transform::Transform
};
pub struct MoveSystem {
    quad_tree: Rc<RefCell<QuadTree>>,
}

impl MoveSystem {
    pub fn new(quad_tree: Rc<RefCell<QuadTree>>) -> Self {
        Self { quad_tree }
    }
}

impl System for MoveSystem {
    fn update(&mut self, world: &ecs::EcsRc, delta: f32) {
        let ecs = world.borrow();

        ecs.visit_all3::<Transform, PlayerController, Movable>(|transform, controller, movable| {
            
            movable.set_dirty(controller.state == PlayerState::Move);

            let direction = controller.direction;

            if !transform.get_direction().eq(&direction) {
                transform.set_direction(&direction);
            }
        });

        ecs.visit_all2::<Transform, Movable>(|transform, movable| {
            if movable.is_dirty() {
                let pos = transform.get_position();
                let speed = movable.get_speed() * delta;
                let dir = transform.get_direction();
                let new_pos = pos + dir * speed;
                if let Some(entity) = transform.entity.upgrade() {
                    let mut quad_tree = self.quad_tree.borrow_mut();
                    let summary = quad_tree.move_object(&ecs, &entity, new_pos);
                    if summary.can_move {
                        transform.set_position(&new_pos);
                    } else if let Some(collide_ent) = summary.collide_ent {
                        //println!("Cant move help!!! {:?}", new_pos);
                        entity.push_event(CollisionEvent::OnEntity(collide_ent));
                    }
                }

                movable.set_dirty(false);
            }
        });
    }
}