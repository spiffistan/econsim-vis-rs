#[feature(globs)];
#[feature(macro_rules)];
#[feature(link_args)];

extern crate png;
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

use cgmath::quaternion::Quat;
use cgmath::transform::Transform3D;
use cgmath::point::Point3;
use cgmath::matrix::*;
use cgmath::vector::*;
use cgmath::angle::*;
use cgmath::ptr::*;
use cgmath::angle::*;
use cgmath::projection::*;

use gl::types::*;

// Statics and globals  -- -- -- -- -- -- -- -- -- -- -- -- -- -- -- -- -- -- --

static DEBUG: bool = true;

static PNG_SRC: &'static str = "heightmap2.png";
static TEX_SRC: &'static str = "grass.png";

static CAMERA_TRANSLATE_BY: f32 = 0.05;
static CAMERA_SCALE_BY: f32 = 0.05;
static CAMERA_ROTATE_BY: f32 = 0.05;

static SCALE_MIN: f32 = 0.0;
static SCALE_MAX: f32 = 25.0;

static SUNLIGHT_INTENSITY_MIN: f32 = 0.5;
static SUNLIGHT_INTENSITY_MAX: f32 = 1.5;

// Shader sources
static VS_SRC: &'static str = "test.vert";
static FS_SRC: &'static str = "test.frag";
static GS_SRC: &'static str = "test.geom";

// Globals  -- -- -- -- -- -- -- -- -- -- -- -- -- -- -- -- -- -- -- -- -- -- --

static mut ticks: f32 = 0.0;
static mut draw_loops: bool = false;

static mut world: World = World {
  model_matrix:      Mat4 {
    x: Vec4 { x: 0.0, y: 0.0, z: 0.0, w: 0.0 },
    y: Vec4 { x: 0.0, y: 0.0, z: 0.0, w: 0.0 },
    z: Vec4 { x: 0.0, y: 0.0, z: 0.0, w: 0.0 },
    w: Vec4 { x: 0.0, y: 0.0, z: 0.0, w: 0.0 }
  },

  rotation:    Quat { s: 0.0, v: Vec3 { x: 0.0, y: 0.0, z: 0.0 } },
  scale:       1.0f32,
  translation: Vec3 { x: 0.0, y: 0.0, z: 0.0 },

  sunlight: DirectionalLight {
    color:     Vec3 { x:  0.8, y:  1.0, z:  1.0 },
    direction: Vec3 { x:  0.2, y:  0.2, z:  0.2 },
    intensity: 0.5
  }
};

static mut camera: Camera = Camera {
  view_matrix:        Mat4 {
    x: Vec4 { x: 0.0, y: 0.0, z: 0.0, w: 0.0 },
    y: Vec4 { x: 0.0, y: 0.0, z: 0.0, w: 0.0 },
    z: Vec4 { x: 0.0, y: 0.0, z: 0.0, w: 0.0 },
    w: Vec4 { x: 0.0, y: 0.0, z: 0.0, w: 0.0 }
  },

  rotation:    Quat { s: 0.0, v: Vec3 { x: 0.0, y: 0.0, z: 0.0 } },
  scale:       1.0f32,
  translation: Vec3 { x: 0.0, y: 0.0, z: 0.0 }
};

static mut screen: Screen = Screen {
  projection_matrix:  Mat4 {
    x: Vec4 { x: 3.0/4.0, y: 0.0, z: 0.0, w: 0.0 },
    y: Vec4 { x:     0.0, y: 1.0, z: 0.0, w: 0.0 },
    z: Vec4 { x:     0.0, y: 0.0, z: 0.5, w: 0.0 },
    w: Vec4 { x:     0.0, y: 0.0, z: 0.0, w: 1.0 }
  }
};

static mut vs_data: VertexShaderData = VertexShaderData {
  projection_matrix: 0,
  view_matrix: 0,
  model_matrix: 0,
  rotation: 0,
  scale: 0,
  translation: 0,
  ticks: 0
};

