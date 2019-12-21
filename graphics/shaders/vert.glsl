#version 450
layout(location = 0) in vec4 position;
layout(location = 1) in vec4 orient;
layout(location = 2) in vec4 normals;
layout(location = 0) out vec4 color;

void main() {
    //TODO: add HiDPI scaling as push_constant
    gl_Position = position;
    gl_Position.y = -gl_Position.y;
    gl_Position.z = (gl_Position.z + gl_Position.w) / 2.0;
    color=gl_Position;
}
