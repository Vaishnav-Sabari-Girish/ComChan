use std::sync::OnceLock;
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

    pub fn cube() -> Self {
        static CUBE: OnceLock<Model> = OnceLock::new();

        CUBE.get_or_init(|| {
            let cube_data = include_str!("./models/cube.wrfm");
            let wrfm = WrfmModel::from_str("cube", cube_data).unwrap();
            Self::from_wrfm(wrfm)
        })
        .clone()
    }

    pub fn tetrahedron() -> Self {
        static TETRAHEDRON: OnceLock<Model> = OnceLock::new();

        TETRAHEDRON
            .get_or_init(|| {
                let tetra_data = include_str!("./models/tetrahedron.wrfm");
                let wrfm = WrfmModel::from_str("tetrahedron", tetra_data).unwrap();
                Self::from_wrfm(wrfm)
            })
            .clone()
    }

    pub fn octahedron() -> Self {
        static OCTAHEDRON: OnceLock<Model> = OnceLock::new();

        OCTAHEDRON
            .get_or_init(|| {
                let octa_data = include_str!("./models/octahedron.wrfm");
                let wrfm = WrfmModel::from_str("octahedron", octa_data).unwrap();
                Self::from_wrfm(wrfm)
            })
            .clone()
    }
}
