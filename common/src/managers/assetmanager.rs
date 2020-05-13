use crate::{managers::AssetManager, Entity, Mesh, Octonion, Quaternion};
use num_traits::{Float, Zero};
use std::{fs, io, path::PathBuf};
use tobj::{
    Model,
    {self},
};

impl AssetManager {
    pub fn new() -> Self {
        let assets_path = PathBuf::from("./graphics/assets");
        let assets_paths = fs::read_dir(&assets_path)
            .expect("Could not read assets.")
            .map(|res| res.map(|e| e.path()))
            .collect::<Result<Vec<_>, io::Error>>()
            .expect("Could not load assets.");
        let mut objects: Vec<Model> = vec![];
        // let mut names: Vec<String> = vec![];
        for p in assets_paths {
            //discard materials with .0
            for o in tobj::load_obj(&p, true).unwrap().0 {
                objects.push(o);
            }
            // names.push(p.file_stem().unwrap().to_str().unwrap().to_string());
        }
        Self {
            // assets_path,
            objects,
            // names,
        }
    }

    pub fn load<T: Float + From<f32>>(
        &self,
        n: &str,
    ) -> Entity<T> {
        let m = Mesh::<T>::from_tobj_to_mesh(
            &self.objects[self.objects.iter().position(|r| r.name == n).unwrap()]
                .mesh,
            [T::zero(); 3],
            0.5,
        );
        Entity {
            pos:    Octonion::zero(),
            orient: Quaternion::zero(),
            len:    m.positions.len(),
            model:  m,
        }
    }
}
