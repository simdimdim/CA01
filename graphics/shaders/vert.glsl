#version 450
layout(location = 0) in vec4 position;
layout(location = 1) in vec4 orient;
layout(location = 0) out vec4 color;

void main() {
    //TODO: add HiDPI scaling as push_constant
    gl_Position = position;
    color=gl_Position;

}
