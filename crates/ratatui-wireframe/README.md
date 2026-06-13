# `ratatui-wireframe`

[![Crates.io](https://img.shields.io/crates/v/ratatui-wireframe.svg)](https://crates.io/crates/ratatui-wireframe)
[![License: MIT](https://img.shields.io/badge/license-MIT-blue.svg)](LICENSE)

`ratatui-wireframe` is a simple, plug-and-play widget for
[ratatui](https://github.com/ratatui-org/ratatui) that allows you to render and
rotate 3D wireframe models in your terminal.

## Features

* **Zero-Dependency 3D Rendering (Braille):** Uses pure math to project 3D space
  into terminal Braille characters.
* **3D models rendering via `ratty`**: Uses the `ratatui-ratty` crate and the
  **Ratty Graphics Protocol** to render actual 3D models from an `.obj` file.
* **Dynamic Orientation:** Easily pass pitch, yaw, and roll to rotate your
  models in real-time.
* **Built-in Models:** Comes with default geometric primitives (Cube,
  Tetrahedron, Octahedron).
* **Custom Model Loading:** Seamlessly load custom `.wrfm` files, or enable the
  `ratty` feature to natively parse and render standard 3D `.obj` files!
* **Integrated Axes:** Includes a built-in stationary X/Y/Z axis triad (gnomon)
  for spatial context.
* **Adaptive Layout:** Automatically handles aspect ratio scaling to prevent
  model distortion when resizing your terminal window.

## Installation

Add this to your `Cargo.toml`:

```toml
[dependencies]
ratatui = "0.26.0"
ratatui-wireframe = "0.7.0"

# Or, to enable native .obj file parsing:
# ratatui-wireframe = { version = "0.7.0", features = ["ratty"] }
```

## Quick Start

Initialize a `Model` and pass it to the `WireframeWidget` in your draw loop
along with your rotation data:

```rust
use ratatui_wireframe::{WireframeWidget, model::Model};
use ratatui::style::Color;

// 1. Select your model (Built-in, .wrfm, or .obj)
let my_model = Model::cube(); 

// Inside your terminal draw loop:
// 2. Pass the model and Euler angles (in radians)
let wireframe = WireframeWidget::new(pitch_rad, yaw_rad, roll_rad)
    .title("3D Telemetry")
    .color(Color::Cyan)
    .model(my_model);

f.render_widget(wireframe, chunk);
```

## Loading Custom Models

You can easily load your own 3D shapes.

**Loading a `.wrfm` file (Default):**

```rust
use wrfm::WrfmModel;
use ratatui_wireframe::model::Model;

let parsed_wrfm = WrfmModel::from_file("my_drone.wrfm").unwrap();
let custom_model = Model::from_wrfm(parsed_wrfm);
```

**Loading a `.obj` file (Requires the `ratty` feature):**

```rust
use ratatui_wireframe::model::Model;

let obj_data = std::fs::read_to_string("spaceship.obj").unwrap();
let custom_model = Model::from_obj(&obj_data).unwrap();
```

## How it works

1. **Vertex Transformation:** The widget takes a `Model` (a collection of 3D
   vertices and edge connections) and applies standard rotation matrices based
   on the provided pitch, yaw, and roll.
2. **Projection:** It applies a perspective projection calculation to create
   depth.
3. **Rendering:** It utilizes the `ratatui::widgets::canvas::Canvas` widget with
   the `Braille` marker symbol to draw the transformed lines onto the terminal
   buffer.

## License

Licensed under the [MIT License](LICENSE).
