//use super::system_trait::System;
use crate::{
    bounds::Bounds,
    collider2d::{Collider2d, ColliderRegisterComponent},
    game::CollisionFlags,
    object_components::{Damagable, Direction, Gun, Movable, MoveSwitch::Forward},
    quad_tree::QuadTree,
    sprite::Sprite,
    transform::Transform,
};
use legion::*;
use rand::{rngs::ThreadRng, RngExt};

enum AiCommand {
    ChangeDir,
    Shoot,
}
pub struct AiController {
    update_time: f32,
    new_direction: Option<Direction>,
}

impl AiController {
    pub fn new() -> Self {
        Self {
            update_time: 1.0,
            new_direction: None,
        }
    }

    pub fn consume_direction(&mut self) -> Option<Direction> {
        self.new_direction.take()
    }

    pub fn update_remaining(&mut self, dt: f32, rng: &mut ThreadRng, prev_dir: Direction) {
        self.update_time -= dt;
        if self.update_time <= 0_f32 {
            self.update_time = rng.random_range::<f32, _>(1.0..3.0);
            self.new_direction = Some(prev_dir.get_rng(rng));
        }
    }
}

#[derive(Clone, Copy)]
enum SpawnItem {
    Tank,
    Bonus,
}

struct SpawnPoint {
    pos: glm::Vec2,
    spawn_delay: f32,
    timer: f32,
    spawn_item: SpawnItem,
    spawned: Option<SpawnItem>,
}

impl SpawnPoint {
    fn new(pos: glm::Vec2) -> Self {
        SpawnPoint {
            pos,
            spawn_delay: 3.0,
            timer: 0.0,
            spawn_item: SpawnItem::Tank,
            spawned: None,
        }
    }

    fn update(&mut self, delta: f32) {
        if self.timer < self.spawn_delay && self.spawned.is_none() {
            self.timer += delta;
        } else {
            self.spawned = self.spawn_item.into();
            self.timer = 0.;
        }
    }

    fn consume_spawn(&mut self) -> Option<SpawnItem> {
        self.spawned.take()
    }
}

pub struct AiSystem {
    spawn_points: Vec<SpawnPoint>,
}

impl AiSystem {
    pub fn new() -> Self {
        let spawn_points = vec![
            SpawnPoint::new(glm::vec2(50., 700.)),
            SpawnPoint::new(glm::vec2(980., 700.)),
        ];
        Self { spawn_points }
    }

    fn spawn_tank(&mut self, world: &mut World, quad_tree: &mut QuadTree, pos: glm::Vec2) {
        let dir = glm::vec2(0_f32, 1_f32);
        let size = 50_f32;
        let bounds = Bounds::with_center_position(pos.x, pos.y, size, size);

        if !quad_tree.can_place(world, &bounds) {
            return;
        }
        let flags =
            CollisionFlags::AiTank | CollisionFlags::PlayerTank | CollisionFlags::PlayerBullet;
        let entity = world.push((
            Sprite::new(size, size, "tank.png"),
            Transform::with_direction(pos, dir),
            ColliderRegisterComponent,
            Collider2d::new(bounds, flags.bits()),
            Movable::new(200, Direction::Down, Forward),
            Damagable::new(10),
            AiController::new(),
            Gun::new(2),
        ));

        println!("On AI Tank spawned");
    }

    fn spawn_bonus(&mut self, world: &mut World) {}

    fn spawn_items(&mut self, world: &mut World, quad_tree: &mut QuadTree, delta: f32) {
        let mut spawn_items = Vec::<(SpawnItem, glm::Vec2)>::new();
        for sp in self.spawn_points.iter_mut() {
            sp.update(delta);

            if let Some(spawn_item) = sp.consume_spawn() {
                spawn_items.push((spawn_item, sp.pos));
            }
        }

        while let Some(item) = spawn_items.pop() {
            match item.0 {
                SpawnItem::Tank => self.spawn_tank(world, quad_tree, item.1),
                SpawnItem::Bonus => self.spawn_bonus(world),
            }
        }
    }
}

pub fn update(
    ai: &mut AiSystem,
    world: &mut World,
    quad_tree: &mut QuadTree,
    rng: &mut ThreadRng,
    delta: f32,
) {
    ai.spawn_items(world, quad_tree, delta);

    let mut query = <(
        Entity,
        &mut AiController,
        &mut Movable,
        &mut Gun,
        &Transform,
    )>::query();
    for (ent, ai, mv, g, tr) in query.iter_mut(world) {
        ai.update_remaining(delta, rng, mv.get_direction());

        if let Some(new_dir) = ai.consume_direction() {
            mv.set_direction(new_dir);
        }
        if g.can_spawn_bullet() {
            g.setup_shoot(*ent, tr.get_position(), mv.get_direction());
        }
    }
}
