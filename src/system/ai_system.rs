use std::{cell::RefCell, rc::Rc};

use ecs::{Component, EcsRc, Entity, EntityId, EntityWeak};
use ecs_derive::component_impl;

use crate::{
    bounds::Bounds, collider2d::Collider2d, object_components::{Bullet, BulletSpawner, Damagable, Gun, Lifetime, Movable}, quad_tree::{self, QuadTree}, sprite::Sprite, transform::Transform
};
use super::system_trait::System;

enum AiCommand {
    ChangeDir,
    Shoot,
}

#[component_impl]
struct AiController {
    dir: glm::Vec2,
}

impl AiController {
    fn new(entity: &EntityWeak, dir: glm::Vec2) -> Self {
        Self { entity: entity.clone(), dir }
    }
}

#[derive(Clone, Copy)]
enum SpawnItem {
    Tank,
    Bonus
}

struct SpawnPoint {
    pos: glm::Vec2,
    spawn_delay: f32,
    timer: f32,
    spawn_item: SpawnItem,
    spawned: Option<SpawnItem>
}

impl SpawnPoint {
    fn new(pos: glm::Vec2) -> Self {
        SpawnPoint{ 
            pos,
            spawn_delay: 3.0, 
            timer: 0.0, 
            spawn_item: SpawnItem::Tank,
            spawned: None
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
    quad_tree: Rc<RefCell<QuadTree>>,
    spawn_points: Vec<SpawnPoint>,
}

impl AiSystem {
    pub fn new(quad_tree: Rc<RefCell<QuadTree>>) -> Self {
        let spawn_points= vec![ 
            SpawnPoint::new(glm::vec2(50., 700.)),
            SpawnPoint::new(glm::vec2(980., 700.)),
        ];
        Self{ 
            quad_tree,
            spawn_points 
        }
    }

    fn spawn_tank(&mut self, world: &ecs::EcsRc, pos: glm::Vec2) {

        let dir = glm::vec2(0_f32, 1_f32);
        let size = 50_f32;
        let bounds = Bounds::with_center_position(pos.x, pos.y, size, size);
        let quad_tree = self.quad_tree.borrow_mut();
        if !quad_tree.can_place(world.borrow(), &bounds) {
            println!("AI Tank failed spawn");
            return;
        }
        drop(quad_tree);

        let entity_weak = Entity::new(world);
        let entity = entity_weak.upgrade().unwrap();

        entity.add_component(|| Sprite::new(&entity_weak, size, size, "tank.png"));
        entity.add_component(|| Transform::with_direction(&entity_weak, pos, dir));
        entity.add_component(|| Collider2d::new(&entity_weak, bounds));
        entity.add_component(|| Movable::new(&entity_weak, 200.));
        entity.add_component(|| Damagable::new(&entity_weak, 10));
        entity.add_component(|| AiController::new(&entity_weak, dir));
        entity.add_component(|| Gun::new(&entity_weak, 2));


        let quad_tree = self.quad_tree.borrow_mut();
        quad_tree.place(world.borrow(), &entity);

        println!("On AI Tank spawned");
    }

    fn spawn_bonus(&mut self, world: &ecs::EcsRc) {

    }

    fn spawn_items(&mut self, world: &EcsRc, delta: f32) {
        let mut spawn_items = Vec::<(SpawnItem, glm::Vec2)>::new();
        for sp in self.spawn_points.iter_mut() {
            sp.update(delta);

            if let Some(spawn_item) = sp.consume_spawn() {
                spawn_items.push((spawn_item, sp.pos));
            }
        }  

        while let Some(item) = spawn_items.pop() {
            match item.0 {
                SpawnItem::Tank => self.spawn_tank(world, item.1),
                SpawnItem::Bonus => self.spawn_bonus(world),
            }
        }
    }
}

impl System for AiSystem {
    fn update(&mut self, world: &ecs::EcsRc, delta: f32) {
        
        self.spawn_items(world, delta);
    }
}