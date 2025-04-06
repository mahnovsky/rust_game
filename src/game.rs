#![allow(dead_code)]
#![allow(unused_variables)]

use crate::bounds::Bounds;
use crate::collider2d::{Collider2d, CollisionEvent};
use crate::system::ai_system::AiSystem;
use crate::system::fire_system::FireSystem;
use crate::quad_tree::QuadTree;
use crate::system::move_system::MoveSystem;
use crate::system::system_trait::System;
use glfw::{Action, Key};
use rand::rngs::ThreadRng;
use std::cell::RefCell;
use std::ops::Deref;
use std::rc::{Rc, Weak};
use std::vec::Vec;
use fxhash::FxHashMap;
use crate::map::Map;
use crate::object_components::{Bullet, Damagable, Gun, Lifetime, Movable};
use crate::render::Drawable;
use crate::render::Render;
use crate::sprite::Sprite;
use crate::transform::Transform;
use crate::player_config::{Player, PlayerAction, PlayerController, PlayerState};
use ::ecs::*;
use ecs_derive::component_impl;

#[component_impl]
#[derive(Debug, Clone)]
pub struct InputLayoutComponent {
    key_actions: FxHashMap<Key, PlayerAction>,
}

impl InputLayoutComponent {
    pub fn new(entity: EntityWeak, actions: FxHashMap<Key, PlayerAction>) -> Self {
        Self{ key_actions: actions, entity: entity }
    }

    fn do_input(&mut self, event: &glfw::WindowEvent) {
        if let glfw::WindowEvent::Key(in_key, _, action, _) = event {
            if let Some(item) = self.key_actions.get(in_key) {
                if let Some(entity) = self.entity.upgrade() {
                    if *action == Action::Press || *action == Action::Repeat {
                        entity.push_event(*item);
                    }
                    else {
                        entity.push_event(PlayerAction::None);
                    }
                }
            }
        }
    }
}

pub struct Game {
    rnd: ThreadRng,
    world: EcsRc,
    players: [Option<Player>; 2],
    map: Map,
    bullets: RefCell<Vec<Box<dyn FnOnce()>>>,
    frame_counter: u32,
    quad_tree: Rc<RefCell<QuadTree>>,
    // fire_system: FireSystem,
    systems: Vec<Box<dyn System>>,
}

impl Game {
    pub fn new(width: u32, height: u32) -> Self {
        Self {
            rnd: rand::rng(),
            world: Rc::new(RefCell::new(Ecs::new())),
            players: [None, None],
            map: Map::new(width, height),
            bullets: RefCell::new(Vec::new()),
            frame_counter: 0,
            quad_tree: Rc::new(RefCell::new(QuadTree::new(Bounds::new(0_f32, 0_f32, width as f32, height as f32)))),
            systems: Vec::new(),
        }
    }

    fn create_player(
        &mut self,
        render: &mut Render,
        config: &str,
        index: u32,
    ) -> Option<Player> {
        
        Player::new(&self.world, index, config, self.quad_tree.borrow_mut(), &render)
    }

    pub fn init(&mut self, render: &mut Render) {

        self.systems.push(Box::new(MoveSystem::new(self.quad_tree.clone())));
        self.systems.push(Box::new(FireSystem::new()));
        self.systems.push(Box::new(AiSystem::new(self.quad_tree.clone())));
        self.players[0] = self.create_player(
            render,
            "player1.yaml",
            0,
        );

        self.players[1] = self.create_player(
            render,
            "player2.yaml",
            1,
        );
    }

    fn bullet_system_update(&self, dt: f32) {
        let ecs = self.world.deref().borrow();
        if ecs.is_componet_exist::<Bullet>() {
            ecs.visit_all3::<Lifetime, Movable, Collider2d>(|lifetime, movable, collider| {
                lifetime.update(dt);

                if collider.is_reached_border() || lifetime.is_time_out() {
                    if let Some(entity) = collider.entity.upgrade() {
                        ecs.remove_entity(entity.get_id());
                    }
                } else {
                    movable.set_dirty(true);
                }
            });
        }
    }

    pub fn update(&mut self, dt: f32) {
        //self.map.update(&self.world);
        for s in self.systems.iter_mut() {
            s.update(&self.world, dt);
        }
        self.process_events();
        self.bullet_system_update(dt);
        //self.fire_system.update(&self.world, dt); 
        //self.move_system_update(dt);
        let mut ecs = self.world.borrow_mut();
        ecs.process_self_events();
        self.frame_counter += 1;
    }

    pub fn process_events(&self) {
        let ecs = self.world.borrow();
         ecs.process_events::<PlayerAction, PlayerController>();
         ecs.process_events::<PlayerAction, Gun>();
         ecs.clean_events::<PlayerAction>();

         ecs.process_events::<CollisionEvent, Bullet>(); 
    }

    pub fn do_input(&mut self, event: &glfw::WindowEvent) {
        let ecs = self.world.borrow_mut();
        ecs.visit_all::<InputLayoutComponent>(|input_component| {
            input_component.do_input(event);
        });
    }

    pub fn do_draw(&mut self, render: &mut Render) {
        let ecs = self.world.borrow();

        ecs.visit_all2::<Transform, Sprite>(|transform, sprite| {
            if !sprite.is_initizlized() {
                sprite.init(render);
            }

            transform.apply_transform_changes();
            sprite.draw(render, &transform.transform);
        });
    }
}
