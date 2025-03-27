#![allow(dead_code)]
#![allow(unused_variables)]

use crate::bounds::Bounds;
use crate::collider2d::Collider2d;
use crate::quad_tree::QuadTree;
use glfw::{Action, Key};
use glm::make_vec2;
use rand::rngs::ThreadRng;
use std::cell::RefCell;
use std::ops::Deref;
use std::rc::{Rc, Weak};
use std::vec::Vec;

use crate::map::Map;
use crate::object_components::{Bullet, Damagable, Lifetime, Movable};
use crate::render::Drawable;
use crate::render::Render;
use crate::sprite::Sprite;
use crate::transform::Transform;
use ::ecs::*;
//use ::ecs::
use ecs_derive::component_impl;

#[derive(Debug, Clone, Copy, PartialEq)]
enum PlayerAction {
    MoveLeft,
    MoveRight,
    MoveTop,
    MoveDown,
    Shoot,
    None,
}

#[component_impl]
#[derive(Debug, Clone)]
pub struct PlayerController {
    direction: glm::Vec2,
    player_index: usize,
    last_action: PlayerAction,
    timer: f32,
    shoot_delay: f32,
}

impl PlayerController {
    fn new(entity: &EntityWeak, player_index: usize, dir: glm::Vec2) -> Self {
        Self {
            entity: entity.clone(),
            direction: glm::vec2(dir.x, dir.y),
            player_index,
            last_action: PlayerAction::None,
            timer: 0.,
            shoot_delay: 1.5,
        }
    }

    fn update(&mut self, dt: f32, player_input: PlayerAction) {
        /*if *player_input != PlayerAction::None {
            println!("{:?}", player_input);
        }*/
        self.timer += dt;

        self.direction = match player_input {
            PlayerAction::MoveLeft => make_vec2(&[-1., 0.]),
            PlayerAction::MoveRight => make_vec2(&[1., 0.]),
            PlayerAction::MoveTop => make_vec2(&[0., 1.]),
            PlayerAction::MoveDown => make_vec2(&[0., -1.]),
            PlayerAction::None | PlayerAction::Shoot => {
                glm::vec2(self.direction.x, self.direction.y)
            }
        };

        self.last_action = player_input;
    }

    fn can_spawn_bullet(&self) -> bool {
        self.timer > self.shoot_delay && self.last_action == PlayerAction::Shoot
    }

    fn spawn_bullet(&mut self, ent: &Weak<Entity>) -> Option<Bullet> {
        self.timer = 0_f32;

        let owner = self.entity.upgrade()?;

        Some(Bullet::new(ent, owner.get_id(), 2))
    }
}

struct Player {
    entity: EntityWeak,
    action: PlayerAction,
    input_layer: [(Key, PlayerAction); 5],
}

impl Player {
    fn new(entity: &EntityWeak, keys: &[(Key, PlayerAction); 5]) -> Self {
        Self {
            entity: entity.clone(),
            action: PlayerAction::None,
            input_layer: *keys,
        }
    }

    fn get_player_entity(&self) -> EntityWeak {
        self.entity.clone()
    }

    fn do_input(&mut self, event: &glfw::WindowEvent) {
        if let glfw::WindowEvent::Key(in_key, _, Action::Press, _) = event {
            if let Some(item) = self.input_layer.iter().find(|x| x.0 == *in_key) {
                self.action = item.1;
            }
        } else if let glfw::WindowEvent::Key(in_key, _, Action::Release, _) = event {
            let res = self.input_layer.iter().find(|x| x.0 == *in_key);

            if res.is_some() {
                self.action = PlayerAction::None;
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
}

fn test(x: EcsEvent) {
    println!("Test {:?}", x);
}

impl Game {
    pub fn new(width: u32, height: u32) -> Self {
        Self {
            rnd: rand::thread_rng(),
            world: Rc::new(RefCell::new(Ecs::new())),
            players: [None, None],
            map: Map::new(width, height),
            bullets: RefCell::new(Vec::new()),
            frame_counter: 0,
            quad_tree: QuadTree::new(Bounds::new(0_f32, 0_f32, width as f32, height as f32)),
        }
    }

    fn create_player(
        &mut self,
        render: &mut Render,
        layer: &[(Key, PlayerAction); 5],
        index: usize,
    ) -> Option<Player> {
        let entity_weak = Entity::new(&self.world);
        let entity = entity_weak.upgrade()?;
        let dir = make_vec2(&[0_f32, 1_f32]);
        let pos = make_vec2(&[50. + (100 * index) as f32, 100.]);
        let size = 50_f32;
        let bounds = Bounds::with_center_position(pos.x, pos.y, size, size);

        entity.add_component(|| Sprite::new(&entity_weak, size, size, "tank.png"));
        entity.add_component(|| Transform::with_direction(&entity_weak, pos, dir));
        entity.add_component(|| Collider2d::new(&entity_weak, bounds));
        entity.add_component(|| Movable::new(&entity_weak, 100.));
        entity.add_component(|| Damagable::new(&entity_weak, 10));
        entity.add_component(|| PlayerController::new(&entity_weak, index, dir));

        entity.visit(|sprite: &mut Option<Sprite>| sprite.as_mut().unwrap().init(render));

        self.quad_tree.place(&entity);

        Some(Player::new(&entity_weak, layer))
    }

    pub fn init(&mut self, render: &mut Render) {
        self.players[0] = self.create_player(
            render,
            &[
                (Key::A, PlayerAction::MoveLeft),
                (Key::D, PlayerAction::MoveRight),
                (Key::W, PlayerAction::MoveTop),
                (Key::S, PlayerAction::MoveDown),
                (Key::LeftControl, PlayerAction::Shoot),
            ],
            0,
        );

        self.players[1] = self.create_player(
            render,
            &[
                (Key::Left, PlayerAction::MoveLeft),
                (Key::Right, PlayerAction::MoveRight),
                (Key::Up, PlayerAction::MoveTop),
                (Key::Down, PlayerAction::MoveDown),
                (Key::RightControl, PlayerAction::Shoot),
            ],
            1,
        );
    }

    fn update_input(&self, dt: f32) {
        let ecs = self.world.deref().borrow();

        ecs.visit_all3::<Transform, PlayerController, Movable>(|transform, controller, movable| {
            if let Some(player) = &self.players[controller.player_index] {
                controller.update(dt, player.action);

                movable.set_dirty(
                    controller.last_action != PlayerAction::None
                        && controller.last_action != PlayerAction::Shoot,
                );

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
                            println!("bulllet was spawned {:?}", id);
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
                        println!("Cant move help!!!");
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
        self.spawn_bullets();
        self.bullet_system_update(dt);
        self.move_system_update(dt);

        self.frame_counter += 1;
    }

    pub fn do_input(&mut self, event: &glfw::WindowEvent) {
        for opt in &mut self.players {
            if let Some(player) = opt.as_mut() {
                player.do_input(event);
            }
        }
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
