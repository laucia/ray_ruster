// extern crate epoxy;
// extern crate gio;
// extern crate gl;
// extern crate gtk;

// use dylib::DynamicLibrary;
// use std::ffi::CStr;
// use std::mem;
// use std::process::exit;
// use std::ptr;

// use gtk::GLArea;

// use gl::types::*;
// use gtk::traits::*;
// use super::glscene::GLScene;

// pub struct GLViewer {
//     pub glarea: gtk::GLArea,
//     pub glscene: super::glscene::GLScene,
// }

// impl GLViewer {
//     pub fn new(glscene: super::glscene::GLScene 'static) -> GLViewer {

//         GLViewer { glarea , glscene}
//     }

//     pub fn connect_realize(self: Box<Self>) {

//     }

//     pub fn connect_render(&self: Box<Self>) {
//         let glarea_connect = self.glarea.clone();

//         glarea_connect.connect_render(move |_, _| {
//             self.render();
//             gtk::Inhibit(false)
//         });
//     }
// }

// fn compile_shader(src: &str, ty: GLenum) -> Result<GLuint, String> {
//     unsafe {
//         let shader = gl::CreateShader(ty);
//         // Attempt to compile the shader
//         let psrc = src.as_ptr() as *const GLchar;
//         let len = src.len() as GLint;
//         gl::ShaderSource(shader, 1, &psrc, &len);
//         gl::CompileShader(shader);

//         // Get the compile status
//         let mut status = epoxy::FALSE as GLint;
//         gl::GetShaderiv(shader, epoxy::COMPILE_STATUS, &mut status);

//         // Fail on error
//         if status != (epoxy::TRUE as GLint) {
//             let mut len = 0;
//             gl::GetShaderiv(shader, epoxy::INFO_LOG_LENGTH, &mut len);
//             let mut buf = vec![0i8; len as usize];
//             gl::GetShaderInfoLog(
//                 shader,
//                 len,
//                 ptr::null_mut(),
//                 buf.as_mut_ptr() as *mut GLchar,
//             );
//             return Err(CStr::from_ptr(buf.as_ptr()).to_string_lossy().into_owned());
//         }

//         Ok(shader)
//     }
// }

// fn link_program(vs: GLuint, fs: GLuint) -> Result<GLuint, String> {
//     unsafe {
//         let program = gl::CreateProgram();
//         gl::AttachShader(program, vs);
//         gl::AttachShader(program, fs);
//         gl::LinkProgram(program);

//         // Get the link status
//         let mut status = epoxy::FALSE as GLint;
//         gl::GetProgramiv(program, epoxy::LINK_STATUS, &mut status);

//         // Fail on error
//         if status != (epoxy::TRUE as GLint) {
//             let mut len: GLint = 0;
//             gl::GetProgramiv(program, epoxy::INFO_LOG_LENGTH, &mut len);
//             let mut buf = vec![0i8; len as usize];
//             gl::GetProgramInfoLog(
//                 program,
//                 len,
//                 ptr::null_mut(),
//                 buf.as_mut_ptr() as *mut GLchar,
//             );
//             return Err(CStr::from_ptr(buf.as_ptr()).to_string_lossy().into_owned());
//         }

//         Ok(program)
//     }
// }

// fn fatal_error(message: &str) {
//     println!("{}", message);

//     // Can't gtk::main_quit as main loop isn't running, call exit
//     exit(1);
// }
