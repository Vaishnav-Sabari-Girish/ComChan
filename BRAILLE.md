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

### 3. Octahedron

A diamond/crystal wireframe consisting of 6 vertices and 12 edges. Its sharp
points and symmetrical design provide excellent, highly readable visual feedback
for complex spatial rotations.

**Usage:**

```bash
comchan --plot --auto --braille octahedron
```

### 4. Custom Wireframe (`.wrfm` files)

You are not limited to the built-in shapes! ComChan can dynamically load custom
wireframe models at runtime. Simply pass the path to any valid `.wrfm` file.

**Usage:**

```bash
comchan --plot --auto --braille my_drone.wrfm
```

---

*Want to create your own 3D shapes? Check out the `.wrfm` format! It is a
dead-simple, human-readable text format for defining 3D vertices and the edges
that connect them.*

*If you are building your own Rust tools, you can easily parse and serialize
these models using the companion [`wrfm`](https://crates.io/crates/wrfm) crate!*
