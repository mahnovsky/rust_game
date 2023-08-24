use std::ffi::{c_void, CStr, CString};
use std::{mem, ptr, vec};

pub trait Bindable {
    fn bind(&self);
    fn unbind(&self);
}

#[derive(Debug, Copy, Clone)]
pub enum ShaderType {
    Vertex,
    Fragment,
}

impl ShaderType {
    fn value(&self) -> u32 {
        match *self {
            ShaderType::Vertex => gl::VERTEX_SHADER,
            ShaderType::Fragment => gl::FRAGMENT_SHADER,
        }
    }
}

pub struct Shader(u32, ShaderType);
#[derive(Debug, Default)]
pub struct ShaderProgram(u32);

impl ShaderProgram {
    pub fn is_valid(&self) -> bool {
        self.0 > 0
    }
}

impl Bindable for ShaderProgram {
    fn bind(&self) {
        unsafe {
            gl::UseProgram(self.0);
        }
    }

    fn unbind(&self) {
        unsafe {
            gl::UseProgram(0);
        }
    }
}

impl Drop for ShaderProgram {
    fn drop(&mut self) {
        unsafe {
            gl::DeleteProgram(self.0);
        }
    }
}

pub fn use_program(program: ShaderProgram) {
    unsafe {
        gl::UseProgram(program.0);
    }
}

pub fn make_shader(src: &CStr, shader_type: ShaderType) -> Shader {
    let shader: u32 = unsafe { gl::CreateShader(shader_type.value()) };
    unsafe {
        gl::ShaderSource(shader, 1, &src.as_ptr(), ptr::null());

        gl::CompileShader(shader);
        check_shader_compile_status(shader);
    }
    Shader(shader, shader_type)
}

pub fn make_program(vert: Shader, frag: Shader) -> Option<ShaderProgram> {
    let program = unsafe { gl::CreateProgram() };
    let mut success: i32 = 0;
    unsafe {
        gl::AttachShader(program, vert.0);
        gl::AttachShader(program, frag.0);
        gl::LinkProgram(program);

        gl::GetProgramiv(program, gl::LINK_STATUS, &mut success);

        gl::DeleteShader(vert.0);
        gl::DeleteShader(frag.0);
    }

    if success == 0 {
        let v = vec![0; 512];
        let s = unsafe { CString::from_vec_unchecked(v) };
        let ptr = s.into_raw();
        unsafe {
            gl::GetProgramInfoLog(program, 512, std::ptr::null_mut(), ptr);

            println!(
                "failed link program {}",
                String::from_raw_parts(ptr as *mut u8, 512, 512)
            );
        }

        return None;
    } else {
        println!("shader linked success");
    }

    Some(ShaderProgram(program))
}

pub fn create_shader_program(vertex_src: &CStr, fragment_src: &CStr) -> Option<ShaderProgram> {
    let vertex_shader = make_shader(vertex_src, ShaderType::Vertex);
    let fragment_shader = make_shader(fragment_src, ShaderType::Fragment);

    make_program(vertex_shader, fragment_shader)
}

pub fn get_uniform_location(program: &ShaderProgram, name: &CStr) -> Option<u32> {
    let location: i32 = unsafe { gl::GetUniformLocation(program.0, name.as_ptr()) };

    if location >= 0 {
        return Some(location as u32);
    }
    println!("failed get uniform location -1 {}", name.to_str().unwrap());
    None
}

pub fn set_uniform_m4x4(program: &ShaderProgram, name: &CStr, matrix: &glm::Mat4) {
    if let Some(location) = get_uniform_location(program, name) {
        let slice = matrix.as_slice();

        unsafe {
            gl::UniformMatrix4fv(location as i32, 1, gl::FALSE, slice.as_ptr());
        }
    }
}

fn check_shader_compile_status(shader: u32) {
    let mut success: i32 = 0;
    unsafe {
        gl::GetShaderiv(shader, gl::COMPILE_STATUS, &mut success);
    }
    if success == 0 {
        println!("failed shader compile");
    } else {
        println!("shader compiled success")
    }
}

pub fn set_texture0_uniform(program: &ShaderProgram) {
    let name = CStr::from_bytes_with_nul("texture_sampler\0".as_bytes());
    let loc = get_uniform_location(program, name.unwrap()).unwrap();
    unsafe {
        gl::Uniform1i(loc as i32, 0);
    }
}

#[derive(Debug, Clone, Copy)]
pub enum BufferType {
    Array,
    Element,
}

impl BufferType {
    fn value(&self) -> u32 {
        match *self {
            BufferType::Array => gl::ARRAY_BUFFER,
            BufferType::Element => gl::ELEMENT_ARRAY_BUFFER,
        }
    }
}

#[derive(Debug)]
pub struct BufferObject {
    handle: u32,
    buffer_type: BufferType,
}

impl BufferObject {
    pub fn new(buffer_type: BufferType) -> Self {
        let mut buffer: u32 = 0;
        unsafe {
            gl::GenBuffers(1, &mut buffer);
        }
        BufferObject {
            handle: buffer,
            buffer_type,
        }
    }

    pub fn load_data<T>(&self, data: Vec<T>) {
        unsafe {
            gl::BufferData(
                self.buffer_type.value(),
                (mem::size_of::<T>() * data.len()) as isize,
                mem::transmute::<*const T, *const c_void>(data.as_ptr()),
                gl::STATIC_DRAW,
            );
        }
    }
}

impl Bindable for BufferObject {
    fn bind(&self) {
        unsafe {
            gl::BindBuffer(self.buffer_type.value(), self.handle);
        }
    }

    fn unbind(&self) {
        unsafe {
            gl::BindBuffer(self.buffer_type.value(), 0);
        }
    }
}

impl Drop for BufferObject {
    fn drop(&mut self) {
        unsafe {
            gl::DeleteBuffers(1, &self.handle);
        }
    }
}

#[derive(Debug)]
pub struct VertexArrayObject {
    handle: u32,
    array_buffer: BufferObject,
    element_buffer: BufferObject,
}

impl VertexArrayObject {
    pub fn new() -> Self {
        let mut vao: u32 = 0;

        unsafe {
            gl::GenVertexArrays(1, &mut vao);
        }

        VertexArrayObject {
            handle: vao,
            array_buffer: BufferObject::new(BufferType::Array),
            element_buffer: BufferObject::new(BufferType::Element),
        }
    }

    pub fn init_array_buffer<T>(&self, data: Vec<T>) {
        self.bind();

        self.array_buffer.bind();

        self.array_buffer.load_data::<T>(data);

        //self.unbind();
    }

    pub fn init_element_buffer<T>(&self, data: Vec<T>) {
        self.bind();

        self.element_buffer.bind();

        self.element_buffer.load_data::<T>(data);

        //self.unbind();
    }
}

impl Bindable for VertexArrayObject {
    fn bind(&self) {
        unsafe {
            gl::BindVertexArray(self.handle);
        }
    }

    fn unbind(&self) {
        unsafe {
            gl::BindVertexArray(0);
        }
    }
}

impl Drop for VertexArrayObject {
    fn drop(&mut self) {
        unsafe {
            gl::DeleteVertexArrays(1, &self.handle);
        }
    }
}
