#version 450
layout(location = 0) out vec4 f_color;
layout(location = 1) in vec3 color;

layout(origin_upper_left) in vec4 gl_FragCoord;
void main() {
f_color = vec4(normalize(color),1.0);
}
