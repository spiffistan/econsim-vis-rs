#version 150
in vec3 position;
in vec3 normal;
// vec3 tangent;
// vec3 bitangent;
// vec4 prev_x;
// vec4 prev_z;
// vec4 next_x;
// vec4 next_z;
//
// //Returns a normal from a grid of heights
//
// vec3 compute_normals(void) {
//
//   float step = 1/2048; //where 1024 is texture width, i.e. heightmap width
//   // calculate the neigbour positions
//   vec4 prev_x = vec4(position.x, 0.0, position.y, 1.0);
//   vec4 prev_z = vec4(position.x, 0.0, position.y, 1.0);
//   vec4 next_x = vec4(position.x, 0.0, position.y, 1.0);
//   vec4 next_z = vec4(position.x, 0.0, position.y, 1.0);
//   prev_x.x -= step;    prev_z.z -= step;
//   next_x.x += step;    next_z.z += step;
//
//   vec4 tangent = next_x - prev_x;
//   vec4 bitangent = next_z - prev_z;
//   vec3 normal = normalize(cross(tangent.xyz, bitangent.xyz));
//
//   return normal;
// }

void main() {
  gl_Position = vec4(position.x, position.y, position.z, 1.0);
}