static mut fs_data: FragmentShaderData = FragmentShaderData {
  sunlight: 0,
  sunlight_color: 0,
  sunlight_direction: 0,
  sunlight_intensity: 0
};

// -- -- -- -- -- -- -- -- -- -- -- -- -- -- -- -- -- -- -- -- -- -- -- -- -- --

struct World {
  model_matrix:  Mat4<f32>,

  rotation:      Quat<f32>,
  scale:         f32,
  translation:   Vec3<f32>,

  sunlight: DirectionalLight
}

struct Camera {
  view_matrix:  Mat4<f32>,

  rotation:     Quat<f32>,
  scale:        f32,
  translation:  Vec3<f32>
}

struct Screen {
  projection_matrix: Mat4<f32>,
}

struct VertexShaderData {
  projection_matrix: i32,
  view_matrix: i32,
  model_matrix: i32,
  rotation: i32,
  scale: i32,
  translation: i32,
  ticks: i32
}

struct FragmentShaderData {
  sunlight: i32,
  sunlight_color: i32,
  sunlight_direction: i32,
  sunlight_intensity: i32
}

struct DirectionalLight {
  color:     Vec3<GLfloat>,
  direction: Vec3<GLfloat>,
  intensity: GLfloat
}

struct Vertex {
  position: Vec3<GLfloat>,
  normal:   Vec3<GLfloat>,
  texture:  Vec2<GLfloat>
}

impl Vertex {
  pub fn new(
    vx: f32, vy: f32, vz: f32,
    nx: f32, ny: f32, nz: f32,
    u:  f32, v:  f32) -> Vertex {
    Vertex {
      position: Vec3::new(vx, vy, vz),
      normal: Vec3::new(nx, ny, nz),
      texture: Vec2::new(u, v)
    }
  }
}

enum Compass {
  North,
  South,
  West,
  East
}

enum Zoom {
  In,
  Out
}


#[start]
fn start(argc: int, argv: **u8) -> int {
  native::start(argc, argv, main)
}

// Terrain initialization  -- -- -- -- -- -- -- -- -- -- -- -- -- -- -- -- -- --

fn load_png_image(file_path: &str) -> png::Image {
  let file = std::os::getcwd().join(Path::new(file_path));
  match png::load_png(&file) {
    Ok(image) => return image,
    Err(s) => fail!(s)
  }
}

fn load_height_data(file_path: &str, size: uint) -> ~[u8] {
  let file = std::os::getcwd().join(Path::new(file_path));
  match File::open(&file).read_bytes(size) {
    Ok(res) => return res,
    Err(s) => fail!(s)
  }
}

fn load_flat_map(height: u32, width: u32, depth: u8) -> ~[u8] {
  let mut data: ~[u8] = ~[];
  for i in range(0, width*height) {
    data.push(depth);
  }
  data
}

// Vertex, Normal and Texture initialization -- -- -- -- -- -- -- -- -- -- -- --

fn initialize_vertices(heightmap: ~[f32], width: u32, height: u32) -> ~[Vec3<GLfloat>] {
  let mut vertices: ~[Vec3<GLfloat>] = ~[];

  for x in range(0, width) {
    for y in range(0, height) {

      let xi = x as f32;
      let yi = y as f32;
      let zi = heightmap[x * width + y] as f32;

      let v = Vec3::new(xi, yi, zi);
      vertices.push(v);
    }
  }
  vertices
}

