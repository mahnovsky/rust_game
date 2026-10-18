//use glm::Vec2;
use legion::*;
use nalgebra_glm::Vec2;
use rand::{rngs::ThreadRng, seq::IteratorRandom, RngExt};
use strum::IntoEnumIterator;
use strum_macros::EnumIter;

#[derive(Debug, Clone, Copy, PartialEq, EnumIter)]
pub enum Direction {
    Left,
    Right,
    Up,
    Down,
}

impl Direction {
    pub fn to_vec(self) -> Vec2 {
        match self {
            Self::Left => Vec2::new(-1_f32, 0_f32),
            Self::Right => Vec2::new(1_f32, 0_f32),
            Self::Up => Vec2::new(0_f32, 1_f32),
            Self::Down => Vec2::new(0_f32, -1_f32),
        }
    }

    pub fn velocity(self, speed: f32) -> Vec2 {
        self.to_vec() * speed
    }

    pub fn get_rng(self, rng: &mut ThreadRng) -> Self {
        let x = Direction::iter().filter(|x| *x != self);

        x.choose(rng).unwrap_or(self)
    }
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum MoveSwitch {
    Stay,
    Forward,
}

impl MoveSwitch {
    fn to_scalar(self) -> f32 {
        match self {
            Self::Stay => 0_f32,
            Self::Forward => 1_f32,
        }
    }
}

#[derive(Debug, Clone)]
pub struct Movable {
    max_speed: u16,
    move_switch: MoveSwitch,
    direction: Direction,
}

impl Movable {
    pub fn new(max_speed: u16, direction: Direction, move_switch: MoveSwitch) -> Self {
        Self {
            max_speed,
            move_switch,
            direction,
        }
    }

    pub fn get_move_fragment(&self, scale: f32) -> Vec2 {
        self.direction.velocity(self.max_speed as f32) * scale * self.move_switch.to_scalar()
    }

    pub fn get_direction_vec(&self) -> Vec2 {
        self.direction.to_vec()
    }

    pub fn get_direction(&self) -> Direction {
        self.direction
    }

    pub fn set_direction(&mut self, dir: Direction) {
        self.direction = dir;
    }

    pub fn set_move_switch(&mut self, ms: MoveSwitch) {
        self.move_switch = ms;
    }
}

#[derive(Debug, Clone)]
pub struct BulletSpawner {
    pub owner_id: Entity,
    pub damage: u32,
    pub pos: glm::Vec2,
    pub dir: Direction,
}

impl BulletSpawner {
    fn new(owner_id: Entity, damage: u32, pos: glm::Vec2, dir: Direction) -> Self {
        Self {
            owner_id,
            damage,
            pos,
            dir,
        }
    }

    pub fn spawn_bullet(&self, ent: Entity) -> Bullet {
        Bullet::new(self.owner_id, self.damage)
    }
}

#[derive(Debug, Clone)]
pub struct Gun {
    damage: u32,
    timer: f32,
    shoot_delay: f32,
    spawner: Option<BulletSpawner>,
}

impl Gun {
    pub fn new(damage: u32) -> Self {
        Self {
            damage,
            timer: 0_f32,
            shoot_delay: 0.3_f32,
            spawner: None,
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

    pub fn setup_shoot(&mut self, owner_id: Entity, pos: Vec2, dir: Direction) {
        self.spawner = Some(BulletSpawner {
            owner_id,
            damage: self.damage,
            pos,
            dir,
        });
    }
}

pub enum BulletEvent {
    OnHit(u32),
}

#[derive(Debug, Clone)]
pub struct Bullet {
    damage: u32,
    owner: Entity,
    target: Option<Entity>,
}

impl Bullet {
    pub fn new(owner: Entity, damage: u32) -> Self {
        Self {
            damage,
            owner,
            target: None,
        }
    }

    pub fn get_damage(&self) -> u32 {
        self.damage
    }

    pub fn get_owner(&self) -> Entity {
        self.owner
    }

    pub fn consume_target(&mut self) -> Option<Entity> {
        self.target.take()
    }
}

#[derive(Debug, Clone)]
pub struct Damagable {
    health: u32,
}

impl Damagable {
    pub fn new(health: u32) -> Self {
        Self { health }
    }

    pub fn do_damage(&mut self, damage: u32) {
        if self.health > damage {
            self.health -= damage;
        } else {
            self.health = 0;
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

#[derive(Debug, Clone)]
pub struct Lifetime {
    timer: f32,
    life_time: f32,
    time_out: bool,
}

impl Lifetime {
    pub fn new(life_time: f32) -> Self {
        Self {
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
        self.time_out
    }
}

pub struct RemovedMarkerComponent;
