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

use cgmath::matrix::*;
use cgmath::vector::*;
use cgmath::angle::*;
use cgmath::ptr::*;
use cgmath::angle::*;
use cgmath::projection::*;

use gl::types::*;

// Statics and globals  -- -- -- -- -- -- -- -- -- -- -- -- -- -- -- -- -- -- --

static DEBUG: bool = true;

static PNG_SRC: &'static str = "map.png";
static TEX_SRC: &'static str = "grass.png";
// static MAP_SRC: &'static str = "elevation.data";
// static MAP_W: uint = 2048;
// static MAP_H: uint = 2048;
// static MAP_SIZE: uint = MAP_W * MAP_H;

static SCALE:   f32 = 64.0;
static SCALE_X: f32 = SCALE;
static SCALE_Y: f32 = SCALE;
static SCALE_Z: f32 = 200.0;

static SCROLLSPEED: f32 = 0.1;
static SCALE_MAX: f32 = 25.0;
static SUNLIGHT_INTENSITY_MIN: f32 = 0.5;
static SUNLIGHT_INTENSITY_MAX: f32 = 1.5;

// Shader sources
static VS_SRC: &'static str = "test.vert";
static FS_SRC: &'static str = "test.frag";

// Globals  -- -- -- -- -- -- -- -- -- -- -- -- -- -- -- -- -- -- -- -- -- -- --

static mut vs_rotation_x: f32 = 0.0;
static mut vs_scale_all: f32 = 5.0;
static mut vs_translate: Vec3<GLfloat> = Vec3 { x: -0.75, y: -0.5, z: 0.0 };

static mut fs_sunlight: DirectionalLight = DirectionalLight {
  color:     Vec3 { x: 1.0, y: 1.0, z: 1.0 },
  direction: Vec3 { x: 0.2, y: 0.5, z: -1.0 },
  intensity: 1.0
};

// Shader uniform pointers
static mut in_rotation_x_p: i32 = 0;
static mut in_scale_p: i32 = 0;
static mut in_translate_p: i32 = 0;
static mut in_sunlight_p: i32 = 0;
static mut in_sunlight_color_p: i32 = 0;
static mut in_sunlight_direction_p: i32 = 0;
static mut in_sunlight_intensity_p: i32 = 0;

// -- -- -- -- -- -- -- -- -- -- -- -- -- -- -- -- -- -- -- -- -- -- -- -- -- --

// TODO: perhaps use this?
//
// Vertex-Normal-Texture
// pub struct Vnt {
//   position: Vec4<GLfloat>,
//   normal:   Vec4<GLfloat>
//   texture:  Vec2<GLfloat>
// }
//
// impl Vnt {
//   pub fn new(
//     x:  f32, y:  f32, z:  f32, w:  f32,
//     nx: f32, ny: f32, nz: f32, nw: f32,
//     u: f32, v: f32) -> Vnt {
//     Vnt {
//       position: Vec4::new(x, y, z, w),
//       normal: Vec4::new(nx, ny, nz, nw),
//       texture: Vec2::new(u, v)
//     }
//   }
// }


