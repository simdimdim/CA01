// use std::path::PathBuf;
use tobj::Model;

pub mod assetmanager;

#[derive(Default)]
pub struct AssetManager {
    // assets_path: PathBuf,
    // names:   Vec<String>,
    objects: Vec<Model>,
}
