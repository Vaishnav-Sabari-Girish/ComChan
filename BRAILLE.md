# ComChan Supported Braille Models

When using ComChan in standard terminal emulators (without the `ratty` hardware
acceleration feature), the 3D Telemetry dashboard uses a zero-dependency
CPU-rendered Braille wireframe engine.

You can select different 3D models using the `--braille` flag.

## Available Models

### 1. Cube (Default)

A standard 3D cube wireframe consisting of 8 vertices and 12 edges. It provides
the best spatial awareness for general Pitch, Yaw, and Roll movements.

**Usage:**

```bash
comchan --plot --auto --braille cube
```

*(Note: Since this is the default, passing `--braille cube` is optional).*

### 2. Tetrahedron (Triangle)

A triangular pyramid wireframe consisting of 4 vertices and 6 edges. It clearly
indicates "up" vs "down" making it excellent for visualizing directional heading
or drone telemetry.

**Usage:**

```bash
comchan --plot --auto --braille tetrahedron
```

---

*To add your own models to this list, check out the `model.rs` file within the
`ratatui-wireframe` crate!*
