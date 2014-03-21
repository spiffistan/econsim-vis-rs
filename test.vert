#version 150

// uniform mat4 projection_matrix;
// uniform mat4 view_matrix;
// uniform mat4 model_matrix;

in vec3 position;
in vec3 normal;

uniform float in_rotate_x;
uniform float in_scale_all;
uniform vec3 in_translate;


out Vertex {
  vec3 vs_worldpos;
  vec3 vs_normal;
} vs_out;

mat4 view_frustum(
    float angle_of_view,
    float aspect_ratio,
    float z_near,
    float z_far
) {
    return mat4(
        vec4(1.0/tan(angle_of_view),           0.0, 0.0, 0.0),
        vec4(0.0, aspect_ratio/tan(angle_of_view),  0.0, 0.0),
        vec4(0.0, 0.0,    (z_far+z_near)/(z_far-z_near), 1.0),
        vec4(0.0, 0.0, -2.0*z_far*z_near/(z_far-z_near), 0.0)
    );
}

mat4 scale(float x, float y, float z) {

    return mat4(
        vec4(x,   0.0, 0.0, 0.0),
        vec4(0.0, y,   0.0, 0.0),
        vec4(0.0, 0.0, z,   0.0),
        vec4(0.0, 0.0, 0.0, 1.0)
    );
}

mat4 translate(float x, float y, float z) {
    return mat4(
        vec4(1.0, 0.0, 0.0, 0.0),
        vec4(0.0, 1.0, 0.0, 0.0),
        vec4(0.0, 0.0, 1.0, 0.0),
        vec4(x,   y,   z,   1.0)
    );
}

mat4 rotate_x(float theta) {
    return mat4(
        vec4(1.0,         0.0,         0.0, 0.0),
        vec4(0.0,  cos(theta),  sin(theta), 0.0),
        vec4(0.0, -sin(theta),  cos(theta), 0.0),
        vec4(0.0,         0.0,         0.0, 1.0)
    );
}

void main() {

  vs_out.vs_normal = normal;
  vs_out.vs_worldpos = position;

  mat4 isometric = mat4(
    vec4(sqrt(3),  0,       -sqrt(3),  0),
    vec4(1.0,      2.0,      1.0,      0),
    vec4(sqrt(2),  -sqrt(2), sqrt(2),  0),
    vec4(0,        0,        0,        1)
  );

  mat4 projection = isometric * (1.0/sqrt(6));

  vec4 projected_position = view_frustum(radians(45.0), 4.0/3.0, 0.0, 5.0 * in_scale_all)
    * translate(in_translate.x, in_translate.y, in_translate.z)
    * rotate_x(radians(in_rotate_x))
    * scale(in_scale_all, in_scale_all, in_scale_all)
    * scale(4.0/3.0,1.0,1.0)
    * vec4(position.xy, position.z * -1, 1.0);

  float perspective_factor = 1.0; // projected_position.z * 0.5 + 1.0;
  gl_Position = vec4(projected_position.xyz/perspective_factor, 1.0);
}
