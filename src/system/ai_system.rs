use std::{cell::{Cell, RefCell}, rc::Rc};

use ecs::{Component, EcsRc, Entity, EntityId, EntityWeak, Listener};
use ecs_derive::component_impl;
use rand::{rng, rngs::ThreadRng, Rng};
use strum::IntoEnumIterator;
use strum_macros::EnumIter;

use crate::{
    bounds::Bounds, collider2d::{Collider2d, CollisionEvent}, object_components::{Bullet, BulletSpawner, Damagable, Gun, Lifetime, Movable}, player_config::PlayerAction, quad_tree::{self, QuadTree}, sprite::Sprite, transform::Transform
};
use super::system_trait::System;

#[derive(Debug, EnumIter, PartialEq)]
enum Orient {
    North, // ^
    South, // \/
    West, // <-
    East, // ->
}

impl Orient {
    fn as_vec2(&self) -> glm::Vec2 {
        match self {
            Self::North => glm::vec2(0., 1.),
            Self::South => glm::vec2(0., -1.),
            Self::West => glm::vec2(-1., 0.),
            Self::East => glm::vec2(1., 0.),
        }
    }

    fn random_change(&mut self, rng: &mut ThreadRng) {
        
        let select = rng.random_range(0..3);
        let mut current = 0;
        for dir in Orient::iter() {
            if dir == *self {
                continue;
            }

            if current == select {
                *self = dir;
            }
            current += 1;
        }
        
    }
}

#[component_impl]
struct AiController {
    dir: Orient,
    rng: ThreadRng,
    change_dir: f32,
    timer: f32,
}

impl AiController {
    fn new(entity: &EntityWeak, dir: Orient) -> Self {
        Self { entity: entity.clone(), dir, rng: rng(), change_dir: 0.1, timer: 0. }
    }

    fn change_direction(&mut self) {
        self.dir.random_change(&mut self.rng);
    }

    fn update(&mut self, delta: f32) {
        self.timer += delta;
        if self.timer > self.change_dir {
            self.change_dir = self.rng.random_range(1.1..3.0);
            self.change_direction();
            self.timer = 0.;
        }
    }
}

impl Listener<CollisionEvent> for AiController {
    fn on_event(&mut self, event: CollisionEvent) {
        match event {
            CollisionEvent::OnEntity(_) => self.change_direction(),
            CollisionEvent::OnBorderReached => self.change_direction(),
        }
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
    ai_tanks: u32,
}

impl AiSystem {
    pub fn new(quad_tree: Rc<RefCell<QuadTree>>) -> Self {
        let spawn_points= vec![ 
            SpawnPoint::new(glm::vec2(50., 700.)),
            SpawnPoint::new(glm::vec2(980., 700.)),
        ];
        Self{ 
            quad_tree,
            spawn_points,
            ai_tanks: 0
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

        entity.add_component(|| Sprite::new(&entity_weak, size, size, "tank1.png"));
        entity.add_component(|| Transform::with_direction(&entity_weak, pos, dir));
        entity.add_component(|| Collider2d::new(&entity_weak, bounds));
        entity.add_component(|| Movable::new(&entity_weak, 100.));
        entity.add_component(|| Damagable::new(&entity_weak, 10));
        entity.add_component(|| AiController::new(&entity_weak, Orient::North));
        entity.add_component(|| Gun::with_auto_shoot(&entity_weak, 2, 0.8));


        let quad_tree = self.quad_tree.borrow_mut();
        quad_tree.place(world.borrow(), &entity);

        println!("On AI Tank spawned");
    }

    fn spawn_bonus(&mut self, world: &ecs::EcsRc) {

    }

    fn spawn_items(&mut self, world: &EcsRc, delta: f32) {
        if self.ai_tanks > 10 {
            return;
        }

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

    fn update_ai(&mut self, world: &EcsRc, delta: f32) {
        let ecs = world.borrow();
        if !ecs.is_componet_exist::<AiController>() {
            return;
        }
        let tanks = Cell::<u32>::new(0);
        ecs.visit_all3::<AiController, Transform, Movable>(|controller, transform, movable|{
            controller.update(delta);
            if let Some(entity) = controller.entity.upgrade() {
                entity.process_event_for_listener::<CollisionEvent>(controller);
                entity.clean_events::<CollisionEvent>();
            }
            let direction = controller.dir.as_vec2();

            if !transform.get_direction().eq(&direction) {
                transform.set_direction(&direction);
            }
            movable.set_dirty(true);
            tanks.set(tanks.get() + 1);
        });
        self.ai_tanks = tanks.get();
    }
}

impl System for AiSystem {  
    fn update(&mut self, world: &ecs::EcsRc, delta: f32) {
        
        self.spawn_items(world, delta);
        self.update_ai(world, delta);
    }
}