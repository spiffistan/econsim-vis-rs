#version 150

// uniform mat4 projection_matrix;
// uniform mat4 view_matrix;
// uniform mat4 model_matrix;

in vec3 position;
in vec3 normal;

out Vertex {
  vec3 vs_worldpos;
  vec3 vs_normal;
} vs_out;

void main() {
  vs_out.vs_normal = normal;
  vs_out.vs_worldpos = position;

  // gl_Position = projection_matrix * view_matrix * model_matrix * position;
  gl_Position = vec4(position.x, position.y, 0.0, 1.0);
}
