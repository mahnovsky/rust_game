use crate::draw_instance::DrawInstance;
use crate::render::{make_quad, Drawable, Render};
use ::ecs::*;
use ecs_derive::component_impl;
use std::convert::From;
use std::rc::Rc;
extern crate nalgebra_glm as glm;

#[component_impl]
#[derive(Debug, Clone)]
pub struct Sprite {
    instance: Option<Rc<DrawInstance>>,
    program_name: String,
    texture_name: String,
    width: f32,
    height: f32,
}

impl Sprite {
    pub fn new(entity: &EntityWeak, width: f32, height: f32, texture_name: &str) -> Self {
        Sprite {
            entity: entity.clone(),
            instance: None,
            program_name: String::from("default"),
            texture_name: String::from(texture_name),
            width,
            height,
        }
    }

    pub fn init(&mut self, render: &Render) {
        let program_name = self.program_name.as_str();
        let program = render.get_shader(program_name);

        let vao = make_quad(self.width, self.height);
        let texture = render.load_texture(self.texture_name.as_str());
        let instance = DrawInstance::new(program.unwrap(), vao, texture);

        self.instance = Some(Rc::new(instance));
    }

    pub fn get_width(&self) -> f32 {
        self.width
    }

    pub fn get_height(&self) -> f32 {
        self.height
    }

    pub fn is_initizlized(&self) -> bool {
        self.instance.is_some()
    }
}

impl Drawable for Sprite {
    fn draw(&self, render: &Render, matrix: &glm::Mat4) {
        if let Some(instance) = self.instance.clone() {
            instance.draw(render, matrix);
        }
    }
}
