extern crate nalgebra_glm as glm;
use ::ecs::*;
use ecs_derive::component_impl;

use glm::vec2;

pub enum TransformProp {
    Position(glm::Vec2),
    Direction(glm::Vec2),
    Rotation(f32),
}

#[component_impl]
#[derive(Debug, Clone, Default)]
pub struct Transform {
    rotation: f32,
    position: glm::Vec2,
    direction: glm::Vec2,
    pub transform: glm::Mat4,
    pub transform_changed: bool,
}

impl Transform {
    pub fn new(entity: &EntityWeak, position: glm::Vec2) -> Self {
        let mut result = Self {
            entity: entity.clone(),
            rotation: 0.,
            position,
            direction: vec2(0., 1.),
            transform: glm::identity(),
            transform_changed: true,
        };

        result.apply_transform_changes();

        result
    }

    pub fn with_direction(entity: &EntityWeak, position: glm::Vec2, dir: glm::Vec2) -> Self {
        let mut result = Self {
            entity: entity.clone(),
            rotation: 0.,
            position,
            direction: dir,
            transform: glm::identity(),
            transform_changed: true,
        };

        result.apply_transform_changes();

        result
    }

    pub fn from_props(props: &[TransformProp]) -> Self {
        let mut tr = Transform::default();

        props.iter().for_each(|p| tr.apply(p));

        tr
    }

    pub fn apply(&mut self, prop: &TransformProp) {
        match prop {
            TransformProp::Position(p) => self.set_position(p),
            TransformProp::Direction(d) => self.set_direction(d),
            TransformProp::Rotation(r) => self.set_rotation(*r),
        }
    }

    pub fn get_position(&self) -> glm::Vec2 {
        glm::make_vec2(&[self.position.x, self.position.y])
    }

    pub fn set_position(&mut self, pos: &glm::Vec2) {
        self.position = *pos;
        self.transform_changed = true;
    }

    pub fn get_rotation(&self) -> f32 {
        self.rotation
    }

    pub fn set_rotation(&mut self, rotation: f32) {
        self.rotation = rotation;
        self.transform_changed = true;
    }

    pub fn set_direction(&mut self, dir: &glm::Vec2) {
        self.direction = vec2(dir.x, dir.y);
        self.transform_changed = true;
    }

    pub fn get_direction(&self) -> glm::Vec2 {
        self.direction
    }

    pub fn apply_transform_changes(&mut self) {
        if self.transform_changed {
            let def_up = glm::vec3::<f32>(0., 1., 0.);
            let def_left = glm::vec3::<f32>(1., 0., 0.);
            let dir = glm::vec3(self.direction.x, self.direction.y, 0.);
            let x_ang = glm::angle::<f32, 3>(&def_left, &dir);
            self.rotation =
                glm::angle::<f32, 3>(&def_up, &dir) * (if x_ang <= 0. { -1. } else { 1. });

            //println!("rotation {}, x_ang {}", self.rotation, x_ang);
            let pos = glm::vec3(self.position.x, self.position.y, 0.);

            self.transform =
                glm::translation(&pos) * glm::rotation(self.rotation, &glm::vec3(0., 0., 1.));

            self.transform_changed = false;
        }
    }
}
