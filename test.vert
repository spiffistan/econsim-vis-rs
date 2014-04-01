#version 330

layout (location = 0) in vec3 position;
layout (location = 1) in vec3 normal;
layout (location = 2) in vec2 texcoord;

uniform mat4 M; // Model
uniform mat4 V; // View
uniform mat4 P; // Projection

out Vertex {
  vec3 eye;
  vec3 position;
  vec2 texcoord;
  vec3 normal;
} vs_out;

out vec3 LightDirection_cameraspace;
out vec3 Normal_cameraspace;
out vec3 EyeDirection_cameraspace;
out vec3 Position_worldspace;

mat4 MVP(void) {
  return P * V * M;
}

void main() {

  vec4 eye = V * M * vec4(position.xy, position.z * -1.0, 1.0);

  vs_out.normal = (V * M * vec4(normal, 0.0)).xyz;
  vs_out.position = position;
  vs_out.eye = eye.xyz;
  vs_out.texcoord = texcoord;

  gl_Position = P * eye; // MVP() * vec4(position.xy, position.z * -1.0, 1.0);

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
