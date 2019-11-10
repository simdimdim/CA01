#version 450
layout(location = 0) in vec4 color;
layout(location = 0) out vec4 f_color;

layout(origin_upper_left) in vec4 gl_FragCoord;
void main() {
f_color = gl_FragCoord/color;
}
