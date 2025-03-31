use ::ecs::*;
use ecs_derive::component_impl;

use crate::player_config::PlayerAction;

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

// impl Listener<PlayerAction> for Movable {
//     fn on_event(&mut self, event: PlayerAction) {
//         println!("Movable action event receive");
//         self.dirty = match event {
//             PlayerAction::MoveDown | 
//             PlayerAction::MoveTop |
//             PlayerAction::MoveLeft |
//             PlayerAction::MoveRight => true,
//             _ => false
//         }
//     }
// }

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
