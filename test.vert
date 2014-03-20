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

  // const mat3 projection = mat3(
  //     vec3(3.0/4.0, 0.0, 0.0),
  //     vec3(    0.0, 1.0, 0.0),
  //     vec3(    0.0, 0.0, 0.5)
  // );

  mat4 isometric = mat4(
    vec4(sqrt(3),  0,       -sqrt(3), 0),
    vec4(1.0,      2.0,          1.0, 0),
    vec4(sqrt(2),  -sqrt(2), sqrt(2), 0),
    vec4(0, 0, 0, 1)
  );

  mat4 scale = mat4(
    vec4(2.5, 0, 0, 0),
    vec4(0, 2.5, 0, 0),
    vec4(0, 0, 2.5, 0),
    vec4(0, 0, 0,  1)
  );

  mat4 translate = mat4(
    vec4(1, 0, 0, 0),
    vec4(0, 1, 0, 0),
    vec4(0, 0, 1, 0),
    vec4(-0.49, -0.49, 0.0, 1)
  );

  vec4 projected_position = isometric * scale * translate * vec4(position.xy, position.z * -1.0, 1.0) * (1.0/sqrt(6));
  float perspective_factor = 1; // projected_position.z * 0.5 + 1.0;

  gl_Position = vec4(projected_position.xyz/perspective_factor, 1.0);
}
