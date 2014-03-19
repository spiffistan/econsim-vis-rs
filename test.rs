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
use std::io::stdio::flush;

use cgmath::matrix::*;
use cgmath::vector::*;
use cgmath::angle::*;
use cgmath::ptr::*;
use cgmath::angle::*;
use cgmath::projection::*;

use gl::types::*;

static DEBUG: bool = true;

static MAP_SRC: &'static str = "elevation.data";
static MAP_W: uint = 2048;
static MAP_H: uint = 2048;
static MAP_SIZE: uint = MAP_W * MAP_H;

static SCALE:   f32 = 2048.0;
static SCALE_X: f32 = SCALE;
static SCALE_Y: f32 = SCALE;
static SCALE_Z: f32 = 1.0;

// Shader sources
static VS_SRC: &'static str = "test.vert";
static FS_SRC: &'static str = "test.frag";


// -- -- -- -- -- -- -- -- -- -- -- -- -- -- -- -- -- -- -- -- -- -- -- -- -- --

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

// TODO XXX
fn move_camera() {

}

fn initialize_vertices(heightmap: ~[u8]) -> ~[Vec4<GLfloat>] {
  let mut vertices: ~[Vec4<GLfloat>] = ~[];

  for x in range(0, MAP_W) {
    for y in range(0, MAP_H) {

      let xi = x as f32 / SCALE_X;
      let yi = y as f32 / SCALE_Y;
      let zi = heightmap[x * MAP_W + y] as f32 / SCALE_Z;
      let wi = 0.0;

      let v = Vec4::new(xi, yi, zi, wi);
      vertices.push(v);

      // println!("+ ({}, {}, {})", xi, yi, zi)
    }
  }
  vertices
}

