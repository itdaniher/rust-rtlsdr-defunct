// Copyright 2013 The gl-rs developers & Ian Daniher.
//
// Licensed under the Apache License, Version 2.0 (the "License");
// you may not use this file except in compliance with the License.
// You may obtain a copy of the License at
//
//	 http://www.apache.org/licenses/LICENSE-2.0
//
// Unless required by applicable law or agreed to in writing, software
// distributed under the License is distributed on an "AS IS" BASIS,
// WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
// See the License for the specific language governing permissions and
// limitations under the License.

#[feature(globs)];
#[feature(macro_rules)];

extern mod glfw;
extern mod gl;

use std::rt;
use std::cast;
use std::ptr;
use std::str;
use std::mem;
use std::vec;
use std::num;
use std::comm;
use std::cell;

use gl::types::*;

static VERTEX_DATA: [GLfloat, ..16] = [
	-1.0,  1.0, 0.0, 0.0, // Top-left
	 1.0,  1.0, 1.0, 0.0, // Top-right
	 1.0, -1.0, 1.0, 1.0, // Bottom-right
	-1.0, -1.0, 0.0, 1.0  // Bottom-left
];

static ELEMENT_DATA: [GLuint, ..6] = [
	0, 1, 2,
	2, 3, 0
];

// Shader sources
static VS_SRC: &'static str =
   "#version 150\n\
	in vec2 position;\n\
	in vec2 texcoord;\n\
	out vec2 Texcoord;\n\
	void main() {\n\
	   Texcoord = texcoord;\n\
	   gl_Position = vec4(position, 0.0, 1.0);\n\
	}";

static FS_SRC: &'static str =
   "#version 150\n\
	in vec2 Texcoord;\n\
	out vec4 out_color;\n\
	uniform sampler2D data;\n\
	void main() {\n\
	   out_color = texture(data, Texcoord);\n\
	}";

fn compile_shader(src: &str, ty: GLenum) -> GLuint {
	let shader = gl::CreateShader(ty);
	unsafe {
		// Attempt to compile the shader
		src.with_c_str(|ptr| gl::ShaderSource(shader, 1, &ptr, ptr::null()));
		gl::CompileShader(shader);

		// Get the compile status
		let mut status = gl::FALSE as GLint;
		gl::GetShaderiv(shader, gl::COMPILE_STATUS, &mut status);

		// Fail on error
		if status != (gl::TRUE as GLint) {
			let mut len = 0;
			gl::GetShaderiv(shader, gl::INFO_LOG_LENGTH, &mut len);
			let mut buf = vec::from_elem(len as uint - 1, 0u8);	 // subtract 1 to skip the trailing null character
			gl::GetShaderInfoLog(shader, len, ptr::mut_null(), vec::raw::to_mut_ptr(buf) as *mut GLchar);
			fail!(str::raw::from_utf8(buf));
		}
	}
	shader
}

fn link_program(vs: GLuint, fs: GLuint) -> GLuint {
	let program = gl::CreateProgram();
	gl::AttachShader(program, vs);
	gl::AttachShader(program, fs);
	gl::LinkProgram(program);
	unsafe {
		// Get the link status
		let mut status = gl::FALSE as GLint;
		gl::GetProgramiv(program, gl::LINK_STATUS, &mut status);

		// Fail on error
		if status != (gl::TRUE as GLint) {
			let mut len: GLint = 0;
			gl::GetProgramiv(program, gl::INFO_LOG_LENGTH, &mut len);
			let mut buf = vec::from_elem(len as uint - 1, 0u8);	 // subtract 1 to skip the trailing null character
			gl::GetProgramInfoLog(program, len, ptr::mut_null(), vec::raw::to_mut_ptr(buf) as *mut GLchar);
			fail!(str::raw::from_utf8(buf));
		}
	}
	program
}