fn initialize_indices(width: u32, height: u32) -> ~[u32] {
  let mut indices: ~[u32] = ~[];

  for x in range(0, width-1) {
    for y in range(0, height-1) {

      let start = (x * width + y);
      let offset = height;

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

fn initialize_texcoords(width: u32, height: u32) -> ~[Vec2<GLfloat>] {
  let mut texcoords: ~[Vec2<GLfloat>] = ~[];

  for x in range(0, width) {
    for y in range(0, height) {

      let u: f32 = if x % 2 == 0 {0.0} else {1.0};
      let v: f32 = if y % 2 == 0 {0.0} else {1.0};

      texcoords.push(Vec2::new(u, v));
    }
  }
  texcoords
}

fn initialize_normals(v: &[Vec3<GLfloat>], width: u32, height: u32) -> ~[Vec3<GLfloat>] {
  let mut normals: ~[Vec3<GLfloat>] = ~[];

  for row in range(0, width) {
    for col in range(0, height) {


      let this = width * row + col;
      let up   = this + 1;
      let down = this - 1;
      let prev = this - width;
      let next = this + width;

      let mut sum: Vec3<f32> = Vec3::new(0f32, 0f32, 0f32);

      let cur = v[this];

      if row+1 < width && col+1 < height {
        sum = sum + (v[up] - v[this]).cross(&(v[next] - v[this])).normalize();
      }

      if row+1 < width && col > 0 && col+1 < height {
        sum = sum + (v[next] - v[this]).cross(&(v[down] - v[this])).normalize();
      }

      if row > 0 && col > 0 && col+1 < height {
        sum = sum + (v[down] - v[this]).cross(&(v[prev] - v[this])).normalize();
      }

      if row > 0 && col+1 < height && row+1 < width {
        sum = sum + (v[prev] - v[this]).cross(&(v[up] - v[this])).normalize();
      }

      sum = sum.normalize();

      // println!("{:?}", sum);

      normals.push(Vec3::new(sum.x, sum.y, sum.z));
    }
  }
  normals
}

fn initialize_vnts(vs: ~[Vec3<GLfloat>], ns: ~[Vec3<GLfloat>], ts: ~[Vec2<GLfloat>]) -> ~[Vertex] {

  // Make sure there are equal numbers of vertices, normals and texture coordinates
  assert!(vs.len() == ts.len());
  assert!(vs.len() == ns.len());

  let mut vnts: ~[Vertex] = ~[];

  for i in range(0, vs.len()) {

    let v = vs[i];
    let n = ns[i];
    let t = ts[i];

    let vnt = Vertex::new(
      v.x, v.y, v.z,
      n.x, n.y, n.z,
      t.x, t.y
    );

    vnts.push(vnt);
  }
  vnts
}

fn box_filter_heightmap(heightmap: ~[u8], width: u32, height: u32, smoothen_edges: bool) -> ~[GLfloat] {

  let mut filtered_map: ~[GLfloat] = ~[];

  let x = 0;
  let z = 0;

  let z_stop = if smoothen_edges {width}  else {width-1};
  let x_stop = if smoothen_edges {height} else {height-1};

  let bounds = width * height;

  let x_start = if smoothen_edges {0} else {1};
  let z_start = if smoothen_edges {0} else {1};

  for z in range(z_start, z_stop) {
    for x in range(x_start, x_stop) {

      // Sample a 3x3 filtering grid based on surrounding neighbors

      let mut value = 0.0f32;
      let mut average = 1.0f32;

      // Sample top row

      if (((x - 1) + (z - 1) * width) >= 0 &&
          ((x - 1) + (z - 1) * width) < bounds)
      {
        value += heightmap[(x - 1) + (z - 1) * width] as f32;
        average += 1.0;
      }

      if (((x - 0) + (z - 1) * width) >= 0 &&
          ((x - 0) + (z - 1) * width) < bounds)
      {
        value += heightmap[(x    ) + (z - 1) * width] as f32;
        average += 1.0;
      }

      if (((x + 1) + (z - 1) * width) >= 0 &&
          ((x + 1) + (z - 1) * width) < bounds)
      {
        value += heightmap[(x + 1) + (z - 1) * width] as f32;
        average += 1.0;
      }

      // Sample middle row

      if (((x - 1) + (z - 0) * width) >= 0 &&
          ((x - 1) + (z - 0) * width) < bounds)
      {
        value += heightmap[(x - 1) + (z    ) * width] as f32;
        average += 1.0;
      }

      // Sample center point (will always be in bounds)
      value += heightmap[x + z * width] as f32;

      if (((x + 1) + (z - 0) * width) >= 0 &&
          ((x + 1) + (z - 0) * width) < bounds)
      {
        value += heightmap[(x + 1) + (z    ) * width] as f32;
        average += 1.0;
      }

      // Sample bottom row

      if (((x - 1) + (z + 1) * width) >= 0 &&
          ((x - 1) + (z + 1) * width) < bounds)
      {
        value += heightmap[(x - 1) + (z + 1) * width] as f32;
        average += 1.0;
      }

      if (((x - 0) + (z + 1) * width) >= 0 &&
          ((x - 0) + (z + 1) * width) < bounds)
      {
        value += heightmap[(x    ) + (z + 1) * width] as f32;
        average += 1.0;
      }

      if (((x + 1) + (z + 1) * width) >= 0 &&
          ((x + 1) + (z + 1) * width) < bounds)
      {
        value += heightmap[(x + 1) + (z + 1) * width] as f32;
        average += 1.0;
      }

      // Store the result
      filtered_map.push(value / average)
      // filtered_map[x + z * width] = value / average;
    }
  }
  filtered_map
}

// Shader compilation and initialization  -- -- -- -- -- -- -- -- -- -- -- -- --

fn load_shader_file(file_name: &str) -> ~str {
  let p = std::os::getcwd().join(Path::new(file_name));
  match File::open(&p).read_to_end() {
    Ok(s) => str::from_utf8_owned(s).unwrap(),
    Err(s) => fail!(s)
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

////////////////////////////////////////////////////////////////////////////////

fn main() {

  if DEBUG { print!("Loading heightmap from png: {}... ", PNG_SRC); flush(); }

  let image = load_png_image(PNG_SRC.to_owned());
  let heightmap = image.pixels.clone();
  let width = image.width.clone();
  let height = image.height.clone();

  if DEBUG { println!("done. ({})", heightmap.len()) }

  // let width = 64;
  // let height = 64;
  // let heightmap = load_flat_map(width, height, 0);

  if DEBUG { print!("Computing vertices... "); flush(); }
  let filtered = box_filter_heightmap(heightmap, width, height, true);
  let vertices = initialize_vertices(filtered, width, height);
  if DEBUG { println!("done. ({} vertices)", vertices.len()) }

  if DEBUG { print!("Computing texcoords... "); flush(); }
  let texcoords = initialize_texcoords(width, height);
  if DEBUG { println!("done. ({} texcoords)", texcoords.len()) }

  if DEBUG { print!("Computing normals... "); flush(); }
  let normals = initialize_normals(vertices.clone(), width, height);
  if DEBUG { println!("done. ({} normals)", normals.len()) }

  if DEBUG { print!("Computing indices... "); flush(); }
  let indices = initialize_indices(width, height);
  if DEBUG { println!("done. ({} indices)", indices.len()) }

  if DEBUG { print!("Creating VNTs... "); flush(); }
  let vnts = initialize_vnts(vertices.clone(), normals.clone(), texcoords.clone());
  if DEBUG { println!("done. ({} VNTs, {} bytes)", vnts.len(), mem::size_of::<Vertex>() * vnts.len()) }

  unsafe {
    initialize_world();
    initialize_camera();
    initialize_screen();
  }

  // Start OpenGL -- -- -- -- -- -- -- -- -- -- -- -- -- -- -- -- -- -- -- -- --

  let vs_src = load_shader_file(VS_SRC);
  let fs_src = load_shader_file(FS_SRC);
  let gs_src = load_shader_file(GS_SRC);

  glfw::set_error_callback(~ErrorContext);

  glfw::start(proc() {

    // Choose a GL profile that is compatible with OS X 10.7+
    glfw::window_hint::context_version(3, 2);
    glfw::window_hint::opengl_profile(glfw::OpenGlCoreProfile);
    glfw::window_hint::opengl_forward_compat(true);

    let window = glfw::Window::create(1920, 1280, "OpenGL", glfw::Windowed).unwrap();
    window.set_key_polling(true);
    window.make_context_current();

    // Load the OpenGL function pointers
    gl::load_with(glfw::get_proc_address);

    // Create GLSL shaders
    let vertex_shader   = compile_shader(vs_src, gl::VERTEX_SHADER);
    let fragment_shader = compile_shader(fs_src, gl::FRAGMENT_SHADER);
    // let geometry_shader = compile_shader(gs_src, gl::GEOMETRY_SHADER);
    let shader_program = link_program(vertex_shader, fragment_shader); // , geometry_shader);

    let mut vertex_array_id = 0;
    let mut vnt_buffer_id = 1;
    let mut index_buffer_id = 2;

    let mut grass_texture_id = 1;

    unsafe {

      // Create Vertex Array Object and Vertex Buffer Objects
      gl::GenVertexArrays(1, &mut vertex_array_id);
      gl::BindVertexArray(vertex_array_id);

      initialize_vbo(vnts,  &mut vnt_buffer_id, gl::ARRAY_BUFFER);

      // // Initialize vertex indices /////////////////////////////////////////////
      let indices_bytes = (indices.len() * mem::size_of::<u32>()) as GLsizeiptr;
      let indices_ptr = cast::transmute(&indices[0]);

      gl::GenBuffers(1, &mut index_buffer_id);
      gl::BindBuffer(gl::ELEMENT_ARRAY_BUFFER, index_buffer_id);
      gl::BufferData(gl::ELEMENT_ARRAY_BUFFER, indices_bytes, indices_ptr, gl::STATIC_DRAW);

      gl::GenTextures(1, &mut grass_texture_id);
      gl::BindTexture(gl::TEXTURE_2D, grass_texture_id);

      let tex = load_png_image(TEX_SRC);
      let tex_height = tex.height.clone();
      let tex_width = tex.width.clone();
      let data = tex.pixels;

      gl::TexImage2D(gl::TEXTURE_2D, 0, gl::RGBA as GLint, tex_width as GLint, tex_height as GLint, 0, gl::RGBA, gl::UNSIGNED_BYTE, data.as_ptr() as GLeglImageOES);

      gl::TexParameteri(gl::TEXTURE_2D, gl::TEXTURE_MAG_FILTER, gl::LINEAR as i32);
      gl::TexParameteri(gl::TEXTURE_2D, gl::TEXTURE_MIN_FILTER, gl::LINEAR as i32);
      gl::GenerateMipmap(gl::TEXTURE_2D);

      // Use shader program
      gl::UseProgram(shader_program);

      initialize_shader_data(shader_program);

      gl::EnableVertexAttribArray(0);

      let stride = mem::size_of::<Vec2<GLfloat>>() + mem::size_of::<Vec3<GLfloat>>()+ mem::size_of::<Vec3<GLfloat>>();

      gl::VertexAttribPointer(0, 3, gl::FLOAT, gl::FALSE, stride as GLint, ptr::null());

      let normals_offset  = cast::transmute(mem::size_of::<Vec3<GLfloat>>());
      gl::VertexAttribPointer(1, 2, gl::FLOAT, gl::FALSE, stride as GLint, normals_offset);

      let texcoords_offset  = cast::transmute(mem::size_of::<Vec3<GLfloat>>() + mem::size_of::<Vec2<GLfloat>>());
      gl::VertexAttribPointer(2, 3, gl::FLOAT, gl::FALSE, stride as GLint, texcoords_offset);

      gl::Enable(gl::DEPTH_TEST);
      gl::BlendFunc(gl::SRC_ALPHA, gl::ONE_MINUS_SRC_ALPHA);
      gl::Enable(gl::CULL_FACE);
      gl::CullFace(gl::BACK);
      gl::FrontFace(gl::CW);
    }

    let mut last_time = glfw::get_time();
    let mut current_time: f64 = 0.0;
    let mut frames: u64 = 0;

    while !window.should_close() {

      // Compute FPS
      current_time = glfw::get_time();
      frames += 1;

      if current_time - last_time >= 1.0 {
        println!("{} FPS ({} ms/frame)", frames, 1000.0/(frames as f64))
        frames = 0;
        last_time += 1.0;
      }

      // Poll events
      glfw::poll_events();
      for event in window.flush_events() {
        handle_window_event(&window, event);
        unsafe { update_uniforms() }
      }

      // Clear the screen to black
      gl::ClearColor(34.0/256.0, 37.0/256.0, 39.0/256.0, 1.0);
      gl::Clear(gl::COLOR_BUFFER_BIT | gl::DEPTH_BUFFER_BIT);

      unsafe {
        let kind = if draw_loops {gl::LINE_LOOP} else {gl::TRIANGLES};
        gl::DrawElements(kind, indices.len() as GLint, gl::UNSIGNED_INT, ptr::null());
        ticks += 1.0;
        gl::Uniform1f(vs_data.ticks, ticks);
      }

      // Swap buffers
      window.swap_buffers();
    }

    // Cleanup
    gl::DeleteProgram(shader_program);
    gl::DeleteShader(fragment_shader);
    gl::DeleteShader(vertex_shader);

    unsafe {
      gl::DeleteBuffers(1, &index_buffer_id);
      gl::DeleteBuffers(1, &vnt_buffer_id);
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

// OpenGL initializers  -- -- -- -- -- -- -- -- -- -- -- -- -- -- -- -- -- -- --

unsafe fn initialize_vbo<T>(vec: ~[T], buf_id: &mut GLuint, array_type: GLenum) {
  let vec_bytes = (vec.len() * mem::size_of::<T>()) as GLsizeiptr;
  let vec_ptr = cast::transmute(&vec[0]);

  gl::GenBuffers(1, buf_id);
  gl::BindBuffer(array_type, *buf_id);
  gl::BufferData(array_type, vec_bytes, vec_ptr, gl::STATIC_DRAW);
}

unsafe fn initialize_world() {

  let xr = deg(15.0f32).to_rad().s;
  let yr = deg(0.0f32).to_rad().s;;
  let zr = deg(-15.0f32).to_rad().s;

  world.rotation = Quat::new(1.0f32, xr, yr, zr); // Rotate the world
  world.translation = Vec3::new(-1.0f32, -1.0f32, 0.0f32); // Center the world
  world.scale = 0.001f32; // Zoom far out

  world.model_matrix = Transform3D::new(world.scale, world.rotation, world.translation).to_mat4();
}

unsafe fn initialize_camera() {

  camera.rotation = Quat::identity(); // No rotation
  camera.translation = Vec3::zero(); // No translation
  camera.scale = 1f32; // No zooming

  // camera.view_matrix = Transform3D::new(camera.scale, camera.rotation, camera.translation).to_mat4();
  camera.view_matrix = Mat4::identity();
}

unsafe fn initialize_screen() {
  screen.projection_matrix = Mat4::identity();
}

unsafe fn update_uniforms() {
  gl::UniformMatrix4fv(vs_data.projection_matrix, 1, gl::FALSE, screen.projection_matrix.cr(0,0));
  gl::UniformMatrix4fv(vs_data.view_matrix, 1, gl::FALSE, camera.view_matrix.cr(0,0));
  gl::UniformMatrix4fv(vs_data.model_matrix, 1, gl::FALSE, world.model_matrix.cr(0,0));

  gl::Uniform3f(fs_data.sunlight_color, world.sunlight.color.x, world.sunlight.color.y, world.sunlight.color.z);
  gl::Uniform3f(fs_data.sunlight_direction, world.sunlight.direction.x, world.sunlight.direction.y, world.sunlight.direction.z);
  gl::Uniform1f(fs_data.sunlight_intensity, world.sunlight.intensity);
}

unsafe fn initialize_shader_data(shader_program: GLuint) {
  vs_data.model_matrix       = "M".with_c_str(|ptr| gl::GetUniformLocation(shader_program, ptr));
  vs_data.view_matrix        = "V".with_c_str(|ptr| gl::GetUniformLocation(shader_program, ptr));
  vs_data.projection_matrix  = "P".with_c_str(|ptr| gl::GetUniformLocation(shader_program, ptr));
  vs_data.ticks  = "timer".with_c_str(|ptr| gl::GetUniformLocation(shader_program, ptr));

  vs_data.scale              = "scaling".with_c_str(|ptr| gl::GetUniformLocation(shader_program, ptr));
  vs_data.translation        = "translation".with_c_str(|ptr| gl::GetUniformLocation(shader_program, ptr));
  fs_data.sunlight_color     = "sunlight.color".with_c_str(|ptr| gl::GetUniformLocation(shader_program, ptr));
  fs_data.sunlight_direction = "sunlight.direction".with_c_str(|ptr| gl::GetUniformLocation(shader_program, ptr));
  fs_data.sunlight_intensity = "sunlight.intensity".with_c_str(|ptr| gl::GetUniformLocation(shader_program, ptr));

  "position".with_c_str(|ptr| gl::GetAttribLocation(shader_program, ptr));
  "texcoord".with_c_str(|ptr| gl::GetAttribLocation(shader_program, ptr));
  "normal".with_c_str(|ptr| gl::GetAttribLocation(shader_program, ptr));

  "out_color".with_c_str(|ptr| gl::BindFragDataLocation(shader_program, 0, ptr));
}

// Event handling -- -- -- -- -- -- -- -- -- -- -- -- -- -- -- -- -- -- -- -- --

unsafe fn update_model_matrix() {
  world.model_matrix = Transform3D::new(world.scale, world.rotation, world.translation).to_mat4();
}

unsafe fn update_view_matrix() {
  camera.view_matrix = Transform3D::new(camera.scale, camera.rotation, camera.translation).to_mat4();
}

unsafe fn update_projection_matrix() {
  // screen.projection_matrix = Mat4::identity();
}

// Rotation  - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - -

unsafe fn rotate_camera(x: f32, y: f32, z: f32) {

  camera.rotation.v.x += x;
  camera.rotation.v.y += y;
  camera.rotation.v.z += z;

  update_view_matrix();
}

unsafe fn rotate_x(cw: bool) {
  let sign = if cw {1.0} else {-1.0};
  rotate_camera(CAMERA_ROTATE_BY * sign, 0.0, 0.0);
}

unsafe fn rotate_y(cw: bool) {
  let sign = if cw {1.0} else {-1.0};
  rotate_camera(0.0, CAMERA_ROTATE_BY * sign, 0.0);
}

unsafe fn rotate_z(cw: bool) {
  let sign = if cw {1.0} else {-1.0};
  rotate_camera(0.0, 0.0, CAMERA_ROTATE_BY * sign);
}

// Translation - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - -

unsafe fn translate_camera(x: f32, y: f32, z: f32) {
  camera.translation.x += x;
  camera.translation.y += y;
  camera.translation.z += z;

  update_view_matrix();
}

unsafe fn move(dir: Compass) {
  match dir {
    North => translate_camera(0f32, -CAMERA_TRANSLATE_BY, 0f32),
    South => translate_camera(0f32,  CAMERA_TRANSLATE_BY, 0f32),
    West  => translate_camera( CAMERA_TRANSLATE_BY, 0f32, 0f32),
    East  => translate_camera(-CAMERA_TRANSLATE_BY, 0f32, 0f32)
  }
}

// Scaling - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - -

unsafe fn scale_camera(factor: f32) {
  if camera.scale + factor > SCALE_MIN && camera.scale + factor < SCALE_MAX {
    camera.scale += factor;
  }

  update_view_matrix();
}

unsafe fn zoom(dir: Zoom) {
  match dir {
    In  => scale_camera( CAMERA_SCALE_BY),
    Out => scale_camera(-CAMERA_SCALE_BY)
  }
}

unsafe fn adjust_light_intensity(factor: f32) {
  if world.sunlight.intensity + factor > SUNLIGHT_INTENSITY_MIN
  && world.sunlight.intensity + factor < SUNLIGHT_INTENSITY_MAX {
    world.sunlight.intensity += factor
  }
}

fn handle_window_event(window: &glfw::Window, (time, event): (f64, glfw::WindowEvent)) {
  unsafe {
    match event {
      glfw::PosEvent(x, y)                => window.set_title(format!("Time: {}, Window pos: ({}, {})", time, x, y)),
      glfw::SizeEvent(w, h)               => window.set_title(format!("Time: {}, Window size: ({}, {})", time, w, h)),
      glfw::CloseEvent                    => println!("Time: {}, Window close requested.", time),
      glfw::RefreshEvent                  => println!("Time: {}, Window refresh callback triggered.", time),
      glfw::FocusEvent(true)              => println!("Time: {}, Window focus gained.", time),
      glfw::FocusEvent(false)             => println!("Time: {}, Window focus lost.", time),
      glfw::IconifyEvent(true)            => println!("Time: {}, Window was minimised", time),
      glfw::IconifyEvent(false)           => println!("Time: {}, Window was maximised.", time),
      glfw::FramebufferSizeEvent(w, h)    => println!("Time: {}, Framebuffer size: ({}, {})", time, w, h),
      glfw::CharEvent(character)          => println!("Time: {}, Character: {}", time, character),
      glfw::MouseButtonEvent(btn, action, mods) => println!("Time: {}, Button: {}, Action: {}, Modifiers: [{}]", time, btn, action, mods),
      glfw::CursorPosEvent(xpos, ypos)    => window.set_title(format!("Time: {}, Cursor position: ({}, {})", time, xpos, ypos)),
      glfw::CursorEnterEvent(true)        => println!("Time: {}, Cursor entered window.", time),
      glfw::CursorEnterEvent(false)       => println!("Time: {}, Cursor left window.", time),
      glfw::ScrollEvent(x, y)             => window.set_title(format!("Time: {}, Scroll offset: ({}, {})", time, x, y)),
      glfw::KeyEvent(key, scancode, action, mods) => {
        println!("Time: {}, Key: {}, ScanCode: {}, Action: {}, Modifiers: [{}]", time, key, scancode, action, mods);
        handle_key_event(window, key, action, mods);
      }
    }
  }
}

unsafe fn handle_key_event(window: &glfw::Window, key: glfw::Key, action: glfw::Action, mods: glfw::Modifiers) {

  match (key, action) {
    (glfw::KeyEscape, glfw::Press) => window.set_should_close(true),
    (glfw::KeyW, _) => { move(North) },
    (glfw::KeyS, _) => { move(South) },
    (glfw::KeyA, _) => { move(West)  },
    (glfw::KeyD, _) => { move(East)  },

    // Rotate X or Z
    (glfw::KeyUp, _)  => {
      if mods.contains(glfw::Shift) { rotate_z(true) } else { rotate_x(true) }
    }

    (glfw::KeyDown, _)  => {
      if mods.contains(glfw::Shift) { rotate_z(false) } else { rotate_x(false) }
    }

    // Rotate Y
    (glfw::KeyLeft, _)   => { rotate_y(true) },
    (glfw::KeyRight, _)  => { rotate_y(false) },

    (glfw::KeyR, glfw::Press)      => { zoom(In) },
    (glfw::KeyR, glfw::Repeat)     => { zoom(In) },
    (glfw::KeyF, glfw::Press)      => { zoom(Out) },
    (glfw::KeyF, glfw::Repeat)     => { zoom(Out) },

    (glfw::KeyK, glfw::Press)      => { adjust_light_intensity(-0.02) },
    (glfw::KeyL, glfw::Press)      => { adjust_light_intensity(0.02) },
    (glfw::KeyK, glfw::Repeat)     => { adjust_light_intensity(-0.02) },
    (glfw::KeyL, glfw::Repeat)     => { adjust_light_intensity(0.02) },

    (glfw::KeyT, glfw::Press)      => { draw_loops = !draw_loops },

    (glfw::KeySpace, glfw::Press) => {
      // Resize should cause the window to "refresh"
      let (window_width, window_height) = window.get_size();
      window.set_size(window_width + 1, window_height);
      window.set_size(window_width, window_height);
    }
    _ => {}
  }
}
