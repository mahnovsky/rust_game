#![allow(dead_code)]
#![allow(unused_variables)]

use crate::{
    bounds::Bounds,
    collider2d::CollisionEvent,
    map::Map,
    object_components::{Damagable, Lifetime, RemovedMarkerComponent},
    player_config::{self, spawn_player, PlayerAction, PlayerController},
    quad_tree::QuadTree,
    render::{Drawable, Render},
    sprite::Sprite,
    system::{
        ai_system::{self, AiSystem},
        fire_system, move_system,
    },
    transform::Transform,
};
use fxhash::FxHashMap;
use glfw::{
    Action::{self},
    Key::{self},
    Window,
};
use legion::{systems::CommandBuffer, *};
use rand::rngs::ThreadRng;

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

    pub fn scan_input(&self, window: &Window) -> Option<PlayerAction> {
        for (k, v) in self.key_actions.iter() {
            match window.get_key(*k) {
                Action::Press => return Some(*v),
                Action::Release | Action::Repeat => {}
            }
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
    res: Resources,
    ai_controller: ai_system::AiSystem,
    collisions: Vec<CollisionEvent>,
}

impl Game {
    pub fn new(width: u32, height: u32) -> Self {
        Self {
            rnd: rand::rng(),
            world: World::default(),
            map: Map::new(width, height),
            frame_counter: 0,
            quad_tree: QuadTree::new(Bounds::new(0_f32, 0_f32, width as f32, height as f32)),
            res: Resources::default(),
            ai_controller: AiSystem::new(),
            collisions: Vec::new(),
        }
    }

    pub fn init(&mut self, render: &mut Render) {
        let p1 = spawn_player(&mut self.world, &mut self.quad_tree, 0, "player1.yaml");
        let p2 = spawn_player(&mut self.world, &mut self.quad_tree, 1, "player2.yaml");

        println!("fn init {:?}", p1);
    }

    fn update_lifetimes(&mut self, cb: &mut CommandBuffer, dt: f32) {
        let mut query = <(Entity, &mut Lifetime)>::query();
        for (ent, lt) in query.iter_mut(&mut self.world) {
            lt.update(dt);

            if lt.is_time_out() {
                cb.add_component(*ent, RemovedMarkerComponent);
            }
        }

        let mut query =
            <(Entity, &Damagable)>::query().filter(!component::<RemovedMarkerComponent>());
        for (ent, d) in query.iter(&self.world) {
            if d.is_dead() {
                cb.add_component(*ent, RemovedMarkerComponent);
            }
        }
    }

    pub fn update(&mut self, window: &glfw::Window, dt: f32) {
        let mut cb = CommandBuffer::new(&self.world);
        self.collisions.clear();

        self.update_lifetimes(&mut cb, dt);
        ai_system::update(
            &mut self.ai_controller,
            &mut self.world,
            &mut self.quad_tree,
            &mut self.rnd,
            dt,
        );
        player_config::update_players(&mut self.world, window);
        move_system::move_update(
            &mut self.world,
            &mut self.quad_tree,
            dt,
            &mut self.collisions,
        );
        fire_system::update(
            &mut self.world,
            &mut self.quad_tree,
            &mut cb,
            dt,
            &self.collisions,
        );

        self.quad_tree
            .update(&mut self.world, &mut cb, &mut self.res);

        self.remove_all_marked_entities(&mut cb);

        cb.flush(&mut self.world, &mut self.res);

        self.frame_counter += 1;
    }

    fn remove_all_marked_entities(&mut self, cb: &mut CommandBuffer) {
        let mut q = <(Entity, &RemovedMarkerComponent)>::query();
        for (ent, _) in q.iter(&self.world) {
            cb.remove(*ent);
        }
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
