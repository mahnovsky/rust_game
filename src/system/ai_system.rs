//use super::system_trait::System;
use crate::{
    bounds::Bounds,
    collider2d::Collider2d,
    object_components::{Bullet, BulletSpawner, Damagable, Gun, Lifetime, Movable},
    quad_tree::{self, QuadTree},
    sprite::Sprite,
    transform::Transform,
};
use legion::*;
use std::{cell::RefCell, rc::Rc};

enum AiCommand {
    ChangeDir,
    Shoot,
}

struct AiController {
    dir: glm::Vec2,
}

impl AiController {
    fn new(dir: glm::Vec2) -> Self {
        Self { dir }
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
    //quad_tree: Rc<RefCell<QuadTree>>,
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

        if quad_tree.can_place(world, &bounds) {
            let entity = world.push((
                Sprite::new(size, size, "tank.png"),
                Transform::with_direction(pos, dir),
                Collider2d::new(bounds),
                Movable::new(200, crate::object_components::Direction::Down),
                Damagable::new(10),
                AiController::new(dir),
                Gun::new(2),
            ));

            quad_tree.place(world, entity);
        }

        println!("On AI Tank spawned");
    }

    fn spawn_bonus(&mut self, world: &mut World) {}

    fn spawn_items(&mut self, world: &mut World, delta: f32) {
        let mut spawn_items = Vec::<(SpawnItem, glm::Vec2)>::new();
        for sp in self.spawn_points.iter_mut() {
            sp.update(delta);

            if let Some(spawn_item) = sp.consume_spawn() {
                spawn_items.push((spawn_item, sp.pos));
            }
        }

        while let Some(item) = spawn_items.pop() {
            match item.0 {
                SpawnItem::Tank => {} //self.spawn_tank(world, item.1),
                SpawnItem::Bonus => self.spawn_bonus(world),
            }
        }
    }
}

fn update(ai: &mut AiSystem, world: &mut World, delta: f32) {
    ai.spawn_items(world, delta);
}
