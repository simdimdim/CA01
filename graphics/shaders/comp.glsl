#version 450
dvec4 quat_conj(dvec4 q){ 
  return dvec4(q.w, -q.x, -q.y, -q.z); 
}
dvec4 quat_mul(dvec4 q1, dvec4 q2){ 
  dvec4 qr;
  qr.x = (q1.x * q2.x) - (q1.y * q2.y) - (q1.z * q2.z) - (q1.w * q2.w);
  qr.y = (q1.x * q2.y) + (q1.y * q2.x) + (q1.z * q2.w) - (q1.w * q2.z);
  qr.z = (q1.x * q2.z) - (q1.y * q2.w) + (q1.z * q2.x) + (q1.w * q2.y);
  qr.w = (q1.x * q2.w) + (q1.y * q2.z) - (q1.z * q2.y) + (q1.w * q2.x);
  return qr;
}
struct mesh {
    dvec3 position;
    dvec4 orient;
};
layout(local_size_x = 1, local_size_y = 1, local_size_z = 1) in;
layout(set = 0, binding = 0) buffer Data {
	mesh data[];
} buf;

void main() {
	uint idx = gl_GlobalInvocationID.x;
	buf.data[idx].position = vec3(quat_mul(quat_mul(buf.data[idx].orient, dvec4(0.0,buf.data[idx].position.xyz)),dvec4(buf.data[idx].orient.x,dvec3(buf.data[idx].orient.yzw))));
}
