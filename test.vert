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

mat4 MVP(void) {
  return P * V * M;
}

const float pi = 3.14159;
const float water_height = 5;
const int num_waves = 1;

const float amplitude = 0.5;
const float wavelength = 5;
const float speed = 0.02;
const vec2 direction = vec2(0.3, 0.2);

uniform float timer;

float wave(float x, float y) {
    float frequency = 2*pi/wavelength;
    float phase = speed * frequency;
    float theta = dot(direction, vec2(x, y));
    return amplitude * sin(theta * frequency + timer * phase);
}

float wave_height(float x, float y) {
    float height = 0.0;
    for (int i = 0; i < num_waves; ++i)
      height += wave(x, y);
    return height;
}

float d_wavedx(float x, float y) {
    float frequency = 2*pi/wavelength;
    float phase = speed * frequency;
    float theta = dot(direction, vec2(x, y));
    float A = amplitude * direction.x * frequency;
    return A * cos(theta * frequency + timer * phase);
}

float d_wavedy(float x, float y) {
    float frequency = 2*pi/wavelength;
    float phase = speed * frequency;
    float theta = dot(direction, vec2(x, y));
    float A = amplitude * direction.y * frequency;
    return A * cos(theta * frequency + timer * phase);
}

vec3 wave_normal(float x, float y) {
    float dx = 0.0;
    float dy = 0.0;
    for (int i = 0; i < num_waves; ++i) {
        dx += d_wavedx(x, y);
        dy += d_wavedy(x, y);
    }
    vec3 n = vec3(-dx, -dy, 1.0);
    return normalize(n);
}

// void main() {
//     vec4 pos = gl_Vertex;
//     pos.z = waterHeight + waveHeight(pos.x, pos.y);
//     position = pos.xyz / pos.w;
//     worldNormal = waveNormal(pos.x, pos.y);
//     eyeNormal = gl_NormalMatrix * worldNormal;
//     gl_Position = gl_ModelViewProjectionMatrix * pos;
// }

void main() {

  vec4 pos = vec4(position.xy, position.z * -1.0, 1.0);
  vec3 nor = vec4(normal, 0.0).xyz;

  if (pos.z *-1 <= water_height) {
    pos.z = water_height + wave_height(pos.x, pos.y);
    nor = wave_normal(pos.x, pos.y);
  }

  vs_out.normal = nor;
  vs_out.position = pos.xyz;
  vs_out.eye = (V * M * pos).xyz;
  vs_out.texcoord = texcoord;

  gl_Position = MVP() * pos;
}
