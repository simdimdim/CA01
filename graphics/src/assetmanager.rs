use common::Entity;
use std::path::PathBuf;
use tobj;

struct AssetManager {
    assets_path: PathBuf,
}
impl AssetManager {
    pub fn new() -> Self {
        let assets_path = PathBuf::from("../assets/cube.obj");
        let box_obj = tobj::load_obj(&assets_path);
        Self { assets_path }
    }

    pub fn load(&mut self) {}
}
