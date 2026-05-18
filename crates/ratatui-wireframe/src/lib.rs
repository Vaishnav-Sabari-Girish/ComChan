use ratatui::{
    buffer::Buffer,
    layout::Rect,
    style::{Color, Style},
    widgets::{
        Block, Borders, Widget,
        canvas::{Canvas, Context, Line},
    },
};

pub struct WireframeWidget {
    title: String,
    color: Color,
    pitch: f64,
    yaw: f64,
    roll: f64,
}

impl WireframeWidget {
    pub fn new(pitch: f64, yaw: f64, roll: f64) -> Self {
        Self {
            title: "3D Telemetry".to_string(),
            color: Color::Cyan,
            pitch,
            yaw,
            roll,
        }
    }

    pub fn title<S: Into<String>>(mut self, title: S) -> Self {
        self.title = title.into();
        self
    }

    pub fn color(mut self, color: Color) -> Self {
        self.color = color;
        self
    }
}

impl Widget for WireframeWidget {
    fn render(self, area: Rect, buf: &mut Buffer) {
        let vertices = [
            (-1.0, -1.0, -1.0),
            (1.0, -1.0, -1.0),
            (1.0, 1.0, -1.0),
            (-1.0, 1.0, -1.0), // Front face
            (-1.0, -1.0, 1.0),
            (1.0, -1.0, 1.0),
            (1.0, 1.0, 1.0),
            (-1.0, 1.0, 1.0), // Back face
        ];

        let edges = [
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
        ];

        let mut rotated = Vec::new();
        for (x, y, z) in vertices.iter() {
            // Rotate X (Pitch)
            let y1 = y * self.pitch.cos() - z * self.pitch.sin();
            let z1 = y * self.pitch.sin() + z * self.pitch.cos();

            // Rotate Y (Yaw)
            let x2 = x * self.yaw.cos() + z1 * self.yaw.sin();
            let z2 = -x * self.yaw.sin() + z1 * self.yaw.cos();

            // Rotate Z (Roll)
            let x3 = x2 * self.roll.cos() - y1 * self.roll.sin();
            let y3 = x2 * self.roll.sin() + y1 * self.roll.cos();

            rotated.push((x3, y3, z2));
        }

        let aspect = (area.width as f64) / ((area.height as f64) * 2.0);

        let y_extent = 4.0;
        let x_extent = aspect * y_extent;

        let canvas = Canvas::default()
            .block(Block::default().borders(Borders::ALL).title(self.title))
            .marker(ratatui::symbols::Marker::Braille)
            .x_bounds([-x_extent, x_extent])
            .y_bounds([-y_extent, y_extent])
            .paint(|ctx| {
                for &(start_idx, end_idx) in edges.iter() {
                    let v1 = rotated[start_idx];
                    let v2 = rotated[end_idx];

                    // Perspective projection: Move object away by adding to Z, then divide X and Y
                    // by Z
                    let distance = 3.0;
                    let z1 = v1.2 + distance;
                    let z2 = v2.2 + distance;

                    let px1 = (v1.0 / z1) * 4.0;
                    let py1 = (v1.1 / z1) * 4.0;

                    let px2 = (v2.0 / z2) * 4.0;
                    let py2 = (v2.1 / z2) * 4.0;

                    ctx.draw(&Line {
                        x1: px1,
                        y1: py1,
                        x2: px2,
                        y2: py2,
                        color: self.color,
                    });
                }
            });

        canvas.render(area, buf);
    }
}
