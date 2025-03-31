#![allow(dead_code)]
#![allow(unused_variables)]

mod bounds;
mod collider2d;
mod draw_instance;
mod game;
mod gl_wrappers;
mod map;
mod object_components;
mod quad_tree;
mod render;
mod sprite;
mod transform;
mod player_config;
extern crate core;
extern crate nalgebra_glm as glm;

use glfw::{Action, Context, Key, OpenGlProfileHint, WindowHint};
use std::time::{SystemTime, UNIX_EPOCH};

use crate::game::Game;

fn main() {
    let mut glfw = glfw::init(glfw::fail_on_errors).unwrap();

    glfw.window_hint(WindowHint::ContextVersion(3, 3));
    glfw.window_hint(WindowHint::OpenGlProfile(OpenGlProfileHint::Core));
    #[cfg(target_os = "macos")]
    glfw.window_hint(WindowHint::OpenGlForwardCompat(true));
    
    glfw.window_hint(WindowHint::Resizable(false));

    // Create a windowed mode window and its OpenGL context
    let (mut window, events) = glfw
        .create_window(1024, 768, "Battle tanks", glfw::WindowMode::Windowed)
        .expect("Failed to create GLFW window.");

    gl::load_with(|s| window.get_proc_address(s) as *const _);
    gl::Viewport::load_with(|s| window.get_proc_address(s) as *const _);

    let (w, h) = window.get_size();
    unsafe {
        gl::Viewport(0, 0, w, h);
        gl::ClearColor(0.3, 0.3, 0.3, 1.);
    }

    let mut render = render::Render::new();

    render.init();

    
    // Make the window's context current
    window.make_current();
    glfw.set_swap_interval(glfw::SwapInterval::None);

    let mut game = Game::new(w as u32, h as u32);

    game.init(&mut render);

    window.set_key_polling(true);
    render::load_projection_matrix(&render.get_shader("default").unwrap(), w as u32, h as u32);
    let mut fps: u32 = 0;
    let mut timer: f64 = 0.;
    let mut delta: f64 = 0.;

    // Loop until the user closes the window
    while !window.should_close() {
        let tp = SystemTime::now();

        glfw.poll_events();
        for (_, event) in glfw::flush_messages(&events) {
            game.do_input(&event);
            if let glfw::WindowEvent::Key(Key::Escape, _, Action::Press, _) = event {
                window.set_should_close(true)
            }
        }

        // Swap front and back buffers
        unsafe {
            gl::Clear(gl::COLOR_BUFFER_BIT);
        }

        game.update(delta as f32);
        game.do_draw(&mut render);

        window.swap_buffers();
        //unsafe { gl::Flush(); }

        delta = if let Ok(frame_time) = SystemTime::now().duration_since(tp) {
            frame_time.as_secs_f64()
        } else {
            0.0
        };

        fps += 1;
        timer += delta;
        if timer > 1. {
            println!("FPS: {}", fps);
            fps = 0;
            timer = 0.;
        }
    }
}
