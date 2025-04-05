use std::cell::RefCell;

use crate::{bounds::Bounds, collider2d::Collider2d, object_components::{Bullet, BulletSpawner, Damagable, Gun, Lifetime, Movable}, sprite::Sprite, transform::Transform};
use ecs::*;

pub struct FireSystem {
    spawners: RefCell<Vec<BulletSpawner>>,
}

impl FireSystem {
    pub fn new() -> Self {
        Self { spawners: RefCell::new(Vec::new()) }
    }

    pub fn update(&mut self, world: &EcsRc, delta: f32) {
        let ecs = world.borrow();
        
        ecs.visit_all::<Gun>(|gun|{
            gun.update_timer(delta);
            let owner = gun.entity.upgrade().unwrap();
            let tr_owner = owner.get_component_clone::<Transform>().unwrap();
            let direction = tr_owner.get_direction();
            let owner_id = owner.get_id();
            if let Some(spawner) = gun.consume_spawner() {
                let mut spawners = self.spawners.borrow_mut();
                spawners.push(spawner);
            }
        });
        drop(ecs);
        let mut spawners = self.spawners.borrow_mut();
        for spawner in spawners.iter() {
            let direction = glm::vec2(0., 0.);//tr_owner.get_direction();
            let weak_bullet = Entity::new(world);
            let bullet = weak_bullet.upgrade().unwrap();
            bullet.add_component(|| spawner.spawn_bullet(&weak_bullet));

            let pos = spawner.pos + spawner.dir * 30.;
            bullet.add_component(|| {
                Transform::with_direction(&weak_bullet, pos, spawner.dir)
            });
            bullet.add_component(|| Movable::new(&weak_bullet, 200.));
            bullet.add_component(|| {
                Collider2d::with_ignore(
                    &weak_bullet,
                    Bounds::with_center_position(pos.x, pos.y, 10., 10.),
                    vec![spawner.owner_id.clone()],
                )
            });
            bullet.add_component(|| Sprite::new(&weak_bullet, 10., 10., "tank1.png"));
            bullet.add_component(|| Damagable::new(&weak_bullet, 1));
            bullet.add_component(|| Lifetime::new(&weak_bullet, 2.));
        }
        spawners.clear();
    }
}