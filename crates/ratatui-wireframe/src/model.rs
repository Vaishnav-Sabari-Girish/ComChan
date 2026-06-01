use crate::alloc::vec::Vec;
use spin::Once;
use wrfm::WrfmModel;

/// Represents a 3D wireframe model consisting of vertices and edges.
#[derive(Clone)]
pub struct Model {
    pub vertices: Vec<(f64, f64, f64)>,
    pub edges: Vec<(usize, usize)>,
}

impl Model {
    pub fn from_wrfm(wrfm: WrfmModel) -> Self {
        Self {
            vertices: wrfm.vertices,
            edges: wrfm.edges,
        }
    }

    pub fn cube() -> &'static Self {
        static CUBE: Once<Model> = Once::new();

        CUBE.call_once(|| {
            let cube_data = include_str!("./models/cube.wrfm");
            let wrfm = WrfmModel::from_str("cube", cube_data).unwrap();
            Self::from_wrfm(wrfm)
        })
    }

    pub fn tetrahedron() -> &'static Self {
        static TETRAHEDRON: Once<Model> = Once::new();

        TETRAHEDRON.call_once(|| {
            let tetra_data = include_str!("./models/tetrahedron.wrfm");
            let wrfm = WrfmModel::from_str("tetrahedron", tetra_data).unwrap();
            Self::from_wrfm(wrfm)
        })
    }

    pub fn octahedron() -> &'static Self {
        static OCTAHEDRON: Once<Model> = Once::new();

        OCTAHEDRON.call_once(|| {
            let octa_data = include_str!("./models/octahedron.wrfm");
            let wrfm = WrfmModel::from_str("octahedron", octa_data).unwrap();
            Self::from_wrfm(wrfm)
        })
    }
}
