#![allow(dead_code)]
#![allow(unused_variables)]

use crate::gl_wrappers;
use crate::gl_wrappers::Bindable;
use crate::gl_wrappers::ShaderProgram;
use crate::gl_wrappers::VertexArrayObject;
use crate::render::Drawable;
use crate::render::Render;

use std::rc::Rc;

#[derive(Debug)]
pub struct DrawInstance {
    program: Rc<ShaderProgram>,
    array_object: VertexArrayObject,
    texture: u32,
}

impl DrawInstance {
    pub fn new(program: Rc<ShaderProgram>, vao: VertexArrayObject, texture: u32) -> Self {
        DrawInstance {
            program,
            array_object: vao,
            texture,
        }
    }
}

impl Drawable for DrawInstance {
    fn draw(&self, render: &Render, matrix: &glm::Mat4) {
        self.program.bind();
        gl_wrappers::set_texture0_uniform(&self.program);
        render.load_transform_matrix(&self.program, matrix);

        unsafe {
            gl::ActiveTexture(gl::TEXTURE0);
            gl::BindTexture(gl::TEXTURE_2D, self.texture);

            //gl::BindVertexArray(self.vao);
            self.array_object.bind();

            gl::DrawElements(gl::TRIANGLES, 6, gl::UNSIGNED_SHORT, std::ptr::null());

            //gl::BindVertexArray(0);
            self.array_object.unbind();
        }

        self.program.unbind();
    }
}
