use std::ffi::{c_void, CStr, CString};
//use glfw::{Action, Context, Key, OpenGlProfileHint, WindowHint};
//use gl::types::*;
use crate::gl_wrappers::{self, Bindable, VertexArrayObject};
use std::cell::RefCell;
use std::collections::HashMap;
use std::rc::Rc;
use std::{mem, ptr};

use image::io::Reader as ImageReader;

use crate::gl_wrappers::ShaderProgram;

extern crate nalgebra_glm as glm;

fn get_default_vertex_src() -> &'static CStr {
    CStr::from_bytes_with_nul(
        "
    #version 330 core
    layout (location = 0) in vec3 aPos;
    layout (location = 1) in vec2 texPos;

    out vec2 TexCoord;

    uniform mat4 Projection;
    uniform mat4 Transform;

    void main()
    {
        gl_Position = Projection * Transform * vec4(aPos.x, aPos.y, aPos.z, 1.0);
        TexCoord = texPos;
    }
\n\0"
            .as_bytes(),
    )
    .unwrap()
}

fn get_default_fragment_src() -> &'static CStr {
    CStr::from_bytes_with_nul(
        "
    #version 330 core

    in vec2 TexCoord;
    out vec4 FragColor;

    uniform sampler2D texture_sampler;

    void main()
    {
        FragColor = texture(texture_sampler, TexCoord);
    }
\n\0"
            .as_bytes(),
    )
    .unwrap()
}

pub fn load_projection_matrix(program: &ShaderProgram, screen_w: u32, screen_h: u32) {
    let proj = glm::ortho(0., screen_w as f32, 0., screen_h as f32, -10., 10.);
    let name = CStr::from_bytes_with_nul(b"Projection\0").unwrap();
    program.bind();
    gl_wrappers::set_uniform_m4x4(program, name, &proj);
}

pub fn make_default_shader_program() -> Option<ShaderProgram> {
    gl_wrappers::create_shader_program(get_default_vertex_src(), get_default_fragment_src())
}

pub fn make_and_bind_vao() -> u32 {
    let mut vao: u32 = 0;
    unsafe {
        gl::GenVertexArrays(1, &mut vao);
        gl::BindVertexArray(vao);
    }

    vao
}

pub fn make_index_buffer(indices: Vec<u16>) -> u32 {
    let mut ebo: u32 = 0;
    unsafe {
        gl::GenBuffers(1, &mut ebo);
        println!("ebo {}", ebo);
        gl::BindBuffer(gl::ELEMENT_ARRAY_BUFFER, ebo);

        let len = (mem::size_of::<u16>() * indices.len()) as isize;
        println!("len {}", len);
        gl::BufferData(
            gl::ELEMENT_ARRAY_BUFFER,
            len,
            mem::transmute::<*const u16, *const c_void>(indices.as_ptr()),
            gl::STATIC_DRAW,
        );
    }
    ebo
}

pub fn make_quad(width: f32, height: f32) -> VertexArrayObject {
    //let triangle = [-0.5, -0.5, 0., 0.5, -0.5, 0., 0.0, 0.5, 0.];
    let mut vertices = vec![
        // first triangle
        0.5, 0.5, 0.0, 1.0, 1.0, // top right
        0.5, -0.5, 0.0, 1.0, 0.0, // bottom right
        -0.5, -0.5, 0.0, 0.0, 0.0, // bottom left
        -0.5, 0.5, 0.0, 0.0, 1.0,
    ];

    for i in (0..vertices.len()).step_by(5) {
        vertices[i] *= width
    }
    for i in (1..vertices.len()).step_by(5) {
        vertices[i] *= height
    }

    let vao = VertexArrayObject::new();
    vao.init_array_buffer::<f32>(vertices);
    /*
    let mut vbo: u32 = 0;
    unsafe {
        gl::GenBuffers(1, &mut vbo);
        gl::BindBuffer(gl::ARRAY_BUFFER, vbo);
        gl::BufferData(
            gl::ARRAY_BUFFER,
            (mem::size_of::<f32>() * vertices.len()) as isize,
            mem::transmute::<*const f32, *const c_void>(vertices.as_ptr()),
            gl::STATIC_DRAW,
        );
    }

    make_index_buffer(vec![
        // note that we start from 0!
        0, 1, 3, // first triangle
        1, 2, 3, // second triangle
    ]);*/

    let indices = vec![
        // note that we start from 0!
        0u16, 1, 3, // first triangle
        1, 2, 3, // second triangle
    ];

    vao.init_element_buffer::<u16>(indices);

    unsafe {
        let stride = (5 * mem::size_of::<f32>()) as i32;
        gl::VertexAttribPointer(0, 3, gl::FLOAT, gl::FALSE, stride, ptr::null());
        gl::EnableVertexAttribArray(0);
        let pointer = 3 * std::mem::size_of::<f32>() as u32;
        gl::VertexAttribPointer(
            1,
            2,
            gl::FLOAT,
            gl::FALSE,
            stride,
            pointer as *const u32 as *const c_void,
        );
        gl::EnableVertexAttribArray(1);
    }

    vao
}

