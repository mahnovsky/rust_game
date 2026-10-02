#![allow(dead_code)]
#![allow(unused_variables)]

use crate::{
    bounds::Bounds,
    collider2d::{Collider2d, CollisionEvent},
    map::Map,
    object_components::{Bullet, Damagable, Gun, Lifetime, Movable},
    player_config::{spawn_player, PlayerAction, PlayerController, PlayerState},
    quad_tree::QuadTree,
    render::{Drawable, Render},
    sprite::Sprite,
    system::{
        ai_system::AiSystem,
        fire_system::FireSystem,
        move_system::{self, MoveSystem},
    },
    transform::Transform,
};
use fxhash::FxHashMap;
use glfw::{Action, Key};
use legion::*;
use rand::rngs::ThreadRng;
use std::ops::Deref;

#[derive(Debug, Clone)]
pub struct InputLayoutComponent {
    key_actions: FxHashMap<Key, PlayerAction>,
}

impl InputLayoutComponent {
    pub fn new(actions: FxHashMap<Key, PlayerAction>) -> Self {
        Self {
            key_actions: actions,
        }
    }

    pub fn do_input(&mut self, event: &glfw::WindowEvent) -> Option<PlayerAction> {
        if let glfw::WindowEvent::Key(in_key, _, action, _) = event {
            return self.key_actions.get(in_key).cloned();
        }
        None
    }
}

pub struct Game {
    rnd: ThreadRng,
    world: World,
    map: Map,
    frame_counter: u32,
    quad_tree: QuadTree,
}

impl Game {
    pub fn new(width: u32, height: u32) -> Self {
        Self {
            rnd: rand::rng(),
            world: World::default(),
            map: Map::new(width, height),
            frame_counter: 0,
            quad_tree: QuadTree::new(Bounds::new(0_f32, 0_f32, width as f32, height as f32)),
        }
    }

    pub fn init(&mut self, render: &mut Render) {
        let p1 = spawn_player(&mut self.world, &mut self.quad_tree, 0, "player1.yaml");
        let p2 = spawn_player(&mut self.world, &mut self.quad_tree, 1, "player2.yaml");

        println!("fn init {:?}", p1);
    }

    fn bullet_system_update(&self, dt: f32) {
        /*if ecs.is_componet_exist::<Bullet>() {
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
        }*/
    }

    fn update_players(&mut self) {
        let mut query = <(&PlayerController, &mut Movable)>::query();

        for (pc, movable) in query.iter_mut(&mut self.world) {
            pc.update_movable_state(movable);
        }
    }

    pub fn update(&mut self, dt: f32) {
        //self.map.update(&self.world);
        /*for s in self.systems.iter_mut() {
            s.update(&self.world, dt);
        }*/

        self.update_players();
        move_system::move_update(&mut self.world, &mut self.quad_tree, dt);

        self.bullet_system_update(dt);
        //self.fire_system.update(&self.world, dt);
        //self.move_system_update(dt);
        //ecs.process_self_events();
        self.frame_counter += 1;
    }

    pub fn do_input(&mut self, event: &glfw::WindowEvent) {
        let mut query = <&mut PlayerController>::query();

        for pc in query.iter_mut(&mut self.world) {
            pc.do_input(event);
        }
    }

    pub fn do_draw(&mut self, render: &mut Render) {
        let mut query = <(&mut Transform, &mut Sprite)>::query();

        for (transform, sprite) in query.iter_mut(&mut self.world) {
            if !sprite.is_initizlized() {
                sprite.init(render);
            }

            transform.apply_transform_changes();
            sprite.draw(render, &transform.transform);
        }
    }
}
