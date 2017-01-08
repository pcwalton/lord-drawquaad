/* Any copyright is dedicated to the Public Domain.
 * http://creativecommons.org/publicdomain/zero/1.0/ */

extern crate gl;
extern crate glfw;
extern crate image;
extern crate lord_drawquaad;

use gl::types::GLint;
use glfw::{Action, Context, Key, OpenGlProfileHint, WindowEvent, WindowHint, WindowMode};
use std::env;
use std::os::raw::c_void;
use std::process;

pub fn main() {
    let path = env::args().nth(1).unwrap_or_else(|| usage());
    let image = image::open(&path).unwrap().to_rgba();

    let mut glfw = glfw::init(glfw::LOG_ERRORS).unwrap();
    glfw.window_hint(WindowHint::ContextVersion(3, 3));
    glfw.window_hint(WindowHint::OpenGlForwardCompat(true));
    glfw.window_hint(WindowHint::OpenGlProfile(OpenGlProfileHint::Core));
    let (mut window, events) = glfw.create_window(image.width(),
                                                  image.height(),
                                                  &path,
                                                  WindowMode::Windowed)
                                   .expect("Couldn't create a window!");

    window.make_current();
    gl::load_with(|symbol| window.get_proc_address(symbol) as *const c_void);

    let context = lord_drawquaad::Context::new();

    let mut texture = 0;
    unsafe {
        gl::GenTextures(1, &mut texture);
        gl::BindTexture(gl::TEXTURE_RECTANGLE, texture);

        gl::TexImage2D(gl::TEXTURE_RECTANGLE,
                       0,
                       gl::RGBA8 as GLint,
                       image.width() as GLint,
                       image.height() as GLint,
                       0,
                       gl::RGBA,
                       gl::UNSIGNED_BYTE,
                       (&*image).as_ptr() as *const c_void);

        gl::TexParameteri(gl::TEXTURE_RECTANGLE, gl::TEXTURE_MIN_FILTER, gl::LINEAR as GLint);
        gl::TexParameteri(gl::TEXTURE_RECTANGLE, gl::TEXTURE_MAG_FILTER, gl::LINEAR as GLint);
        gl::TexParameteri(gl::TEXTURE_RECTANGLE, gl::TEXTURE_WRAP_S, gl::CLAMP_TO_EDGE as GLint);
        gl::TexParameteri(gl::TEXTURE_RECTANGLE, gl::TEXTURE_WRAP_T, gl::CLAMP_TO_EDGE as GLint);
    }

    while !window.should_close() {
        context.draw(texture);
        window.swap_buffers();

        glfw.poll_events();
        for (_, event) in glfw::flush_messages(&events) {
            match event {
                WindowEvent::Key(Key::Escape, _, Action::Press, _) => {
                    window.set_should_close(true)
                }
                _ => {}
            }
        }
    }
}

fn usage() -> ! {
    println!("usage: example shrek.jpg");
    process::exit(0)
}

