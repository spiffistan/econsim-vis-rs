#version 330

layout (location = 0) in vec3 position;
layout (location = 1) in vec3 normal;
layout (location = 2) in vec2 texcoord;

uniform mat4 M;
uniform mat4 V;
uniform mat4 P;

uniform vec3 scaling;
uniform vec3 translation;

uniform struct SimpleDirectionalLight {
  vec3 color;
  vec3 direction;
  float intensity;
} sunlight;

out Vertex {
  vec3 position;
  vec2 texcoord;
  vec3 normal;
} vs_out;

out vec3 LightDirection_cameraspace;
out vec3 Normal_cameraspace;
out vec3 EyeDirection_cameraspace;
out vec3 Position_worldspace;

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

mat4 MVP(void) {
  return P * V * M;
}

void main() {

  vs_out.normal = normal;
  vs_out.position = position;
  vs_out.texcoord = texcoord;

  // mat4 projected = MVP();
    // * translate(translation.x, translation.y, translation.z)
    // * scale(scaling.x, scaling.y, 1.0);
    // * projection_matrix
    // * rotate(vec3(0,0,1), radians(15))
    // * rotate(vec3(1,0,0), radians(-15))

  gl_Position = MVP() * vec4(position.xyz, 1.0);

  //
  // // Position of the vertex, in worldspace : M * position
  // Position_worldspace = (M * vec4(position,1)).xyz;
  //
  // // Vector that goes from the vertex to the camera, in camera space.
  // // In camera space, the camera is at the origin (0,0,0).
  // vec3 vertexPosition_cameraspace = ( V * M * vec4(position, 1)).xyz;
  // EyeDirection_cameraspace = vec3(0,0,0) - vertexPosition_cameraspace;
  //
  // // Vector that goes from the vertex to the light, in camera space. M is ommited because it's identity.
  // vec3 LightPosition_cameraspace = ( V * vec4(0, 1, 4, 1)).xyz;
  // LightDirection_cameraspace = LightPosition_cameraspace + EyeDirection_cameraspace;
  //
  // // Normal of the the vertex, in camera space
  // Normal_cameraspace = ( V * M * vec4(normal, 0)).xyz;
  // // Only correct if ModelMatrix does not scale the model ! Use its inverse transpose if not.



}
