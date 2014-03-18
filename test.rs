#[feature(globs)];
#[feature(macro_rules)];
#[feature(link_args)];

extern crate glfw = "glfw-rs";
extern crate gl;
extern crate native;
extern crate cgmath;

use std::cast;
use std::ptr;
use std::str;
use std::mem;
use std::vec;
use std::io::File;

use cgmath::matrix::*;
use cgmath::vector::*;
use cgmath::ptr::*;
use cgmath::angle::*;
use cgmath::projection::*;

use gl::types::*;

static MAP_SRC: &'static str = "elevation.data";
static MAP_W: uint = 2048;
static MAP_H: uint = 2048;
static MAP_SIZE: uint = MAP_W * MAP_H;

static SCALE:   f32 = 2048f32;
static SCALE_X: f32 = SCALE;
static SCALE_Y: f32 = SCALE;
static SCALE_Z: f32 = 1.0;

// Shader sources
static VS_SRC: &'static str = "test.vert";
static FS_SRC: &'static str = "test.frag";

#[start]
fn start(argc: int, argv: **u8) -> int {
  native::start(argc, argv, main)
}

fn load_heightmap() -> ~[u8] {
  let p = std::os::getcwd().join(Path::new(MAP_SRC));
  match File::open(&p).read_bytes(MAP_SIZE) {
    Ok(res) => return res,
    Err(_) => fail!("Could not load heightmap!")
  }
}

// fn initialize_indices() -> ~[Vec3<GLfloat>] {
//   let mut indices: ~[Vec3<GLfloat>] = ~[];
//   let mut i = 0;
//
//   for row in range(0, MAP_W-1) {
//     for col in range(0, MAP_H-1) {
//
//       let i0 = row * MAP_W + col;
//       let i1 = i0 + MAP_H;
//
//       indices.push_all(~[
//         i0   as f32,
//         i0+1 as f32,
//         i1   as f32,
//         i1   as f32,
//         i1+1 as f32,
//         i0+1 as f32
//       ]);
//     }
//   }
//   indices
// }

fn initialize_vertices(heightmap: ~[u8]) -> ~[Vec3<GLfloat>] {
  let mut vertices: ~[Vec3<GLfloat>] = ~[];
  let mut i = 0;

  for x in range(0, MAP_W-1) {
    for y in range(0, MAP_H-1) {

      let xi = x as f32 / SCALE_X;
      let yi = y as f32 / SCALE_Y;
      let zi = heightmap[x * MAP_W + y] as f32 / SCALE_Z;

      let sx = 1.0f32 / SCALE_X; // Step x
      let sy = 1.0f32 / SCALE_Y; // Step y

      let pos: [(f32, f32, f32), ..6] = [
        (xi   , yi   , zi),
        (xi   , yi+sy, zi),
        (xi+sx, yi   , zi),
        (xi   , yi+sy, zi),
        (xi+sx, yi   , zi),
        (xi+sx, yi+sy, zi)
      ];

      //println!("grid: ({}, {}, {})", xi, yi, zi);

      for tup in pos.iter() {
        let (x, y, z) = *tup;
        let v = Vec3::new(x as GLfloat, y as GLfloat, z as GLfloat);
        vertices.push(v);
        // if z > 0.0 { println!("  {}", v); }
      }
    }
  }
  vertices
}

fn initialize_normals(v: ~[Vec3<GLfloat>]) -> ~[Vec3<GLfloat>] {
  let mut normals: ~[Vec3<GLfloat>] = ~[];

  for row in range(0, MAP_W-1) {
    for col in range(0, MAP_H-1) {

      let hr = MAP_W * row;
      let hc = col;

      let mut sum = Vec3::new(0f32, 0f32, 0f32);
      let cur = v[hr+hc];

      if row+1 < MAP_W && col+1 < MAP_H {
        sum = sum + (v[hr+0 + hc+1] - cur).cross(&(v[hr+1 + hc+0] - cur)).normalize();
      }

      if row+1 < MAP_W && col > 0 {
        sum = sum + (v[hr+1 + hc+0] - cur).cross(&(v[hr+0 + hc+1] - cur)).normalize();
      }

      if row > 0 && col > 0 {
        sum = sum + (v[hr+0 + hc+1] - cur).cross(&(v[hr+1 + hc+0] - cur)).normalize();
      }

      if row > 0 && col+1 < MAP_H {
        sum = sum + (v[hr+1 + hc+0] - cur).cross(&(v[hr+0 + hc+1] - cur)).normalize();
      }

      normals.push(sum.normalize());
    }
  }
  normals
}

// -- -- -- -- -- -- -- -- -- -- -- -- -- -- -- -- -- -- -- -- -- -- -- -- -- --

