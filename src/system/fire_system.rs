use legion::{systems::CommandBuffer, *};

use crate::{
    bounds::Bounds,
    collider2d::Collider2d,
    object_components::{Bullet, Damagable, Gun, Lifetime, Movable, MoveSwitch},
    sprite::Sprite,
    transform::Transform,
};

fn update_bullet_spawns(world: &mut World, cb: &mut CommandBuffer, delta: f32) {
    let mut query = <(Entity, &mut Gun, &Transform, &Movable)>::query();
    for (entity, gun, transform, mv) in query.iter_mut(world) {
        gun.update_timer(delta);
        if let Some(spawner) = gun.consume_spawner() {
            let dir = spawner.dir;
            let pos = spawner.pos + spawner.dir.to_vec() * 30.;
            cb.push((
                Bullet::new(spawner.owner_id, spawner.damage),
                Transform::with_direction(pos, dir.to_vec()),
                Movable::new(300, dir, MoveSwitch::Forward),
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
}

fn update_bullet_collisions(world: &mut World, cb: &mut CommandBuffer, delta: f32) {
    let mut query = <(Entity, &Collider2d, &Bullet, &Lifetime)>::query();
    for (entity, collider, bullet, lt) in query.iter(world) {
        if collider.is_reached_border() || lt.is_time_out() {
            cb.remove(*entity);
        }
    }
}

pub fn update(world: &mut World, cb: &mut CommandBuffer, delta: f32) {
    update_bullet_spawns(world, cb, delta);

    update_bullet_collisions(world, cb, delta);
}
