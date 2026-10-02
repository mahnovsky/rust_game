use legion::*;
use std::cell::RefCell;

use crate::{
    bounds::Bounds,
    collider2d::Collider2d,
    object_components::{Bullet, BulletSpawner, Damagable, Gun, Lifetime, Movable},
    sprite::Sprite,
    transform::Transform,
};

pub struct FireSystem {
    spawners: Vec<BulletSpawner>,
}

impl FireSystem {
    pub fn new() -> Self {
        Self {
            spawners: Vec::new(),
        }
    }

    fn update_guns(&mut self, world: &mut World, delta: f32) {
        let mut query = <(Entity, &mut Gun, &Transform)>::query();
        for (entity, gun, transform) in query.iter_mut(world) {
            gun.update_timer(delta);
            let Some(spawner) = gun.consume_spawner() else {
                continue;
            };
            self.spawners.push(spawner);

            let direction = transform.get_direction();
        }
    }

    fn spawn_bullets(&mut self, world: &mut World) {
        while let Some(spawner) = self.spawners.pop() {
            let direction = glm::vec2(0., 0.); //tr_owner.get_direction();
            let pos = spawner.pos + spawner.dir * 30.;
            world.push((
                Bullet::new(spawner.owner_id, spawner.damage),
                Transform::with_direction(pos, spawner.dir),
                Movable::new(300, crate::object_components::Direction::Down),
                Collider2d::with_ignore(
                    Bounds::with_center_position(pos.x, pos.y, 10., 10.),
                    vec![spawner.owner_id],
                ),
                Sprite::new(10., 10., "tank1.png"),
                Damagable::new(1),
                Lifetime::new(5.),
            ));
        }
    }

    fn apply_damage(&self, world: &World) {
        /*static mut DAMAGE_TARGETS: Vec<(u32, Entity)> = Vec::new();
        ecs.visit_all::<Bullet>(|bullet| {
            if let Some(target) = bullet.consume_target() {
                unsafe { DAMAGE_TARGETS.push((bullet.get_damage(), target)) };
                bullet.entity.upgrade().unwrap().kill();
            }
        });

        unsafe {
            while let Some(info) = DAMAGE_TARGETS.pop() {
                ecs.visit::<Damagable>(info.1, |damagable| {
                    if let Some(damagable) = damagable {
                        damagable.do_damage(info.0);
                    }
                });
            }
        }*/
    }
}

pub fn update(fs: &mut FireSystem, world: &mut World, delta: f32) {
    fs.update_guns(world, delta);

    fs.spawn_bullets(world);

    fs.apply_damage(world);
}
