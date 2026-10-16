use std::collections;

use fxhash::{FxBuildHasher, FxHashMap};
use legion::{systems::CommandBuffer, *};

use crate::{
    bounds::Bounds,
    collider2d::{Collider2d, CollisionEvent},
    object_components::{Bullet, Damagable, Gun, Lifetime, Movable, MoveSwitch},
    quad_tree::QuadTree,
    sprite::Sprite,
    transform::Transform,
};

fn update_bullet_spawns(
    world: &mut World,
    quad_tree: &mut QuadTree,
    cb: &mut CommandBuffer,
    delta: f32,
) {
    let mut query = <(Entity, &mut Gun, &Transform, &Movable)>::query();
    for (entity, gun, transform, mv) in query.iter_mut(world) {
        gun.update_timer(delta);
        if let Some(spawner) = gun.consume_spawner() {
            let dir = spawner.dir;
            let pos = spawner.pos + spawner.dir.to_vec() * 30.;
            let mut collider = Collider2d::with_ignore(
                Bounds::with_center_position(pos.x, pos.y, 10., 10.),
                vec![spawner.owner_id],
            );
            if quad_tree.place(&mut collider) {
                let bullet_ent = cb.push((
                    Bullet::new(spawner.owner_id, spawner.damage),
                    Transform::with_direction(pos, dir.to_vec()),
                    Movable::new(300, dir, MoveSwitch::Forward),
                    collider,
                    Sprite::new(10., 10., "tank1.png"),
                    Damagable::new(1),
                    Lifetime::new(5.),
                ));
            }
        }
    }
}

fn update_bullet_collisions(
    world: &mut World,
    quad_tree: &mut QuadTree,
    cb: &mut CommandBuffer,
    delta: f32,
    collisions: &[CollisionEvent],
) {
    if collisions.is_empty() {
        return;
    }

    let mut query = <(Entity, &Collider2d, &Bullet)>::query();
    let mut ents = FxHashMap::with_hasher(FxBuildHasher::new());

    for (entity, collider, bullet) in query.iter(world) {
        let Some(event) = collisions.iter().find(|c| c.entity == *entity) else {
            continue;
        };

        cb.remove(*entity);

        if event.reached_border {
            continue;
        }

        let mut q = <(Entity, &Collider2d)>::query().filter(component::<Damagable>());
        if let Some(collided_entity) = q
            .iter(world)
            .find(|(e, c)| event.collide_with.contains(&c.get_tree_id()))
            .map(|(e, c)| *e)
        {
            ents.insert(collided_entity, bullet.clone());
        }
    }

    let mut query = <(Entity, &mut Damagable)>::query();
    for (ent, d) in query.iter_mut(world) {
        if let Some(b) = ents.get(ent) {
            d.do_damage(b.get_damage());
        }
    }
}

pub fn update(
    world: &mut World,
    quad_tree: &mut QuadTree,
    cb: &mut CommandBuffer,
    delta: f32,
    collisions: &[CollisionEvent],
) {
    update_bullet_spawns(world, quad_tree, cb, delta);

    update_bullet_collisions(world, quad_tree, cb, delta, collisions);
}
