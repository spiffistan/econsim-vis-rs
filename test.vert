#version 330

layout (location = 0) in vec4 position;
layout (location = 1) in vec3 normal;
layout (location = 2) in vec2 texcoord;

uniform mat4 projection_matrix;
// uniform mat4 view_matrix;
// uniform mat4 model_matrix;

uniform vec3 scaling;
uniform vec3 translation;

out Vertex {
  vec4 position;
  vec2 texcoord;
  vec3 normal;
} vs_out;

mat4 rotate(vec3 axis, float angle)
{
    axis = normalize(axis);
    float s = sin(angle);
    float c = cos(angle);
    float oc = 1.0 - c;

    return mat4(oc * axis.x * axis.x + c,           oc * axis.x * axis.y - axis.z * s,  oc * axis.z * axis.x + axis.y * s,  0.0,
                oc * axis.x * axis.y + axis.z * s,  oc * axis.y * axis.y + c,           oc * axis.y * axis.z - axis.x * s,  0.0,
                oc * axis.z * axis.x - axis.y * s,  oc * axis.y * axis.z + axis.x * s,  oc * axis.z * axis.z + c,           0.0,
                0.0,                                0.0,                                0.0,                                1.0);
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

void main() {

  vs_out.normal = normal;
  vs_out.position = position;
  vs_out.texcoord = texcoord;

  vec4 projected_position =
    translate(translation.x, translation.y, translation.z)
    * scale(scaling.x, scaling.y, 1.0)
    // * projection_matrix
    * rotate(vec3(0,0,1), radians(15))
    * rotate(vec3(1,0,0), radians(-15))
    * vec4(position.xy, position.z * -1.0,  1.0);

  float perspective_factor = 1.0; // projected_position.z * 0.5 + 1.0;
  gl_Position = vec4(projected_position.xyz/perspective_factor, 1.0);
}
