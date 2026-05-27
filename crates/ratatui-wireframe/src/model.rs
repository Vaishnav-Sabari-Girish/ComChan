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
        let data = include_str!("./models/cube.wrfm");
        let wrfm = WrfmModel::from_str("cube", data).unwrap();
        Self::from_wrfm(wrfm)
    }

    pub fn tetrahedron() -> Self {
        let data = include_str!("./models/tetrahedron.wrfm");
        let wrfm = WrfmModel::from_str("tetrahedron", data).unwrap();
        Self::from_wrfm(wrfm)
    }
}
