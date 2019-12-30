pub mod cs {
    vulkano_shaders::shader! {ty: "compute", path: "src/shaders/comp.glsl"}
}
pub mod vs {
    vulkano_shaders::shader! {ty: "vertex", path:"src/shaders/vert.glsl"}
}
pub mod fs {
    vulkano_shaders::shader! {ty: "fragment", path:"src/shaders/frag.glsl"}
}
// Used to force recompilation of shader change
#[allow(dead_code)]
const SHADER1: &str = include_str!("comp.glsl");
#[allow(dead_code)]
const SHADER2: &str = include_str!("vert.glsl");
#[allow(dead_code)]
const SHADER3: &str = include_str!("frag.glsl");