fn initialize_indices() -> ~[u32] {
  let mut indices: ~[u32] = ~[];

  for x in range(0, MAP_W-1) {
    for y in range(0, MAP_H-1) {

      let start = (x * MAP_W + y);
      let offset = MAP_H;

      indices.push_all(&[
        // Triangle 1
        start as u32,
        (start + 1) as u32,
        (start + offset) as u32,
        // Triangle 2
        (start + 1) as u32,
        (start + 1 + offset) as u32,
        (start + offset) as u32
      ]);
    }
  }
  indices
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

  // let mut field_of_view:      f32 = 60.0;
  // let mut aspect_ratio:       f32 = MAP_W as f32 / MAP_H as f32;
  // let mut near_plane:         f32 = 0.1;
  // let mut far_plane:          f32 = 100.0;
  // let mut frustum_length:     f32 = far_plane - near_plane;
  // let mut y_scale:            f32 = cot(deg(field_of_view / 2.0).to_rad());
  // let mut x_scale:            f32 = y_scale / aspect_ratio;
  // let mut matrix_44_buf:      ~[GLfloat] = ~[0f32, ..16];
  //
  // let mut view_matrix:        Mat4<GLfloat> = Mat4::zero();
  // let mut model_matrix:       Mat4<GLfloat> = Mat4::zero();
  //
  // let c2r2: f32 = -((far_plane + near_plane) / frustum_length);
  // let c3r2: f32 = -((2.0 * near_plane * far_plane) / frustum_length);
  //
  // let mut projection_matrix:  Mat4<GLfloat> = Mat4::new(
  //   x_scale, 0.0,     0.0,      0.0,
  //   0.0,     y_scale, 0.0,      0.0,
  //   0.0,     0.0,     c2r2,    -1.0,
  //   0.0,     0.0,     c3r2,     0.0
  // );

  // cgmath doesn't seem to be able to update individual cells (yet?)
  // projection_matrix.c0r0 = x_scale;
  // projection_matrix.c1r1 = y_scale;
  // projection_matrix.c2r2 = -((far_plane + near_plane) / frustum_length);
  // projection_matrix.c2r3 = -1.0;
  // projection_matrix.c3r2 = -((2 * near_plane * far_plane) / frustum_length);
  // projection_matrix.c3r3 = 0.0;

  if DEBUG { print!("Loading heightmap from {}... ", MAP_SRC); flush(); }
  let mut heightmap: ~[u8] = load_heightmap();
  if DEBUG { println!("done. ({} bytes)", heightmap.len()) }

  if DEBUG { print!("Computing vertices... "); flush(); }
  let vertices = initialize_vertices(heightmap);
  if DEBUG { println!("done. ({} vertices)", vertices.len()) }

  if DEBUG { print!("Computing indices... "); flush(); }
  let indices = initialize_indices();
  if DEBUG { println!("done. ({} indices)", indices.len()) }

  // if DEBUG { print!("Computing normals... "); flush(); }
  // let normals  = initialize_normals(vertices.clone());
  // if DEBUG { println!("done. ({} normals)", normals.len()) }

  let vs_src = load_shader_file(VS_SRC);
  let fs_src = load_shader_file(FS_SRC);

  glfw::set_error_callback(~ErrorContext);

  glfw::start(proc() {

    // Choose a GL profile that is compatible with OS X 10.7+
    glfw::window_hint::context_version(3, 2);
    glfw::window_hint::opengl_profile(glfw::OpenGlCoreProfile);
    glfw::window_hint::opengl_forward_compat(true);

    let window = glfw::Window::create(1920, 1280, "OpenGL", glfw::Windowed).unwrap();
    window.make_context_current();

    // Load the OpenGL function pointers
    gl::load_with(glfw::get_proc_address);

    // Create GLSL shaders
    let vertex_shader = compile_shader(vs_src, gl::VERTEX_SHADER);
    let fragment_shader = compile_shader(fs_src, gl::FRAGMENT_SHADER);
    let shader_program = link_program(vertex_shader, fragment_shader);

    let mut vertex_array_id = 0;
    let mut vertex_buffer_id = 1;
    let mut index_buffer_id = 2;

    unsafe {

      // Create Vertex Array Object and Vertex Buffer Objects
      gl::GenVertexArrays(1, &mut vertex_array_id);

      gl::BindVertexArray(vertex_array_id);

      // Initialize vertex positions ///////////////////////////////////////////
      let vertices_bytes = (vertices.len() * mem::size_of::<Vec4<GLfloat>>()) as GLsizeiptr;
      let vertices_ptr = cast::transmute(&vertices[0]);

      gl::GenBuffers(1, &mut vertex_buffer_id);
      gl::BindBuffer(gl::ARRAY_BUFFER, vertex_buffer_id);
      gl::BufferData(gl::ARRAY_BUFFER, vertices_bytes, vertices_ptr, gl::STATIC_DRAW);

      // Initialize vertex indices /////////////////////////////////////////////
      let indices_bytes = (indices.len() * mem::size_of::<u32>()) as GLsizeiptr;
      let indices_ptr = cast::transmute(&indices[0]);

      gl::GenBuffers(1, &mut index_buffer_id);
      gl::BindBuffer(gl::ELEMENT_ARRAY_BUFFER, index_buffer_id);
      gl::BufferData(gl::ELEMENT_ARRAY_BUFFER, indices_bytes, indices_ptr, gl::STATIC_DRAW);

      // Create Vertex Buffer Object 2 for the vertex position normals
      // gl::BindBuffer(gl::ARRAY_BUFFER, vertex_buffer_object_2);
      // gl::BufferData(gl::ARRAY_BUFFER,
      //                (normals.len() * mem::size_of::<Vec3<GLfloat>>()) as GLsizeiptr,
      //                cast::transmute(&normals[0]),
      //                gl::STATIC_DRAW);

      // Use shader program
      gl::UseProgram(shader_program);
      "out_color".with_c_str(|ptr| gl::BindFragDataLocation(shader_program, 0, ptr));

      let position_p = "position".with_c_str(|ptr| gl::GetAttribLocation(shader_program, ptr));
      //let normal_p = "normal".with_c_str(|ptr| gl::GetAttribLocation(shader_program, ptr));

      gl::EnableVertexAttribArray(position_p as GLuint);
      //gl::EnableVertexAttribArray(normal_p as GLuint);
      gl::VertexAttribPointer(position_p as GLuint, 4, gl::FLOAT, gl::FALSE, 0, ptr::null());
      //gl::VertexAttribPointer(normal_p as GLuint, 1, gl::FLOAT, gl::FALSE, 0, ptr::null());
    }

    while !window.should_close() {
      // Poll events
      glfw::poll_events();

      // Clear the screen to black
      gl::ClearColor(0.3, 0.3, 0.3, 1.0);
      gl::Clear(gl::COLOR_BUFFER_BIT);

      unsafe { gl::DrawElements(gl::TRIANGLES, indices.len() as GLint, gl::UNSIGNED_INT, ptr::null()); }

      // Swap buffers
      window.swap_buffers();
    }

    // Cleanup
    gl::DeleteProgram(shader_program);
    gl::DeleteShader(fragment_shader);
    gl::DeleteShader(vertex_shader);

    unsafe {
      gl::DeleteBuffers(1, &vertex_buffer_id);
      gl::DeleteBuffers(1, &index_buffer_id);
      gl::DeleteVertexArrays(1, &vertex_array_id);
    }
  });
}

struct ErrorContext;
impl glfw::ErrorCallback for ErrorContext {
    fn call(&self, _: glfw::Error, description: ~str) {
        println!("GLFW Error: {:s}", description);
    }
}