pub fn draw_instance(program: ShaderProgram, vao: u32, texture: u32) {
    unsafe {
        program.bind();

        gl_wrappers::set_texture0_uniform(&program);
        gl::ActiveTexture(gl::TEXTURE0);
        gl::BindTexture(gl::TEXTURE_2D, texture);

        gl::BindVertexArray(vao);

        gl::DrawElements(gl::TRIANGLES, 6, gl::UNSIGNED_SHORT, std::ptr::null());

        gl::BindVertexArray(0);

        program.unbind();
    }
}

pub struct ShaderCache {
    programs: HashMap<String, Rc<ShaderProgram>>,
}

impl ShaderCache {
    pub fn new() -> ShaderCache {
        ShaderCache {
            programs: HashMap::new(),
        }
    }

    fn is_program_exist(&self, name: &str) -> bool {
        self.programs.contains_key(name)
    }

    pub fn get_program(&mut self, name: &str) -> Option<Rc<ShaderProgram>> {
        match self.programs.get(name) {
            Some(value) => Some(value.clone()),
            None => self.try_load_program(name),
        }
    }

    fn try_load_program(&mut self, name: &str) -> Option<Rc<ShaderProgram>> {
        if name == "default" {
            if let Some(default_program) = make_default_shader_program() {
                let program = Rc::new(default_program);
                self.programs.insert(name.to_string(), Rc::clone(&program));

                return Some(program);
            }
        }

        None
    }
}

pub trait Drawable {
    fn draw(&self, render: &Render, matrix: &glm::Mat4);
}

pub struct Render {
    shader_cache: RefCell<ShaderCache>,
    textures: RefCell<HashMap<String, u32>>,
    transform_name: CString,
}

impl Render {
    pub fn new() -> Render {
        Render {
            shader_cache: RefCell::new(ShaderCache::new()),
            textures: RefCell::new(HashMap::new()),
            transform_name: CString::new("Transform").unwrap(),
        }
    }

    pub fn init(&self) {
        unsafe {
            gl::Enable(gl::BLEND);
            gl::BlendFunc(gl::SRC_ALPHA, gl::ONE_MINUS_SRC_ALPHA);
        }
    }

    pub fn get_shader(&self, name: &str) -> Option<Rc<ShaderProgram>> {
        let mut shader_cache = self.shader_cache.borrow_mut();
        if shader_cache.is_program_exist(name) {
            shader_cache.get_program(name)
        } else {
            shader_cache.try_load_program(name)
        }
    }

    pub fn load_transform_matrix(&self, program: &ShaderProgram, matrix: &glm::Mat4) {
        gl_wrappers::set_uniform_m4x4(program, self.transform_name.as_c_str(), matrix);
    }

    pub fn load_texture(&self, file_name: &str) -> u32 {
        {
            let textures = self.textures.borrow();
            if let Some(texture) = textures.get(file_name) {
                return *texture;
            }
        }

        if let Ok(img) = ImageReader::open(file_name) {
            let img = img.decode().unwrap().flipv();

            unsafe {
                let mut texture: u32 = 0;
                gl::GetError();
                gl::ActiveTexture(gl::TEXTURE0);
                gl::GenTextures(1, &mut texture);
                gl::BindTexture(gl::TEXTURE_2D, texture);

                // set the texture wrapping/filtering options (on the currently bound texture object)
                gl::TexParameteri(gl::TEXTURE_2D, gl::TEXTURE_WRAP_S, gl::REPEAT as i32);
                gl::TexParameteri(gl::TEXTURE_2D, gl::TEXTURE_WRAP_T, gl::REPEAT as i32);
                gl::TexParameteri(
                    gl::TEXTURE_2D,
                    gl::TEXTURE_MIN_FILTER,
                    gl::LINEAR_MIPMAP_LINEAR as i32,
                );
                gl::TexParameteri(gl::TEXTURE_2D, gl::TEXTURE_MAG_FILTER, gl::LINEAR as i32);

                // load and generate the texture
                let mut created = false;
                if let Some(rgb) = img.as_rgb8() {
                    gl::TexImage2D(
                        gl::TEXTURE_2D,
                        0,
                        gl::RGB as i32,
                        rgb.width() as i32,
                        rgb.height() as i32,
                        0,
                        gl::RGB,
                        gl::UNSIGNED_BYTE,
                        rgb.as_ptr() as *const c_void,
                    );

                    created = true;
                } else if let Some(rgba) = img.as_rgba8() {
                    gl::TexImage2D(
                        gl::TEXTURE_2D,
                        0,
                        gl::RGBA as i32,
                        rgba.width() as i32,
                        rgba.height() as i32,
                        0,
                        gl::RGBA,
                        gl::UNSIGNED_BYTE,
                        rgba.as_ptr() as *const c_void,
                    );

                    created = true;
                }

                if created {
                    gl::GenerateMipmap(gl::TEXTURE_2D);

                    let mut textures = self.textures.borrow_mut();
                    textures.insert(String::from(file_name), texture);

                    return texture;
                }
            }
        }
        0
    }
}
