#version 150

out vec4 out_color;
uniform sampler2D sampler;

in Vertex {
  vec2 vs_uv;
  vec3 vs_worldpos;
  vec3 vs_normal;
} vs_out;

void main() {

    vec4 color;
    float z = vs_out.vs_worldpos.z;
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

    out_color = texture(sampler, vs_out.vs_uv);

    // out_color = vec4(1.0, 1.0, vs_out.vs_worldpos.z / 128.0, 1.0);
}
