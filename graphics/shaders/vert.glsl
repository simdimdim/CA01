#version 450
layout(location = 0) in vec3 position;
layout(location = 1) in vec4 orient;
layout(location = 0) out vec4 color;

void main() {
    gl_Position = vec4 (position.xy, 0.0,1.0);
    color= normalize(orient*vec4(position,0.0));
}