pub fn doWorkWithPEs (pDataC: comm::Port<~[f32]>, x: uint, y: uint) {

	do glfw::set_error_callback |_, description| {
		println!("GLFW Error: {}", description);
	}

	do glfw::start {
		glfw::window_hint::context_version(3, 2);
		glfw::window_hint::opengl_profile(glfw::OpenGlCoreProfile);
		glfw::window_hint::opengl_forward_compat(true);

		let window = glfw::Window::create(1024, 640, "OpenGL", glfw::Windowed).unwrap();
		window.make_context_current();

		// Load the OpenGL function pointers
		gl::load_with(glfw::get_proc_address);

		// Create GLSL shaders
		let vs = compile_shader(VS_SRC, gl::VERTEX_SHADER);
		let fs = compile_shader(FS_SRC, gl::FRAGMENT_SHADER);
		let program = link_program(vs, fs);

		let mut vao = 0;
		let mut vbo = 0;
		let mut ebo = 0;
		let mut tex = 0;

		unsafe {
			let mut data: ~[f32] = range(0, x*y).map(|x| num::sin(x as f32/1e5)).collect();
			// Create Vertex Array Object
			gl::GenVertexArrays(1, &mut vao);
			gl::BindVertexArray(vao);

			// Create a Vertex Buffer Object and copy the vertex data to it
			gl::GenBuffers(1, &mut vbo);
			gl::BindBuffer(gl::ARRAY_BUFFER, vbo);
			gl::BufferData(gl::ARRAY_BUFFER,
						(VERTEX_DATA.len() * mem::size_of::<GLfloat>()) as GLsizeiptr,
						cast::transmute(&VERTEX_DATA[0]),
						gl::STATIC_DRAW);

			// Create a Element Buffer Object and copy the element data to it
			gl::GenBuffers(1, &mut ebo);
			gl::BindBuffer(gl::ELEMENT_ARRAY_BUFFER, ebo);
			gl::BufferData(gl::ELEMENT_ARRAY_BUFFER,
						(ELEMENT_DATA.len() * mem::size_of::<GLuint>()) as GLsizeiptr,
						cast::transmute(&ELEMENT_DATA[0]),
						gl::STATIC_DRAW);

			// Use shader program
			gl::UseProgram(program);
			"out_color".with_c_str(|ptr| gl::BindFragDataLocation(program, 0, ptr));

			// Specify the layout of the vertex data
			let pos_attr = "position".with_c_str(|ptr| gl::GetAttribLocation(program, ptr));
			gl::EnableVertexAttribArray(pos_attr as GLuint);
			gl::VertexAttribPointer(pos_attr as GLuint, 2, gl::FLOAT,
									gl::FALSE as GLboolean, 16, ptr::null());

			let tex_attr = "texcoord".with_c_str(|ptr| gl::GetAttribLocation(program, ptr));
			gl::EnableVertexAttribArray(tex_attr as GLuint);
			gl::VertexAttribPointer(tex_attr as GLuint, 2, gl::FLOAT,
									gl::FALSE as GLboolean, 16, cast::transmute(8));

			gl::GenTextures(1, &mut tex);
			gl::ActiveTexture(gl::TEXTURE0);
			gl::BindTexture(gl::TEXTURE_2D, 0);
			"data".with_c_str(|ptr| gl::Uniform1i(gl::GetUniformLocation(program, ptr), 0));
			gl::TexImage2D(gl::TEXTURE_2D, 0, gl::RED as i32, x as i32, y as i32, 0, gl::RED, gl::FLOAT, cast::transmute(data.unsafe_mut_ref(0)));
			gl::TexParameteri(gl::TEXTURE_2D, gl::TEXTURE_WRAP_S, gl::CLAMP_TO_EDGE as i32);
			gl::TexParameteri(gl::TEXTURE_2D, gl::TEXTURE_WRAP_T, gl::CLAMP_TO_EDGE as i32);
			gl::TexParameteri(gl::TEXTURE_2D, gl::TEXTURE_MIN_FILTER, gl::NEAREST as i32);
			gl::TexParameteri(gl::TEXTURE_2D, gl::TEXTURE_MAG_FILTER, gl::NEAREST as i32);

			while !window.should_close() {

				glfw::poll_events();
				let (width, height) = window.get_size();
				gl::Viewport(0, 0, width as i32, height as i32);
				gl::ClearColor(0.0, 0.0, 0.0, 0.0);
				gl::Clear(gl::COLOR_BUFFER_BIT);

				gl::DrawElements(gl::TRIANGLES, 6, gl::UNSIGNED_INT, ptr::null());

				if pDataC.peek() {
					data = pDataC.recv();
					gl::BindTexture(gl::TEXTURE_2D, 0);
					gl::TexSubImage2D(gl::TEXTURE_2D, 0, 0, 0, 1024, (data.len()/1024) as i32, gl::RED, gl::FLOAT, cast::transmute(data.unsafe_mut_ref(0)));
				}

				// Swap buffers
				window.swap_buffers();
			}
			// Cleanup
			gl::DeleteProgram(program);
			gl::DeleteShader(fs);
			gl::DeleteShader(vs);
			gl::DeleteBuffers(1, &vbo);
			gl::DeleteBuffers(1, &ebo);
			gl::DeleteVertexArrays(1, &vao);
		}
	}
}

pub fn spawnVectorVisualSink(x: uint, y: uint) -> std::comm::Chan<~[f32]> {

	let (pData, cData): (comm::Port<~[f32]>, comm::Chan<~[f32]>) = comm::stream();

	let argx = cell::Cell::new(x);
	let argy = cell::Cell::new(y);
	let argpData = cell::Cell::new(pData);

	do spawn {
		doWorkWithPEs(argpData.take(), argx.take(), argy.take());
	}
	return cData;
}
