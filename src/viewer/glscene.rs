extern crate epoxy;
extern crate gio;
extern crate gl;
extern crate gtk;

use dylib::DynamicLibrary;
use gl::types::*;
use std::convert::TryInto;
use std::ffi::CStr;
use std::mem;
use std::process::exit;
use std::ptr;

const DEFAULT_VERTEX_SHADER: &'static str = r#"
#version 140
in vec2 position;
in vec3 color;
out vec3 vertex_color;
void main() {
    vertex_color = color;
    gl_Position = vec4(position, 0.0, 1.0);
}"#;

const DEFAULT_FRAGMENT_SHADER: &'static str = r#"
#version 140
in vec3 vertex_color;
out vec4 color;
void main() {
    color = vec4(vertex_color, 1.0);
}"#;

pub struct GLScene {
    pub vertices: Vec<GLfloat>,
}

impl GLScene {
    pub fn new(vertices: &[GLfloat]) -> GLScene {
        GLScene {
            vertices: vertices.to_vec(),
        }
    }
    pub fn load_vertices(&self) {
        let vertices = &self.vertices;
        // Load epoxy to be able to interact with the GTK OpenGL context
        epoxy::load_with(|s| unsafe {
            match DynamicLibrary::open(None).unwrap().symbol(s) {
                Ok(v) => v,
                Err(_) => ptr::null(),
            }
        });
        gl::load_with(epoxy::get_proc_addr);

        // Load shaders
        let program = make_program(DEFAULT_VERTEX_SHADER, DEFAULT_FRAGMENT_SHADER);
        let mut vao: GLuint = 0;
        let mut vbo: GLuint = 0;

        // Load Vertices
        unsafe {
            gl::GenVertexArrays(1, &mut vao);
            gl::BindVertexArray(vao);

            gl::GenBuffers(1, &mut vbo);
            gl::BindBuffer(epoxy::ARRAY_BUFFER, vbo);
            gl::BufferData(
                epoxy::ARRAY_BUFFER,
                (vertices.len() * mem::size_of::<GLfloat>()) as GLsizeiptr,
                mem::transmute(&vertices[0]),
                epoxy::STATIC_DRAW,
            );

            gl::UseProgram(program);
            gl::BindFragDataLocation(program, 0, b"color\0".as_ptr() as *const GLchar);

            let pos_attr = gl::GetAttribLocation(program, b"position\0".as_ptr() as *const GLchar);
            gl::EnableVertexAttribArray(pos_attr as GLuint);
            gl::VertexAttribPointer(
                pos_attr as GLuint,
                2,
                epoxy::FLOAT,
                epoxy::FALSE as GLboolean,
                (5 * mem::size_of::<GLfloat>()) as GLint,
                ptr::null(),
            );

            let color_attr = gl::GetAttribLocation(program, b"color\0".as_ptr() as *const GLchar);
            gl::EnableVertexAttribArray(color_attr as GLuint);
            gl::VertexAttribPointer(
                color_attr as GLuint,
                3,
                epoxy::FLOAT,
                epoxy::FALSE as GLboolean,
                (5 * mem::size_of::<GLfloat>()) as GLint,
                (2 * mem::size_of::<GLfloat>()) as *const GLvoid,
            );
        }
    }

    pub fn render(&self) {
        unsafe {
            gl::ClearColor(0.3, 0.3, 0.3, 1.0);
            gl::Clear(epoxy::COLOR_BUFFER_BIT);

            gl::DrawArrays(epoxy::TRIANGLES, 0, self.vertices.len().try_into().unwrap());
        };
    }
}

fn compile_shader(src: &str, ty: GLenum) -> Result<GLuint, String> {
    unsafe {
        let shader = gl::CreateShader(ty);
        // Attempt to compile the shader
        let psrc = src.as_ptr() as *const GLchar;
        let len = src.len() as GLint;
        gl::ShaderSource(shader, 1, &psrc, &len);
        gl::CompileShader(shader);

        // Get the compile status
        let mut status = epoxy::FALSE as GLint;
        gl::GetShaderiv(shader, epoxy::COMPILE_STATUS, &mut status);

        // Fail on error
        if status != (epoxy::TRUE as GLint) {
            let mut len = 0;
            gl::GetShaderiv(shader, epoxy::INFO_LOG_LENGTH, &mut len);
            let mut buf = vec![0i8; len as usize];
            gl::GetShaderInfoLog(
                shader,
                len,
                ptr::null_mut(),
                buf.as_mut_ptr() as *mut GLchar,
            );
            return Err(CStr::from_ptr(buf.as_ptr()).to_string_lossy().into_owned());
        }

        Ok(shader)
    }
}

fn make_program(vert_shader_src: &str, frag_shader_src: &str) -> GLuint {
    let vs = match compile_shader(vert_shader_src, epoxy::VERTEX_SHADER) {
        Ok(v) => v,
        Err(e) => {
            fatal_error(&*format!("Error compiling vertex shader: {}", e));
            0
        }
    };
    let fs = match compile_shader(frag_shader_src, epoxy::FRAGMENT_SHADER) {
        Ok(v) => v,
        Err(e) => {
            fatal_error(&*format!("Error compiling fragment shader: {}", e));
            0
        }
    };

    let program = match link_program(vs, fs) {
        Ok(v) => v,
        Err(e) => {
            fatal_error(&*format!("Error linking shader: {}", e));
            0
        }
    };

    program
}

fn link_program(vs: GLuint, fs: GLuint) -> Result<GLuint, String> {
    unsafe {
        let program = gl::CreateProgram();
        gl::AttachShader(program, vs);
        gl::AttachShader(program, fs);
        gl::LinkProgram(program);

        // Get the link status
        let mut status = epoxy::FALSE as GLint;
        gl::GetProgramiv(program, epoxy::LINK_STATUS, &mut status);

        // Fail on error
        if status != (epoxy::TRUE as GLint) {
            let mut len: GLint = 0;
            gl::GetProgramiv(program, epoxy::INFO_LOG_LENGTH, &mut len);
            let mut buf = vec![0i8; len as usize];
            gl::GetProgramInfoLog(
                program,
                len,
                ptr::null_mut(),
                buf.as_mut_ptr() as *mut GLchar,
            );
            return Err(CStr::from_ptr(buf.as_ptr()).to_string_lossy().into_owned());
        }

        Ok(program)
    }
}

fn fatal_error(message: &str) {
    println!("{}", message);

    // Can't gtk::main_quit as main loop isn't running, call exit
    exit(1);
}
