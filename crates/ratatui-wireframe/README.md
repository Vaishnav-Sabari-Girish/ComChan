# `ratatui-wireframe`

[![Crates.io](https://img.shields.io/crates/v/ratatui-wireframe.svg)](https://crates.io/crates/ratatui-wireframe)
[![License: MIT](https://img.shields.io/badge/license-MIT-blue.svg)](LICENSE)

`ratatui-wireframe` is a simple, plug-and-play widget for
[ratatui](https://github.com/ratatui-org/ratatui) that allows you to render and
rotate 3D wireframe models in your terminal.

## Features

* **Zero-Dependency 3D Rendering:** Uses pure math to project 3D space into
  terminal Braille characters.
* **Dynamic Orientation:** Easily pass pitch, yaw, and roll to rotate your
  models in real-time.
* **Integrated Axes:** Includes a built-in stationary X/Y/Z axis triad (gnomon)
  for spatial context.
* **Adaptive Layout:** Automatically handles aspect ratio scaling to prevent
  model distortion when resizing your terminal window.

## Installation

Add this to your `Cargo.toml`:

```toml
[dependencies]
ratatui = "0.26.0"
ratatui-wireframe = "0.1.0"
```

## Quick Start

Simply initialize the `WireframeWidget` in your draw loop with your rotation
data:

```rust
use ratatui_wireframe::WireframeWidget;

// Inside your terminal draw loop:
let wireframe = WireframeWidget::new(pitch_rad, yaw_rad, roll_rad)
    .title("3D Telemetry")
    .color(Color::Cyan);

f.render_widget(wireframe, chunk);
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
