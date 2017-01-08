// Copyright 2017 Mozilla Foundation. See the COPYRIGHT file
// at the top-level directory of this distribution and at
// http://rust-lang.org/COPYRIGHT.
//
// Licensed under the Apache License, Version 2.0 <LICENSE-APACHE or
// http://www.apache.org/licenses/LICENSE-2.0> or the MIT license
// <LICENSE-MIT or http://opensource.org/licenses/MIT>, at your
// option. This file may not be copied, modified, or distributed
// except according to those terms.

//! A minimalist, dead-simple library to draw full-screen textured quads in OpenGL 3.3+.
//!
//! The goal is to factor out the annoying shader boilerplate.
//!
//! You should not use this library if you're particularly concerned about performance. It doesn't
//! do any batching.

extern crate gl;

use gl::types::{GLchar, GLint, GLsizei, GLsizeiptr, GLuint, GLvoid};
use std::mem;
use std::os::raw::c_void;

pub struct Context {
    vertex_shader: GLuint,
    fragment_shader: GLuint,
    program: GLuint,
    texture_uniform: GLint,
    vertex_array: GLuint,
    vertex_buffer: GLuint,
}

impl Context {
    /// Creates a context, encapsulating the state necessary to draw textured quads.
    ///
    /// You must have a current valid GL context before calling this.
    pub fn new() -> Context {
        unsafe {
            let vertex_shader = gl::CreateShader(gl::VERTEX_SHADER);
            let fragment_shader = gl::CreateShader(gl::FRAGMENT_SHADER);
            gl::ShaderSource(vertex_shader,
                             1,
                             &(VERTEX_SHADER.as_ptr() as *const GLchar),
                             &(VERTEX_SHADER.len() as GLint));
            gl::ShaderSource(fragment_shader,
                             1,
                             &(FRAGMENT_SHADER.as_ptr() as *const GLchar),
                             &(FRAGMENT_SHADER.len() as GLint));
            gl::CompileShader(vertex_shader);
            gl::CompileShader(fragment_shader);

            let program = gl::CreateProgram();
            gl::AttachShader(program, vertex_shader);
            gl::AttachShader(program, fragment_shader);
            gl::LinkProgram(program);
            gl::UseProgram(program);

            let position_attribute =
                gl::GetAttribLocation(program, "aPosition\0".as_ptr() as *const GLchar);
            let tex_coord_attribute =
                gl::GetAttribLocation(program, "aTexCoord\0".as_ptr() as *const GLchar);
            let texture_uniform =
                gl::GetUniformLocation(program, "uTexture\0".as_ptr() as *const GLchar);

            let mut vertex_array = 0;
            gl::GenVertexArrays(1, &mut vertex_array);
            gl::BindVertexArray(vertex_array);

            let mut vertex_buffer = 0;
            gl::GenBuffers(1, &mut vertex_buffer);
            gl::BindBuffer(gl::ARRAY_BUFFER, vertex_buffer);
            gl::BufferData(gl::ARRAY_BUFFER,
                           mem::size_of::<Vertex>() as GLsizeiptr * 4,
                           VERTICES.as_ptr() as *const c_void,
                           gl::STATIC_DRAW);

            gl::VertexAttribPointer(position_attribute as GLuint,
                                    2,
                                    gl::FLOAT,
                                    gl::FALSE,
                                    mem::size_of::<Vertex>() as GLsizei,
                                    (mem::size_of::<f32>() * 0) as *const GLvoid);
            gl::VertexAttribPointer(tex_coord_attribute as GLuint,
                                    2,
                                    gl::FLOAT,
                                    gl::FALSE,
                                    mem::size_of::<Vertex>() as GLsizei,
                                    (mem::size_of::<f32>() * 2) as *const GLvoid);
            gl::EnableVertexAttribArray(position_attribute as GLuint);
            gl::EnableVertexAttribArray(tex_coord_attribute as GLuint);

            Context {
                vertex_shader: vertex_shader,
                fragment_shader: fragment_shader,
                program: program,
                texture_uniform: texture_uniform,
                vertex_array: vertex_array,
                vertex_buffer: vertex_buffer,
            }
        }
    }

    /// Draws the given texture to the full viewport.
    ///
    /// *The texture must be of `GL_TEXTURE_RECTANGLE` type, not `GL_TEXTURE_2D`.* (This is for
    /// compatibility with macOS, which can only bind `IOSurface`s to texture rectangles.)
    ///
    /// If you want to draw to a subrect, simply call `gl::Viewport()` before calling this. If you
    /// want to draw only a portion of the texture, set the scissor box with `gl::Scissor()` and
    /// enable it with `gl::Enable(gl::SCISSOR_TEST)` before calling this. You can also use the
    /// stencil buffer for more advanced effects.
    ///
    /// Remember to set magnification and minification filters on the texture first
    /// (`GL_TEXTURE_MIN_FILTER` and `GL_TEXTURE_MAG_FILTER`).
    ///
    /// The same context that was current at the time `Context::new()` was called must be current
    /// at the time this is called.
    pub fn draw(&self, texture: GLuint) {
        unsafe {
            gl::UseProgram(self.program);
            gl::BindVertexArray(self.vertex_array);

            gl::BindBuffer(gl::ARRAY_BUFFER, self.vertex_buffer);

            gl::ActiveTexture(gl::TEXTURE0);
            gl::BindTexture(gl::TEXTURE_RECTANGLE, texture);
            gl::Uniform1i(self.texture_uniform, 0);

            gl::DrawArrays(gl::TRIANGLE_STRIP, 0, 4);
        }
    }
}

impl Drop for Context {
    fn drop(&mut self) {
        unsafe {
            gl::DeleteBuffers(1, &mut self.vertex_buffer);
            gl::DeleteVertexArrays(1, &mut self.vertex_array);
            gl::DeleteProgram(self.program);
            gl::DeleteShader(self.fragment_shader);
            gl::DeleteShader(self.vertex_shader);
        }
    }
}

#[repr(C)]
#[derive(Clone, Copy)]
struct Vertex {
    x: f32,
    y: f32,
    u: f32,
    v: f32,
}

static VERTICES: [Vertex; 4] = [
    Vertex { x: -1.0, y:  1.0, u: 0.0, v: 0.0 },
    Vertex { x:  1.0, y:  1.0, u: 1.0, v: 0.0 },
    Vertex { x: -1.0, y: -1.0, u: 0.0, v: 1.0 },
    Vertex { x:  1.0, y: -1.0, u: 1.0, v: 1.0 },
];

static VERTEX_SHADER: &'static str = r#"
#version 330

in vec2 aPosition;
in vec2 aTexCoord;

out vec2 vTexCoord;

void main() {
    vTexCoord = aTexCoord;
    gl_Position = vec4(aPosition, 0.0, 1.0);
}
"#;

static FRAGMENT_SHADER: &'static str = r#"
#version 330

uniform sampler2DRect uTexture;

in vec2 vTexCoord;

out vec4 oFragColor;

void main() {
    ivec2 size = textureSize(uTexture);
    oFragColor = texture(uTexture, vTexCoord * vec2(float(size.x), float(size.y)));
}
"#;

