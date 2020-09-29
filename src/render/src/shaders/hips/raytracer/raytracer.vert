#version 300 es
precision highp float;
precision lowp sampler2D;
precision lowp isampler2D;
precision highp int;

layout (location = 0) in vec2 pos_clip_space;
layout (location = 1) in vec3 pos_world_space;

out vec3 out_vert_pos;

uniform mat4 model;
uniform vec2 ndc_to_clip;
uniform float clip_zoom_factor;

const mat4 GALACTIC_TO_J2000 = mat4(
    -0.0548755604024359,
    -0.8734370902479237,
    -0.4838350155267381,
    0.0,
    
    0.4941094279435681,
    -0.4448296299195045,
    0.7469822444763707,
    0.0, 
    
    -0.8676661489811610,
    -0.1980763734646737,
    0.4559837762325372,
    0.0,

    0.0,
    0.0,
    0.0,
    1.0
);
const mat4 J2000_TO_GALACTIC = mat4(
    -0.8676661489811610,
    -0.0548755604024359,
    0.4941094279435681,
    0.0,

    -0.1980763734646737,
    -0.873437090247923,
    -0.4448296299195045,
    0.0,

    0.4559837762325372,
    -0.4838350155267381,
    0.7469822444763707,
    0.0,

    0.0,
    0.0,
    0.0,
    1.0
);

void main() {
    gl_Position = vec4(pos_clip_space / (ndc_to_clip * clip_zoom_factor), 0.0, 1.0);
    //out_vert_pos = vec3(inverse(gal_to_icrs) * model * vec4(pos_world_space, 1.f));
    out_vert_pos = vec3(model * vec4(pos_world_space, 1.f));
}