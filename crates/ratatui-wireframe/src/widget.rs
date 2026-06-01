use crate::model::Model;
use alloc::string::{String, ToString};
use alloc::vec::Vec;
use ratatui::{
    buffer::Buffer,
    layout::Rect,
    style::{Color, Style},
    text::Span,
    widgets::{
        Block, Borders, Widget,
        canvas::{Canvas, Line},
    },
};

pub struct WireframeWidget<'a> {
    title: String,
    color: Color,
    pitch: f64,
    yaw: f64,
    roll: f64,
    model: Option<&'a Model>,
}

impl<'a> WireframeWidget<'a> {
    pub fn new(pitch: f64, yaw: f64, roll: f64) -> Self {
        Self {
            title: "3D Telemetry".to_string(),
            color: Color::Cyan,
            pitch,
            yaw,
            roll,
            model: Some(Model::cube()), // Default to the cube
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

    pub fn model(mut self, model: &'a Model) -> Self {
        self.model = Some(model);
        self
    }
}

impl<'a> Widget for WireframeWidget<'a> {
    fn render(self, area: Rect, buf: &mut Buffer) {
        let current_model = self.model.expect("WireframeWidget must have a model");
        // 1. Rotate main model vertices
        let mut rotated = Vec::new();
        for (x, y, z) in current_model.vertices.iter() {
            let y1 = y * self.pitch.cos() - z * self.pitch.sin();
            let z1 = y * self.pitch.sin() + z * self.pitch.cos();

            let x2 = x * self.yaw.cos() + z1 * self.yaw.sin();
            let z2 = -x * self.yaw.sin() + z1 * self.yaw.cos();

            let x3 = x2 * self.roll.cos() - y1 * self.roll.sin();
            let y3 = x2 * self.roll.sin() + y1 * self.roll.cos();

            rotated.push((x3, y3, z2));
        }

        // Calculate dynamic aspect ratio to prevent stretching
        let aspect = (area.width as f64) / ((area.height as f64) * 2.0);
        let y_extent = 4.0;
        let x_extent = aspect * y_extent;

        let canvas = Canvas::default()
            .block(Block::default().borders(Borders::ALL).title(self.title))
            .marker(ratatui::symbols::Marker::Braille)
            .x_bounds([-x_extent, x_extent])
            .y_bounds([-y_extent, y_extent])
            .paint(|ctx| {
                // ── A. Draw the Main 3D Object ────────────────────────────────
                for &(start_idx, end_idx) in current_model.edges.iter() {
                    let v1 = rotated[start_idx];
                    let v2 = rotated[end_idx];

                    // Perspective projection
                    let distance = 5.0;
                    let z1 = v1.2 + distance;
                    let z2 = v2.2 + distance;

                    let scale = 8.0;

                    let px1 = (v1.0 / z1) * scale;
                    let py1 = (v1.1 / z1) * scale;
                    let px2 = (v2.0 / z2) * scale;
                    let py2 = (v2.1 / z2) * scale;

                    ctx.draw(&Line {
                        x1: px1,
                        y1: py1,
                        x2: px2,
                        y2: py2,
                        color: self.color,
                    });
                }

                let p_origin = (0.0, 0.0);

                let p_x = (4.0, 0.0);
                let p_y = (0.0, 3.5);
                let p_z = (-2.8, -2.8);

                ctx.draw(&Line {
                    x1: p_origin.0,
                    y1: p_origin.1,
                    x2: p_x.0,
                    y2: p_x.1,
                    color: Color::Red,
                });

                ctx.draw(&Line {
                    x1: p_origin.0,
                    y1: p_origin.1,
                    x2: p_y.0,
                    y2: p_y.1,
                    color: Color::Yellow,
                });

                ctx.draw(&Line {
                    x1: p_origin.0,
                    y1: p_origin.1,
                    x2: p_z.0,
                    y2: p_z.1,
                    color: Color::LightBlue,
                });

                ctx.print(
                    p_x.0 + 0.2,
                    p_x.1,
                    Span::styled("X", Style::default().fg(Color::Red)),
                );
                ctx.print(
                    p_y.0,
                    p_y.1 + 0.2,
                    Span::styled("Y", Style::default().fg(Color::Yellow)),
                );
                ctx.print(
                    p_z.0 - 0.5,
                    p_z.1 - 0.5,
                    Span::styled("Z", Style::default().fg(Color::LightBlue)),
                );
            });

        canvas.render(area, buf);
    }
}
