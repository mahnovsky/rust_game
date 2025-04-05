#![allow(dead_code)]
#![allow(unused_variables)]

use crate::bounds::Bounds;
use crate::collider2d::Collider2d;
use crate::fire_system::{self, FireSystem};
use crate::quad_tree::QuadTree;
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
    quad_tree: QuadTree,
    fire_system: FireSystem,
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
            quad_tree: QuadTree::new(Bounds::new(0_f32, 0_f32, width as f32, height as f32)),
            fire_system: FireSystem::new(),
        }
    }

    fn create_player(
        &mut self,
        render: &mut Render,
        config: &str,
        index: u32,
    ) -> Option<Player> {
        
        Player::new(&self.world, index, config, &self.quad_tree, &render)
    }

    pub fn init(&mut self, render: &mut Render) {
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

    fn update_input(&self, dt: f32) {
        let ecs = self.world.deref().borrow();

        ecs.visit_all3::<Transform, PlayerController, Movable>(|transform, controller, movable| {
            if let Some(player) = &self.players[controller.player_index as usize] {
                
                movable.set_dirty(controller.state == PlayerState::Move);

                let direction = controller.direction;

                if !transform.get_direction().eq(&direction) {
                    transform.set_direction(&direction);
                }

                if controller.can_spawn_bullet() {
                    let tr_owner = transform.clone();
                    let mut bullets = self.bullets.borrow_mut();
                    let weak_ent = Entity::new(&self.world);
                    let bullet = controller.spawn_bullet(&weak_ent).unwrap();
                    let owner_id = movable.get_entity_id().unwrap();
                    bullets.push(Box::new(move || {
                        if let Some(entity) = weak_ent.upgrade() {
                            let pos = tr_owner.get_position() + direction * 30.;
                            entity.add_component(|| bullet);
                            entity.add_component(|| {
                                Transform::with_direction(&weak_ent, pos, direction)
                            });
                            entity.add_component(|| Movable::new(&weak_ent, 200.));
                            entity.add_component(|| {
                                Collider2d::with_ignore(
                                    &weak_ent,
                                    Bounds::with_center_position(pos.x, pos.y, 10., 10.),
                                    vec![owner_id],
                                )
                            });
                            entity.add_component(|| Sprite::new(&weak_ent, 10., 10., "tank1.png"));
                            entity.add_component(|| Damagable::new(&weak_ent, 1));
                            entity.add_component(|| Lifetime::new(&weak_ent, 2.));

                            let id = entity.get_id();
                            println!("bullet was spawned {:?}", id);
                        }
                    }));
                }
            }
        });
    }

    fn spawn_bullets(&self) {
        let mut v = self.bullets.borrow_mut();
        while let Some(e) = v.pop() {
            e();
        }
    }

    pub fn move_system_update(&mut self, dt: f32) {
        let ecs = self.world.deref().borrow();

        ecs.visit_all2::<Transform, Movable>(|transform, movable| {
            if movable.is_dirty() {
                let pos = transform.get_position();
                let speed = movable.get_speed() * dt;
                let dir = transform.get_direction();
                let new_pos = pos + dir * speed;
                if let Some(entity) = transform.entity.upgrade() {
                    if self.quad_tree.move_object(&ecs, &entity, new_pos) {
                        transform.set_position(&new_pos);
                    } else {
                        println!("Cant move help!!! {:?}", new_pos);
                    }
                }

                movable.set_dirty(false);
            }
        });
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
        self.update_input(dt);
        self.process_events();
        self.spawn_bullets();
        self.bullet_system_update(dt);
        self.fire_system.update(&self.world, dt); 
        self.move_system_update(dt);

        self.frame_counter += 1;
    }

    pub fn process_events(&self) {
        let ecs = self.world.borrow();
         ecs.process_events::<PlayerAction, PlayerController>();
         //ecs.process_events::<PlayerAction, Movable>();
         ecs.process_events::<PlayerAction, Gun>();
         ecs.clean_events::<PlayerAction>();
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
