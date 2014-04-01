#version 150

out vec4 out_color;

uniform mat4 M; // Model
uniform mat4 V; // View
uniform mat4 P; // Projection

uniform sampler2D sampler;
uniform float timer;

in Vertex {
  vec3 eye;
  vec3 position;
  vec2 texcoord;
  vec3 normal;
} vs_out;

uniform struct SimpleDirectionalLight {
  vec3 color;
  vec3 direction;
  float intensity;
} sunlight;

// in vec3 LightDirection_cameraspace;
// in vec3 Normal_cameraspace;
// in vec3 EyeDirection_cameraspace;
// in vec3 Position_worldspace;

// vec3 face_normal( vec3 position[3], vec3 normal[3])
// {
//   vec3 p0 = position.y - position.x;
//   vec3 p1 = position.z - position.y;
//   vec3 face_normal = cross( p0, p1 );
//
//   vec3 vertex_normal = normals.x; // or you can average 3 normals.
//   float dot_product = dot( face_normal, vertex_normal );
//
//   return ( dot_product < 0.0f ) ? -face_normal : face_normal;
// }

vec4 rgba2vec4(int r, int g, int b, int a) {
  return vec4(r/256.0, g/256.0, b/256.0, a/256.0);
}

const vec3 light_direction = vec3(0.408248, 0.816497, 0.408248);
const vec4 light_diffuse = vec4(0.8, 0.8, 0.8, 0.0);
const vec4 light_ambient = vec4(0.1, 0.1, 0.1, 1.0);
const vec4 light_specular = vec4(1.0, 1.0, 1.0, 1.0);

void main() {

  vec4 color;
  float z = vs_out.position.z * -1.0;
  float res = 256;

  vec4 water = rgba2vec4(7,103,163,1);
  vec4 shore = rgba2vec4(3,54,73,1);
  vec4 sand  = rgba2vec4(205,179,179,1);
  vec4 grass = rgba2vec4(0,80,9,1);
  vec4 dirt = rgba2vec4(0,30,9,1);

  vec4 rock  = vec4(0.5, 0.5, 0.5, 1.0);
  vec4 snow  = vec4(1.0, 1.0, 1.0, 1.0);

  float s_water = res * 0.03;
  float s_shore = res * 0.04;
  float s_sand  = res * 0.045;
  float s_grass = res * 0.09;
  float s_dirt  = res * 0.60;
  float s_rock  = res * 0.88;
  float s_snow  = res * 0.99;

  color = mix(water, shore, smoothstep(s_water, s_shore, z));
  color = mix(color, sand,  smoothstep(s_shore, s_sand,  z));
  color = mix(color, grass, smoothstep(s_sand,  s_grass, z));
  color = mix(color, dirt,  smoothstep(s_grass, s_dirt,  z));
  color = mix(color, rock,  smoothstep(s_dirt,  s_rock,  z));
  color = mix(color, snow,  smoothstep(s_rock,  res,     z));

  // vec4 tex = texture(sampler, vs_out.position.xy * vec2(512));
  //float diffuse_intensity = max(0.0, dot(normalize(vs_out.normal), normalize(vec3(sunlight.direction.xy, sunlight.direction.z * sin(timer)))));
//
//   // vec4 light = vec4(sunlight.color * (sunlight.intensity + diffuse_intensity), 1.0);
//
// // Normal of the computed fragment, in camera space
//  vec3 n = normalize( Normal_cameraspace );
//  // Direction of the light (from the fragment to the light)
//  vec3 l = normalize( LightDirection_cameraspace );
//
// float light_angle = clamp( dot( n,l ), 0,1 );
//
//   // out_color = color * tex;
//   out_color = color * light_angle; // * vec4(vs_out.normal, 0);

    float angle = timer / 20;
    float sun_x = sin(angle) * 10 + 10;
    float sun_z = cos(angle) * 10 + 10;

    vec3 sun = (V * vec4(sun_x, 10.0, sun_z, 0.0)).xyz;

    vec3 v = (V * M * vec4(vs_out.position.xy, vs_out.position.z * -1, 0)).xyz;
    vec3 N = (V * M * vec4(vs_out.normal, 0)).xyz;

    vec3 L = normalize(sun - v);
    vec3 E = normalize(v);
    vec3 R = normalize(-reflect(L,N));

    // vec3 mv_light_direction = (V * vec4(light_direction, 0.0)).xyz,
    //      normal = normalize(vs_out.normal),
    //      eye = normalize(vs_out.eye),
    //      reflection = reflect(mv_light_direction, normal);

    // vec4 frag_diffuse = texture2D(texture, frag_texcoord);
    vec4 diffuse_factor = max(-dot(N, L), 0.0) * light_diffuse;
    vec4 ambient_diffuse_factor = diffuse_factor + light_ambient;

    vec4 specular_factor = pow(max(-dot(R, E), 0.0), 2.0) * light_specular;
    specular_factor = clamp(specular_factor, 0.0, 2.0);

    out_color = color * (specular_factor + ambient_diffuse_factor);

    // out_color = specular_factor * frag_specular
    //     + ambient_diffuse_factor * frag_diffuse;

}
