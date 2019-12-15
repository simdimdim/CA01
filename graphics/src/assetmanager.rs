use crate::AssetManager;
use common::{Entity, Mesh, Octonion, Quaternion};
use num_traits::{Float, Zero};
use std::path::PathBuf;
use tobj::{self};

impl AssetManager {
    pub fn new() -> Self {
        let assets_path = PathBuf::from("./graphics/assets/cube.obj");
        // if assets_path.exists() {
        //     print!("{:?}", std::fs::canonicalize(&assets_path));
        // }
        //discard materials with .0
        let objects = tobj::load_obj(&assets_path).unwrap().0;
        let names = vec!["cube".to_string()];
        Self {
            assets_path,
            objects,
            names,
        }
    }

    pub fn load<T: Float + From<f32>>(
        &self,
        n: &str,
    ) -> Entity<T> {
        Entity {
            pos:    Octonion::zero(),
            orient: Quaternion::zero(),
            model:  Mesh::<T>::from_tobj_to_mesh(
                &self.objects[self.names.iter().position(|r| r == n).unwrap()]
                    .mesh,
                [T::zero(); 3],
                0.5,
            ),
        }
    }
}
