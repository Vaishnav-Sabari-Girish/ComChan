# ComChan (Communication Channel)

![Banner](./images/ComChan.png)

<div align="center">

**A Blazingly Fast Serial Monitor for Embedded Systems and Serial
Communication**

</div>

[![Crates info](https://img.shields.io/crates/v/comchan.svg)](https://crates.io/crates/comchan)
[![License: MIT](https://img.shields.io/github/license/Vaishnav-Sabari-Girish/ComChan?color=blue)](LICENSE)
![Rust](https://img.shields.io/github/languages/top/Vaishnav-Sabari-Girish/ComChan?color=orange)

For detailed instructions with images and videos, take a look at `comchan`'s
[wiki](https://github.com/Vaishnav-Sabari-Girish/ComChan/wiki)

## Installation

Choose your preferred installation method:

### From crates.io

> [!NOTE]
> The easiest way to install ComChan is via `cargo install`

```bash
# Install from source (Standard Braille Engine)
cargo install comchan

# Install with Hardware-Accelerated 3D support (Ratty Terminal) and BLE
cargo install comchan --features ratty,ble

```

Verify the installation:

```bash
comchan --version

```

### From AUR

For Arch Linux users, ComChan is available in the AUR (thanks to
[orhun](https://github.com/orhun)!):

```bash
# ComChan with default features (No 3D/BLE)
## Using yay
yay -S comchan
## Using paru 
paru -S comchan

# ComChan with 3D (no BLE) 
## Using yay 
yay -S comchan-ratty
## Using paru
paru -S comchan-ratty

```

### The Binary

```bash
# Using binstall (Has ratty and BLE)
cargo binstall comchan

# AUR 
yay -S comchan-bin
paru -S comchan-bin
```

> [!NOTE]
> The binary is packaged with the `ratty` and `ble` features enabled, which
> means you can install the binary if your machine is unable to compile
> `comchan` with the `ratty` and `ble` features.

### Using `elda`

`comchan` can also be installed using
[`elda`](https://github.com/Mjoyufull/Elda)

```bash
elda i https://github.com/Vaishnav-Sabari-Girish/ComChan
```

> [!NOTE]
> Credits for `elda` go to [**Rikona**](https://github.com/Mjoyufull)

### From source

Build from source for the latest development version:

```bash
# Clone from GitHub
git clone https://github.com/Vaishnav-Sabari-Girish/ComChan.git

# Build and run with all features
cd ComChan
cargo run --release --features ble,ratty -- --version

```

## CLI Usage

```text
Blazingly Fast Minimal Serial Monitor with Plotting

Usage: comchan [OPTIONS]

Options:
      --completions <COMPLETIONS>     Generate Shell completions
  -p, --port <PORT>                   Serial port to connect to (or 'BLE_STREAM' for Bluetooth)
      --ble                           Start ComChan in Bluetooth Low Energy (BLE) stream mode
  -r, --baud <BAUD>                   Baud Rate of the Serial Monitor
  -l, --log <LOG_FILE>                Log data into a file
  -v, --verbose                       Enable verbose output
      --plot                          Launch the serial plotter
  -c, --config <CONFIG_FILE>          Path to config file
  -x, --hex                           Display incoming data in hex dump format
  -h, --help                          Print help
  -V, --version                       Print version
  # ... (and many more)

```

---

## Common Commands

### BLE Streaming (Nordic UART Service)

ComChan supports streaming data wirelessly from BLE-enabled embedded devices
(like the nRF52 series) using the standard Nordic UART Service (NUS).

```bash
# Start ComChan in BLE mode to scan and connect to a peripheral
comchan --ble

```

ComChan will scan for devices, prompt you to select your target, and
automatically subscribe to the NUS TX characteristic. You can seamlessly switch
between the raw monitor and the plotter (`--plot`) using `Ctrl+P` while the BLE
stream is active.

### Basic Serial Monitor

```bash
comchan -p /dev/ttyUSB0 -r 115200

```

### RTT & Defmt Debug Probe Mode

Bypass physical UART serial ports entirely! ComChan can stream zero-latency logs
directly from your microcontroller's memory via SWD.

```bash
comchan --rtt --elf path/to/firmware.elf --chip nRF52840_xxAA

```

### Plotter & 3D Spatial Telemetry

Visualize sensor data in real-time. Use `Tab` or `2` to toggle between the 2D
Line Chart and the 3D Telemetry Dashboard.

```bash
comchan --port /dev/ttyUSB0 --baud 115200 --plot

```

---

## Features

### Current Features ✅

* **BLE Support** - Stream data wirelessly via Nordic UART Service (NUS).
* **Read & Write Serial Data** - Monitor incoming data and send commands.
* **Instant Mode Hot-Swapping** - Seamlessly toggle between Monitor and Plotter
  via `Ctrl+P`.
* **RTT & Defmt Support** - Stream logs via SWD/J-Link directly from memory.
* **Auto-Recovery & Graceful Exit** - Robust handling of connection drops and
  hardware resets.
* **Terminal-Based Serial Plotter** - Visualize sensor values with auto-scaling.
* **3D Spatial Telemetry (IMU)** - Real-time 3D rotation dashboard.
* **Real-Time Session Replay** - Replay previously recorded `.log` or `.csv`
  files.
* **Continuous CSV Streaming** - Stream parsed numeric data to CSV on-the-fly.
* **Hex Dump View** - Inspect raw binary payloads with `--hex` or
  `--hex-pretty`.
* **Export Plot to SVG** - Save visualizations as high-quality SVGs.
* **Hardware Simulation** - Generate mock sensor data for testing without
  hardware.

## Stargazers over time (Graph)

## 🧠 (mostly) Brain made

**This project was NOT vibe-coded BUT AI is still involved in some parts of
it.**

* **Generating test code:** Because it's something I always skip so I would
rather have some AI generated tests than none at all.
* **Micro-improvements:** I have used AI as an advisor to improve some bits of
code here and there. Big refactors or new features are done by my hand though.

<br>

![img](https://brainmade.org/white-logo.svg)

<br>

[⬆ Back to Top](https://www.google.com/search?q=%23comchan-communication-channel)
