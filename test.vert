#version 150
in vec3 position;
in vec3 normal;

out Vertex {
  vec4 normal;
  vec4 color;
} vertex;


void main() {
  vertex.normal = vec4(normal.x, normal.y, normal.z, 1.0);
  vertex.color = vec4(1.0, 1.0, normal.z / 256.0, 1.0);
  gl_Position = vec4(position.x, position.y, position.z, 1.0);
}
