#version 150
in vec3 position;
in vec3 normal;
out vec3 fs_position;

// out Vertex {
//   vec4 normal;
//   vec4 color;
// } vertex;


void main() {
  //vertex.normal = vec4(normal.x, normal.y, normal.z, 1.0);
  // vertex.color = vec4(1.0, position.y, 1.0, 1.0);
  fs_position = position.xyz;
  gl_Position = vec4(position.x, position.y, 0.0, 1.0);
}
