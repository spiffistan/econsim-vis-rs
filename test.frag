#version 150
out vec4 out_color;
in vec3 fs_position;

void main() {

    vec4 color;
    float z = fs_position.z;
    float res = 256.0;

    vec4 water = vec4(0.0, 0.0, 0.5, 1.0);
    vec4 shore = vec4(0.0, 0.5, 1.0, 1.0);
    vec4 sand  = vec4(0.9, 0.9, 0.2, 1.0);

    vec4 grass = vec4(0.1, 0.6, 0.0, 1.0);
    vec4 dirt  = vec4(0.8, 0.8, 0.0, 1.0);
    vec4 rock  = vec4(0.5, 0.5, 0.5, 1.0);
    vec4 snow  = vec4(1.0, 1.0, 1.0, 1.0);


    float s_water = res * 0.03;
    float s_shore = res * 0.07;
    float s_sand  = res * 0.12;
    float s_grass = res * 0.22;
    float s_dirt  = res * 0.37;
    float s_rock  = res * 0.75;
    float s_snow  = res * 0.98;

    color = mix(water, shore, smoothstep(s_water, s_shore, z));
    color = mix(color, sand,  smoothstep(s_shore, s_sand,  z));
    color = mix(color, grass, smoothstep(s_sand,  s_grass, z));
    color = mix(color, dirt,  smoothstep(s_grass, s_dirt,  z));
    color = mix(color, rock,  smoothstep(s_dirt,  s_rock,  z));
    color = mix(color, snow,  smoothstep(s_rock,  res,     z));


    out_color = color;

}

  //
  // renderer.AddGradientPoint (-1.0000, utils::Color (  0,   0, 128, 255)); // deeps
  // renderer.AddGradientPoint (-0.2500, utils::Color (  0,   0, 255, 255)); // shallow
  // renderer.AddGradientPoint ( 0.0000, utils::Color (  0, 128, 255, 255)); // shore
  // renderer.AddGradientPoint ( 0.0625, utils::Color (240, 240,  64, 255)); // sand
  // renderer.AddGradientPoint ( 0.1250, utils::Color ( 32, 160,   0, 255)); // grass
  // renderer.AddGradientPoint ( 0.3750, utils::Color (224, 224,   0, 255)); // dirt
  // renderer.AddGradientPoint ( 0.7500, utils::Color (128, 128, 128, 255)); // rock
  // renderer.AddGradientPoint ( 1.0000, utils::Color (255, 255, 255, 255)); // snow