struct DirectionalLight {
  color: Vec3<GLfloat>,
  direction: Vec3<GLfloat>,
  intensity: GLfloat
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

fn initialize_vertices(heightmap: ~[u8], width: u32, height: u32) -> ~[Vec4<GLfloat>] {
  let mut vertices: ~[Vec4<GLfloat>] = ~[];

  for x in range(0, width) {
    for y in range(0, height) {

      let xi = x as f32 / SCALE_X;
      let yi = y as f32 / SCALE_Y;
      let zi = heightmap[x * width + y] as f32 / SCALE_Z;
      let wi = 0.0;

      let v = Vec4::new(xi, yi, zi, wi);
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

      let u: f32 = if x % 2 == 0 { 0.0 } else { 1.0 };
      let v: f32 = if y % 2 == 0 { 0.0 } else { 1.0 };

      texcoords.push(Vec2::new(u, v));
    }
  }
  texcoords
}

fn initialize_normals(v: ~[Vec4<GLfloat>], width: u32, height: u32) -> ~[Vec3<GLfloat>] {
  let mut normals: ~[Vec3<GLfloat>] = ~[];

  for row in range(0, width-1) {
    for col in range(0, height-1) {

      // let hr = width * row;
      // let hc = col;
      //
      // let mut sum = Vec3::new(0f32, 0f32, 0f32);
      // let cur = v[hr+hc].truncate();
      //
      // if row+1 < width && col+1 < height {
      //   sum = sum + (v[hr+0 + hc+1].truncate() - cur).cross(&(v[hr+1 + hc+0].truncate() - cur)).normalize();
      // }
      //
      // if row+1 < width && col > 0 {
      //   sum = sum + (v[hr+1 + hc+0].truncate() - cur).cross(&(v[hr+0 + hc+1].truncate() - cur)).normalize();
      // }
      //
      // if row > 0 && col > 0 {
      //   sum = sum + (v[hr+0 + hc+1].truncate() - cur).cross(&(v[hr+1 + hc+0].truncate() - cur)).normalize();
      // }
      //
      // if row > 0 && col+1 < height {
      //   sum = sum + (v[hr+1 + hc+0].truncate() - cur).cross(&(v[hr+0 + hc+1].truncate() - cur)).normalize();
      // }
      //
      // sum = sum.normalize();
      //
      // normals.push(Vec3::new(sum.x, sum.y, sum.z));
      normals.push(Vec3::new(0f32, 1f32, 0f32))
    }
  }
  normals
}


// Shader compilation and initialization  -- -- -- -- -- -- -- -- -- -- -- -- --

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

////////////////////////////////////////////////////////////////////////////////

fn main() {

  // if DEBUG { print!("Loading heightmap from png: {}... ", PNG_SRC); flush(); }
  //
  // let image = load_png_image(PNG_SRC.to_owned());
  // let heightmap = image.pixels.clone();
  // let width = image.width.clone();
  // let height = image.height.clone();
  //
  // if DEBUG { println!("done. ({})", heightmap.len()) }

  let width = 64;
  let height = 64;
  let heightmap = load_flat_map(width, height, 0);

  if DEBUG { print!("Computing vertices... "); flush(); }
  let vertices = initialize_vertices(heightmap, width, height);
  if DEBUG { println!("done. ({} vertices)", vertices.len()) }

  if DEBUG { print!("Computing texcoords... "); flush(); }
  let texcoords = initialize_texcoords(width, height);
  if DEBUG { println!("done. ({} texcoords)", texcoords.len()) }

  if DEBUG { print!("Computing indices... "); flush(); }
  let indices = initialize_indices(width, height);
  if DEBUG { println!("done. ({} indices)", indices.len()) }

  if DEBUG { print!("Computing normals... "); flush(); }
  let normals  = initialize_normals(vertices.clone(), width, height);
  if DEBUG { println!("done. ({} normals)", normals.len()) }

  // let mut field_of_view:      f32 = 60.0;
  // let mut aspect_ratio:       f32 = width as f32 / height as f32;
  // let mut near_plane:         f32 = 0.1;
  // let mut far_plane:          f32 = 5.0;
  // let mut frustum_length:     f32 = far_plane - near_plane;
  // let mut y_scale:            f32 = cot(deg(field_of_view / 2.0).to_rad());
  // let mut x_scale:            f32 = y_scale / aspect_ratio;
  //
  // let mut matrix_44_buf:      ~[GLfloat] = ~[0f32, ..16]; // XXX Needed?
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

  // Start OpenGL -- -- -- -- -- -- -- -- -- -- -- -- -- -- -- -- -- -- -- -- --

  let vs_src = load_shader_file(VS_SRC);
  let fs_src = load_shader_file(FS_SRC);

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
    let vertex_shader = compile_shader(vs_src, gl::VERTEX_SHADER);
    let fragment_shader = compile_shader(fs_src, gl::FRAGMENT_SHADER);
    let shader_program = link_program(vertex_shader, fragment_shader);

    let mut vertex_array_id = 0;
    let mut vertex_buffer_id = 1;
    let mut index_buffer_id = 2;
    let mut texcoords_buffer_id = 3;
    let mut grass_texture_id = 4;
    let mut normal_buffer_id = 5;

    unsafe {

      // Create Vertex Array Object and Vertex Buffer Objects
      gl::GenVertexArrays(1, &mut vertex_array_id);
      gl::BindVertexArray(vertex_array_id);

      initialize_vbo(vertices, &mut vertex_buffer_id, gl::ARRAY_BUFFER);
      initialize_vbo(texcoords, &mut texcoords_buffer_id, gl::ARRAY_BUFFER);
      initialize_vbo(normals, &mut normal_buffer_id, gl::ARRAY_BUFFER);
      // initialize_vbo(indices, &mut index_buffer_id, gl::ELEMENT_ARRAY_BUFFER);

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
      "out_color".with_c_str(|ptr| gl::BindFragDataLocation(shader_program, 0, ptr));

      in_rotation_x_p = "in_rotate_x".with_c_str(|ptr| gl::GetUniformLocation(shader_program, ptr));
      gl::Uniform1f(in_rotation_x_p, vs_rotation_x);

      in_scale_p = "in_scale_all".with_c_str(|ptr| gl::GetUniformLocation(shader_program, ptr));
      gl::Uniform1f(in_scale_p, vs_scale_all);

      in_translate_p = "in_translate".with_c_str(|ptr| gl::GetUniformLocation(shader_program, ptr));
      gl::Uniform3f(in_translate_p, vs_translate.x, vs_translate.y, vs_translate.z);

      initialize_sunlight(shader_program);

      let position_p = "position".with_c_str(|ptr| gl::GetAttribLocation(shader_program, ptr));
      let texcoord_p = "texcoord".with_c_str(|ptr| gl::GetAttribLocation(shader_program, ptr));
      let normal_p =   "normal".with_c_str(|ptr| gl::GetAttribLocation(shader_program, ptr));

      gl::EnableVertexAttribArray(0);
      gl::EnableVertexAttribArray(1);
      gl::EnableVertexAttribArray(2);
      gl::VertexAttribPointer(0, 4, gl::FLOAT, gl::FALSE, 0, ptr::null());
      gl::VertexAttribPointer(1, 2, gl::FLOAT, gl::FALSE, 0, ptr::null());
      gl::VertexAttribPointer(2, 3, gl::FLOAT, gl::FALSE, 1, ptr::null());

      gl::Enable(gl::DEPTH_TEST);
      gl::Enable(gl::CULL_FACE);
      gl::CullFace(gl::BACK);
      gl::FrontFace(gl::CW);
    }

    while !window.should_close() {
      // Poll events
      glfw::poll_events();
      for event in window.flush_events() {
        handle_window_event(&window, event);
        unsafe {
          // Update the globals
          gl::Uniform1f(in_rotation_x_p, vs_rotation_x);
          gl::Uniform1f(in_scale_p, vs_scale_all);
          gl::Uniform3f(in_translate_p, vs_translate.x, vs_translate.y, vs_translate.z);
          gl::Uniform3f(in_sunlight_color_p, fs_sunlight.color.x, fs_sunlight.color.x, fs_sunlight.color.x);
          gl::Uniform3f(in_sunlight_direction_p, fs_sunlight.direction.x, fs_sunlight.direction.x, fs_sunlight.direction.x);
          gl::Uniform1f(in_sunlight_intensity_p, fs_sunlight.intensity);
        }
      }

      // Clear the screen to black
      gl::ClearColor(0.3, 0.3, 0.3, 1.0);
      gl::Clear(gl::COLOR_BUFFER_BIT | gl::DEPTH_BUFFER_BIT);

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
      // gl::DeleteBuffers(1, &normal_buffer_id);
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

unsafe fn initialize_vbo<T>(vec: ~[T], buf_id: &mut GLuint, array_type: u32) {
  let vec_bytes = (vec.len() * mem::size_of::<T>()) as GLsizeiptr;
  let vec_ptr = cast::transmute(&vec[0]);

  gl::GenBuffers(1, buf_id);
  gl::BindBuffer(array_type, *buf_id);
  gl::BufferData(array_type, vec_bytes, vec_ptr, gl::STATIC_DRAW);
}

unsafe fn initialize_sunlight(shader_program: GLuint) {
  in_sunlight_color_p     = "sunlight.color".with_c_str(|ptr| gl::GetUniformLocation(shader_program, ptr));
  in_sunlight_direction_p = "sunlight.direction".with_c_str(|ptr| gl::GetUniformLocation(shader_program, ptr));
  in_sunlight_intensity_p = "sunlight.intensity".with_c_str(|ptr| gl::GetUniformLocation(shader_program, ptr));

  gl::Uniform3f(in_sunlight_color_p, fs_sunlight.color.x, fs_sunlight.color.x, fs_sunlight.color.x);
  gl::Uniform3f(in_sunlight_direction_p, fs_sunlight.direction.x, fs_sunlight.direction.x, fs_sunlight.direction.x);
  gl::Uniform1f(in_sunlight_intensity_p, fs_sunlight.intensity);
}

// Event handling -- -- -- -- -- -- -- -- -- -- -- -- -- -- -- -- -- -- -- -- --

unsafe fn move_world(x_factor: f32, y_factor: f32) {
  vs_translate.x += x_factor;
  vs_translate.y += y_factor;
}

unsafe fn scale_world(factor: f32) {
  if vs_scale_all + factor > 0.0 && vs_scale_all + factor < SCALE_MAX {
    vs_scale_all += factor
  }
}

unsafe fn adjust_light_intensity(factor: f32) {
  if fs_sunlight.intensity + factor > SUNLIGHT_INTENSITY_MIN
  && fs_sunlight.intensity + factor < SUNLIGHT_INTENSITY_MAX {
    fs_sunlight.intensity += factor
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
        match (key, action) {
          (glfw::KeyEscape, glfw::Press) => window.set_should_close(true),
          (glfw::KeyW, glfw::Repeat)     => { move_world( 0.00, -SCROLLSPEED) },
          (glfw::KeyW, glfw::Press)      => { move_world( 0.00, -SCROLLSPEED) },
          (glfw::KeyS, glfw::Repeat)     => { move_world( 0.00,  SCROLLSPEED) },
          (glfw::KeyS, glfw::Press)      => { move_world( 0.00,  SCROLLSPEED) },
          (glfw::KeyA, glfw::Repeat)     => { move_world( SCROLLSPEED,  0.00) },
          (glfw::KeyA, glfw::Press)      => { move_world( SCROLLSPEED,  0.00) },
          (glfw::KeyD, glfw::Repeat)     => { move_world(-SCROLLSPEED,  0.00) },
          (glfw::KeyD, glfw::Press)      => { move_world(-SCROLLSPEED,  0.00) },

          (glfw::KeyR, glfw::Press)      => { scale_world(0.5) },
          (glfw::KeyR, glfw::Repeat)     => { scale_world(0.5) },
          (glfw::KeyF, glfw::Press)      => { scale_world(-0.5) },
          (glfw::KeyF, glfw::Repeat)     => { scale_world(-0.5) },

          (glfw::KeyK, glfw::Press)      => { adjust_light_intensity(-0.02) },
          (glfw::KeyL, glfw::Press)      => { adjust_light_intensity(0.02) },
          (glfw::KeyK, glfw::Repeat)     => { adjust_light_intensity(-0.02) },
          (glfw::KeyL, glfw::Repeat)     => { adjust_light_intensity(0.02) },

          (glfw::KeyDown, glfw::Repeat)  => { vs_rotation_x -= 5.0; },
          (glfw::KeyUp, glfw::Repeat)    => { vs_rotation_x += 5.0; },

          (glfw::KeySpace, glfw::Press) => {
            // Resize should cause the window to "refresh"
            let (window_width, window_height) = window.get_size();
            window.set_size(window_width + 1, window_height);
            window.set_size(window_width, window_height);
          }
          _ => {}
        }
      }
    }
  }
}
