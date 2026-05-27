/// Represents a 3D wireframe model consisting of vertices and edges.
pub struct Model {
    pub vertices: Vec<(f64, f64, f64)>,
    pub edges: Vec<(usize, usize)>,
}

impl Model {
    /// Creates a standard 3D cube model.
    pub fn cube() -> Self {
        Self {
            vertices: vec![
                (-1.0, -1.0, -1.0),
                (1.0, -1.0, -1.0),
                (1.0, 1.0, -1.0),
                (-1.0, 1.0, -1.0), // Front face
                (-1.0, -1.0, 1.0),
                (1.0, -1.0, 1.0),
                (1.0, 1.0, 1.0),
                (-1.0, 1.0, 1.0), // Back face
            ],
            edges: vec![
                (0, 1),
                (1, 2),
                (2, 3),
                (3, 0), // Front facing edges
                (4, 5),
                (5, 6),
                (6, 7),
                (7, 4), // Back facing edges
                (0, 4),
                (1, 5),
                (2, 6),
                (3, 7), // Connecting edges
            ],
        }
    }

    pub fn tetrahedron() -> Self {
        Self {
            vertices: vec![
                (1.0, 1.0, 1.0),
                (1.0, -1.0, -1.0),
                (-1.0, 1.0, -1.0),
                (-1.0, -1.0, 1.0),
            ],
            edges: vec![(0, 1), (0, 2), (0, 3), (1, 2), (2, 3), (3, 1)],
        }
    }
}
