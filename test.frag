#version 150

out vec4 out_color;
uniform sampler2D sampler;

uniform struct SimpleDirectionalLight {
  vec3 color;
  vec3 direction;
  float intensity;
} sunlight;

in Vertex {
  vec4 position;
  vec2 texcoord;
  vec3 normal;
} vs_out;

void main() {

  vec4 color;
  float z = vs_out.position.z;
  float res = 0.125;

  vec4 water = vec4(0.0, 0.0, 0.5, 1.0);
  vec4 shore = vec4(0.0, 0.5, 1.0, 1.0);
  vec4 sand  = vec4(0.9, 0.9, 0.2, 1.0);

  vec4 grass = vec4(0.1, 0.6, 0.0, 1.0);
  vec4 dirt  = vec4(0.8, 0.8, 0.0, 1.0);
  vec4 rock  = vec4(0.5, 0.5, 0.5, 1.0);
  vec4 snow  = vec4(1.0, 1.0, 1.0, 1.0);

  float s_water = res * 0.05;
  float s_shore = res * 0.10;
  float s_sand  = res * 0.15;
  float s_grass = res * 0.22;
  float s_dirt  = res * 0.50;
  float s_rock  = res * 0.75;
  float s_snow  = res * 0.99;

  color = mix(water, shore, smoothstep(s_water, s_shore, z));
  color = mix(color, sand,  smoothstep(s_shore, s_sand,  z));
  color = mix(color, grass, smoothstep(s_sand,  s_grass, z));
  color = mix(color, dirt,  smoothstep(s_grass, s_dirt,  z));
  color = mix(color, rock,  smoothstep(s_dirt,  s_rock,  z));
  color = mix(color, snow,  smoothstep(s_rock,  res,     z));

  vec4 tex = texture(sampler, vs_out.texcoord);
  float diffuse_intensity = max(0.0, dot(normalize(vs_out.normal), -sunlight.direction));

  vec4 light = vec4(sunlight.color * (sunlight.intensity + diffuse_intensity), 1.0);

  // out_color = color;
  out_color = tex * light * vec4(1.0, 1.0, 1.0, 1.0);

}
