use ::ecs::*;
use ecs_derive::component_impl;
use glfw::ffi::GLFWcharfun;

use crate::{player_config::PlayerAction, transform::Transform};

#[component_impl]
#[derive(Debug, Clone)]
pub struct Movable {
    speed: f32,
    dirty: bool,
}

impl Movable {
    pub fn new(entity: &EntityWeak, speed: f32) -> Self {
        Self {
            entity: entity.clone(),
            speed,
            dirty: true,
        }
    }

    pub fn get_speed(&self) -> f32 {
        self.speed
    }

    pub fn set_dirty(&mut self, dirty: bool) {
        self.dirty = dirty;
    }

    pub fn is_dirty(&self) -> bool {
        self.dirty
    }
}

#[derive(Debug, Clone)]
pub struct BulletSpawner {
    pub owner_id: EntityId,
    damage: u32,
    pub pos: glm::Vec2,
    pub dir: glm::Vec2,
}

impl BulletSpawner {
    fn new(owner_id: EntityId, damage: u32, pos: glm::Vec2, dir: glm::Vec2) -> Self {
        Self { owner_id, damage, pos, dir }
    }

    pub fn spawn_bullet(&self, ent: &EntityWeak) -> Bullet {
        Bullet::new(ent, self.owner_id, self.damage)
    }
}

#[component_impl]
#[derive(Debug, Clone)]
pub struct Gun {
    damage: u32,
    timer: f32,
    shoot_delay: f32,
    spawner: Option<BulletSpawner>
}

impl Gun {
    pub fn new(entity: &EntityWeak, damage: u32) -> Self {
        Self {
            entity: entity.clone(),
            damage,
            timer: 0_f32,
            shoot_delay: 2_f32,
            spawner: None
        }
    }

    pub fn get_damage(&self) -> u32 {
        self.damage
    }

    pub fn can_spawn_bullet(&self) -> bool {
        self.timer > self.shoot_delay
    }

    pub fn update_timer(&mut self, delta: f32) {
        if self.timer < self.shoot_delay {
            self.timer += delta;
        }
    }

    pub fn consume_spawner(&mut self) -> Option<BulletSpawner> {
        if self.spawner.is_some() {
            self.timer = 0.;
        }
        self.spawner.take()
    } 
}

impl Listener<PlayerAction> for Gun {
    fn on_event(&mut self, event: PlayerAction) {
        println!("Gun action event receive");
        if event == PlayerAction::Shoot && self.can_spawn_bullet() {
            let id = self.get_entity_id().unwrap();
            let ent = self.entity.upgrade().unwrap();
            let tr = ent.get_component_clone::<Transform>().unwrap();
            self.spawner = Some(BulletSpawner::new(id, self.damage, tr.get_position(), tr.get_direction()))
        }
    }
}

#[component_impl]
#[derive(Debug, Clone)]
pub struct Bullet {
    damage: u32,
    owner: EntityId,
}

impl Bullet {
    pub fn new(entity: &EntityWeak, owner: EntityId, damage: u32) -> Self {
        Self {
            entity: entity.clone(),
            damage,
            owner,
        }
    }

    pub fn get_damage(&self) -> u32 {
        self.damage
    }

    pub fn get_owner(&self) -> EntityId {
        self.owner
    }
}

#[component_impl]
#[derive(Debug, Clone)]
pub struct Damagable {
    health: u32,
}

impl Damagable {
    pub fn new(entity: &EntityWeak, health: u32) -> Self {
        Self {
            entity: entity.clone(),
            health,
        }
    }

    pub fn do_damage(&mut self, damage: u32) {
        if let Some(entity) = self.entity.upgrade() {
            if self.health > damage {
                self.health -= damage;
            } else {
                self.health = 0;

                entity.kill();
            }
        }
        println!("do damage for {}, dmg: {}", self.health, damage);
    }

    pub fn is_dead(&self) -> bool {
        self.health == 0
    }

    pub fn kill(&mut self) {
        self.health = 0;
    }
}

#[component_impl]
#[derive(Debug, Clone)]
pub struct Lifetime {
    timer: f32,
    life_time: f32,
    time_out: bool,
}

impl Lifetime {
    pub fn new(entity: &EntityWeak, life_time: f32) -> Self {
        Self {
            entity: entity.clone(),
            timer: 0.,
            life_time,
            time_out: false,
        }
    }

    pub fn update(&mut self, dt: f32) {
        self.timer += dt;
        if !self.time_out && self.timer > self.life_time {
            self.time_out = true;
        }
    }

    pub fn is_time_out(&self) -> bool {
        if self.time_out {
            if let Some(ent) = self.entity.upgrade() {
                println!("Lifetime timeout {}, {:?}", self.timer, ent.get_id());
            }
        }
        self.time_out
    }
}
