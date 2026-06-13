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

    #[cfg(feature = "ratty")]
    pub fn from_obj(obj_string: &str) -> Result<Self, crate::alloc::string::String> {
        let mut vertices = Vec::new();
        let mut edges = Vec::new();

        for (line_num, line) in obj_string.lines().enumerate() {
            let line = line.trim();
            if line.is_empty() || line.starts_with('#') {
                continue;
            }

            let mut parts = line.split_whitespace();
            let Some(prefix) = parts.next() else {
                continue;
            };

            match prefix {
                "v" => {
                    let x = parts
                        .next()
                        .and_then(|s| s.parse::<f64>().ok())
                        .ok_or_else(|| {
                            crate::alloc::format!("Invalid X on line {}", line_num + 1)
                        })?;
                    let y = parts
                        .next()
                        .and_then(|s| s.parse::<f64>().ok())
                        .ok_or_else(|| {
                            crate::alloc::format!("Invalid Y on line {}", line_num + 1)
                        })?;
                    let z = parts
                        .next()
                        .and_then(|s| s.parse::<f64>().ok())
                        .ok_or_else(|| {
                            crate::alloc::format!("Invalid Z on line {}", line_num + 1)
                        })?;

                    vertices.push((x, y, z));
                }
                "f" | "l" => {
                    let mut indices = Vec::new();
                    for part in parts {
                        let v_str = part.split('/').next().unwrap_or("");
                        let idx = v_str.parse::<usize>().map_err(|_| {
                            crate::alloc::format!("Invalid Index on line {}", line_num + 1)
                        })?;

                        if idx == 0 {
                            return Err(crate::alloc::format!(
                                "OBJ indices are 1-based, found 0 on line {}",
                                line_num + 1
                            ));
                        }

                        indices.push(idx - 1);
                    }

                    if indices.len() >= 2 {
                        let is_polygon = prefix == "f";
                        let limit = if is_polygon {
                            indices.len()
                        } else {
                            indices.len() - 1
                        };

                        let range = core::range::Range {
                            start: 0,
                            end: limit,
                        };

                        for i in range {
                            let start = indices[i];
                            let end = indices[(i + 1) % indices.len()];

                            let edge = if start < end {
                                (start, end)
                            } else {
                                (end, start)
                            };

                            edges.push(edge);
                        }
                    }
                }
                _ => {}
            }
        }

        edges.sort_unstable();
        edges.dedup();

        Ok(Self { vertices, edges })
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
