#version 450
vec4 quat_conj(vec4 q){
  return vec4(q.x, -q.y, -q.z, -q.w);
}
vec4 quat_mul(vec4 q1, vec4 q2){
  vec4 qr;
  qr.x = (q1.x * q2.x) - (q1.y * q2.y) - (q1.z * q2.z) - (q1.w * q2.w);
  qr.y = (q1.x * q2.y) + (q1.y * q2.x) + (q1.z * q2.w) - (q1.w * q2.z);
  qr.z = (q1.x * q2.z) - (q1.y * q2.w) + (q1.z * q2.x) + (q1.w * q2.y);
  qr.w = (q1.x * q2.w) + (q1.y * q2.z) - (q1.z * q2.y) + (q1.w * q2.x);
  return qr;
}
vec4 rotate(vec4 pos, vec4 rotator){
  vec3 a=quat_mul(quat_mul(rotator,vec4(0.0,pos.xyz)),quat_conj(rotator)).yzw;
  return vec4(a.xy,(a.z+1.0)/2.0,1.0);
}
struct vert {
    vec4 position;
    vec4 orient;
};
layout(local_size_x = 1, local_size_y = 1, local_size_z = 1) in;
layout(set = 0, binding = 0) buffer Data {
	vert data[];
} buf;

void main() {
  uint idx = gl_GlobalInvocationID.x;
 	buf.data[idx].position = rotate(buf.data[idx].position,buf.data[idx].orient);
}
