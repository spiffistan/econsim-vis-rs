#[feature(globs)];
#[feature(macro_rules)];
#[feature(link_args)];

extern crate glfw = "glfw-rs";
extern crate gl;
extern crate native;
// extern crate cgmath;

use std::cast;
use std::ptr;
use std::str;
use std::mem;
use std::vec;
use std::io::File;

use gl::types::*;

static MAP_SRC: &'static str = "elevation.data";
static MAP_W: uint = 2048;
static MAP_H: uint = 2048;
static MAP_SIZE: uint = MAP_W * MAP_H;

// Shader sources
static VS_SRC: &'static str = "test.vert";
static FS_SRC: &'static str = "test.frag";


// Since cgmath doesn't compile yet...
#[deriving(Eq, Clone)]
pub struct Vec3 {
  x: GLfloat,
  y: GLfloat,
  z: GLfloat
}

impl Vec3 {

  pub fn new(x: GLfloat, y: GLfloat, z: GLfloat) -> Vec3 {
    Vec3 { x: x, y: y, z: z }
  }

  pub fn cross(&self, other: &Vec3) -> Vec3 {
    Vec3::new(
      (self.y * other.z) - (self.z * other.y),
      (self.z * other.x) - (self.x * other.z),
      (self.x * other.y) - (self.y * other.x)
    )
  }

  pub fn dot(&self) -> GLfloat {
    self.x * self.x + self.y * self.y + self.z * self.z
  }

  pub fn normalize(&self) -> Vec3 {
    let len = std::f32::sqrt(self.dot());
    return Vec3::new(self.x / len, self.y / len, self.z / len);
  }
}

impl Add<Vec3, Vec3> for Vec3 {
  fn add(&self, other: &Vec3) -> Vec3 {
    Vec3::new(self.x + other.x, self.y + other.y, self.z + other.z)
  }
}

impl Sub<Vec3, Vec3> for Vec3 {
  fn sub(&self, other: &Vec3) -> Vec3 {
    Vec3::new(self.x - other.x, self.y - other.y, self.z - other.z)
  }
}

#[start]
fn start(argc: int, argv: **u8) -> int {
  native::start(argc, argv, main)
}

fn load_heightmap() -> ~[u8] {
  let p = std::os::getcwd().join(Path::new(MAP_SRC));
  match File::open(&p).read_bytes(MAP_SIZE) {
    Ok(res) => return res,
    Err(_) => fail!()
  }
}

fn initialize_vertices(heightmap: ~[u8]) -> ~[Vec3] {
  let mut vertices: ~[Vec3] = ~[];
  let mut i = 0;

  for x in range(0, MAP_W) {
    for y in range(0, MAP_H) {
      let v = Vec3::new(x as GLfloat, y as GLfloat, heightmap[(y * MAP_W) + x] as GLfloat);
      vertices.push(v);
      // vertices.push(x as GLfloat);
      // vertices.push(y as GLfloat);
      // vertices.push(heightmap[(y * MAP_W) + x] as GLfloat);
    }
  }
  vertices
}

fn initialize_normals(v: ~[Vec3]) -> ~[Vec3] {
  let mut normals: ~[Vec3] = ~[];

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
//
//     VecMat normals( hm );
//     for( int col = 0; col < hm.cols(); ++col )
//         for( int row = 0; row < hm.rows(); ++row )
//         {
//             Vector3f sum( Vector3f::Zero() );
//             const Vector3f& cur = hm( row, col );
//             if( row+1 < hm.rows() && col+1 < hm.cols() )
//                 sum += ( hm( row+0, col+1 ) - cur ).cross( hm( row+1, col+0 ) - cur ).normalized();
//             if( row+1 < hm.rows() && col > 0 )
//                 sum += ( hm( row+1, col+0 ) - cur ).cross( hm( row+0, col-1 ) - cur ).normalized();
//             if( row > 0 && col > 0 )
//                 sum += ( hm( row+0, col-1 ) - cur ).cross( hm( row-1, col+0 ) - cur ).normalized();
//             if( row > 0 && col+1 < hm.cols() )
//                 sum += ( hm( row-1, col+0 ) - cur ).cross( hm( row+0, col+1 ) - cur ).normalized();
//             normals( row, col ) = sum.normalized();
//         }
//     return normals;

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

  let mut heightmap: ~[u8] = ~[0u8, ..(MAP_SIZE * 3)];
  let vertices = initialize_vertices(heightmap);
  let normals  = initialize_normals(vertices.clone());

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
    let vs = compile_shader(vs_src, gl::VERTEX_SHADER);
    let fs = compile_shader(fs_src, gl::FRAGMENT_SHADER);
    let program = link_program(vs, fs);

    let mut vao = 0;
    let mut vbo = 0;

    unsafe {
      // Create Vertex Array Object
      gl::GenVertexArrays(1, &mut vao);
      gl::BindVertexArray(vao);

      // Create a Vertex Buffer Object and copy the vertex data to it
      gl::GenBuffers(1, &mut vbo);
      gl::BindBuffer(gl::ARRAY_BUFFER, vbo);
      gl::BufferData(gl::ARRAY_BUFFER,
                     (vertices.len() * mem::size_of::<GLfloat>()) as GLsizeiptr,
                     cast::transmute(&vertices[0]),
                     gl::STATIC_DRAW);

      // Use shader program
      gl::UseProgram(program);
      "out_color".with_c_str(|ptr| gl::BindFragDataLocation(program, 0, ptr));

      // Specify the layout of the vertex data
      let pos_attr = "position".with_c_str(|ptr| gl::GetAttribLocation(program, ptr));

      gl::EnableVertexAttribArray(pos_attr as GLuint);
      gl::VertexAttribPointer(pos_attr as GLuint, 3, gl::FLOAT, gl::FALSE as GLboolean, 0, ptr::null());
    }

    while !window.should_close() {
      // Poll events
      glfw::poll_events();

      // Clear the screen to black
      gl::ClearColor(0.3, 0.3, 0.3, 1.0);
      gl::Clear(gl::COLOR_BUFFER_BIT);

      // Draw a triangle from the 3 vertices
      gl::DrawArrays(gl::TRIANGLES, 0, (MAP_SIZE * 3) as i32);

      // Swap buffers
      window.swap_buffers();
    }

    // Cleanup
    gl::DeleteProgram(program);
    gl::DeleteShader(fs);
    gl::DeleteShader(vs);
    unsafe {
      gl::DeleteBuffers(1, &vbo);
      gl::DeleteVertexArrays(1, &vao);
    }
  });
}

struct ErrorContext;
impl glfw::ErrorCallback for ErrorContext {
    fn call(&self, _: glfw::Error, description: ~str) {
        println!("GLFW Error: {:s}", description);
    }
}
