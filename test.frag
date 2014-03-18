#version 150
out vec4 out_color;
in vec3 fs_position;

void main() {
  out_color = vec4(0.0, 0.0, (fs_position.z / 256.0), 1.0);
}