fn load_shader_file(file_name: &str) -> ~str {
  let p = std::os::getcwd().join(Path::new(file_name));
  match File::open(&p).read_to_end() {
    Ok(s) => str::from_utf8_owned(s).unwrap(),
    Err(_) => fail!("Could not read shader file!")
  }
}

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
      let mut buf = vec::from_elem(len as uint - 1, 0u8);     // subtract 1 to skip the trailing null character
      gl::GetShaderInfoLog(shader, len, ptr::mut_null(), buf.as_mut_ptr() as *mut GLchar);
      fail!(str::raw::from_utf8(buf).to_owned());
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
      let mut buf = vec::from_elem(len as uint - 1, 0u8);     // subtract 1 to skip the trailing null character
      gl::GetProgramInfoLog(program, len, ptr::mut_null(), buf.as_mut_ptr() as *mut GLchar);
      fail!(str::raw::from_utf8(buf).to_owned());
    }
  }
  program
}

fn main() {

  let vs_src = load_shader_file(VS_SRC);
  let fs_src = load_shader_file(FS_SRC);

  let mut heightmap: ~[u8] = load_heightmap();
  let vertices = initialize_vertices(heightmap);
  //let normals  = initialize_normals(vertices.clone());

  glfw::set_error_callback(~ErrorContext);

  glfw::start(proc() {

    // Choose a GL profile that is compatible with OS X 10.7+
    glfw::window_hint::context_version(3, 2);
    glfw::window_hint::opengl_profile(glfw::OpenGlCoreProfile);
    glfw::window_hint::opengl_forward_compat(true);

    let window = glfw::Window::create(800, 600, "OpenGL", glfw::Windowed).unwrap();
    window.make_context_current();

    // Load the OpenGL function pointers
    gl::load_with(glfw::get_proc_address);

    // Create GLSL shaders
    let vertex_shader = compile_shader(vs_src, gl::VERTEX_SHADER);
    let fragment_shader = compile_shader(fs_src, gl::FRAGMENT_SHADER);
    let shader_program = link_program(vertex_shader, fragment_shader);

    let mut vertex_array_object = 0;
    let mut vertex_buffer_object = 0;

    unsafe {

      // Create Vertex Array Object and Vertex Buffer Objects
      gl::GenVertexArrays(1, &mut vertex_array_object);
      gl::GenBuffers(1, &mut vertex_buffer_object);

      gl::BindVertexArray(vertex_array_object);

      // Create Vertex Buffer Object 1 for the vertex position data
      gl::BindBuffer(gl::ARRAY_BUFFER, vertex_buffer_object);
      gl::BufferData(gl::ARRAY_BUFFER,
                     (vertices.len() * mem::size_of::<Vec3<GLfloat>>()) as GLsizeiptr,
                     cast::transmute(&vertices[0]),
                     gl::STATIC_DRAW);

      // Create Vertex Buffer Object 2 for the vertex position normals
      // gl::BindBuffer(gl::ARRAY_BUFFER, vbo_2);
      // gl::BufferData(gl::ARRAY_BUFFER,
      //                (normals.len() * mem::size_of::<Vec3<GLfloat>>()) as GLsizeiptr,
      //                cast::transmute(&normals[0]),
      //                gl::STATIC_DRAW);

      // Use shader program
      gl::UseProgram(shader_program);
      "out_color".with_c_str(|ptr| gl::BindFragDataLocation(shader_program, 0, ptr));

      let pos_attr = "position".with_c_str(|ptr| gl::GetAttribLocation(shader_program, ptr));
      //let nrm_attr = "normal".with_c_str(|ptr| gl::GetAttribLocation(shader_program, ptr));

      gl::EnableVertexAttribArray(pos_attr as GLuint);
      //gl::EnableVertexAttribArray(nrm_attr as GLuint);
      gl::VertexAttribPointer(pos_attr as GLuint, 3, gl::FLOAT, gl::FALSE, 0, ptr::null());
      //gl::VertexAttribPointer(nrm_attr as GLuint, 3, gl::FLOAT, gl::FALSE as GLboolean, 0, ptr::null());

    }

    while !window.should_close() {
      // Poll events
      glfw::poll_events();

      // Clear the screen to black
      gl::ClearColor(0.3, 0.3, 0.3, 1.0);
      gl::Clear(gl::COLOR_BUFFER_BIT);

      // Draw a triangle from the 3 vertices
      gl::DrawArrays(gl::TRIANGLES, 0, vertices.len() as GLint);

      // Swap buffers
      window.swap_buffers();
    }

    // Cleanup
    gl::DeleteProgram(shader_program);
    gl::DeleteShader(fragment_shader);
    gl::DeleteShader(vertex_shader);
    unsafe {
      gl::DeleteBuffers(1, &vertex_buffer_object);
      gl::DeleteVertexArrays(1, &vertex_array_object);
    }
  });
}

struct ErrorContext;
impl glfw::ErrorCallback for ErrorContext {
    fn call(&self, _: glfw::Error, description: ~str) {
        println!("GLFW Error: {:s}", description);
    }
}
