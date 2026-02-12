![Banner](./docs/src/images/ComChan.png)

<div align="center">

# ComChan (Communication Channel)

**A Blazingly Fast Serial Monitor for Embedded Systems and Serial Communication**

[![Release](https://img.shields.io/badge/Release-V0.2.4-blue?style=for-the-badge&labelColor=gray)](https://github.com/Vaishnav-Sabari-Girish/ComChan/releases/tag/v0.2.4)

[Features](#features) • [Installation](#installation) • [Documentation](https://vaishnav.world/ComChan) • [Examples](#examples) • [Contributing](CONTRIBUTING.md)

</div>

---

<!-- START doctoc generated TOC please keep comment here to allow auto update -->
<!-- DON'T EDIT THIS SECTION, INSTEAD RE-RUN doctoc TO UPDATE -->
**Table of Contents**

- [ComChan (Communication Channel)](#comchan-communication-channel)
  - [Installation](#installation)
    - [From crates.io](#from-cratesio)
    - [From AUR](#from-aur)
    - [Using Homebrew](#using-homebrew)
    - [From source](#from-source)
  - [Documentation](#documentation)
  - [Contributing](#contributing)
  - [Common Commands](#common-commands)
    - [Basic Serial Monitor](#basic-serial-monitor)
    - [Verbose Mode](#verbose-mode)
    - [Log Mode](#log-mode)
    - [Serial Plotter](#serial-plotter)
    - [Automatically detect serial ports](#automatically-detect-serial-ports)
    - [Use a Configuration file](#use-a-configuration-file)
  - [Features](#features)
    - [Legends](#legends)
- [Examples](#examples)
  - ["Hello World" Program](#hello-world-program)
  - [User Input](#user-input)
  - [Serial Plotter](#serial-plotter-1)
  - [Auto Serial Port Detector](#auto-serial-port-detector)
  - [Using the Configuration file](#using-the-configuration-file)
    - [Serial Monitor (`plot = false`)](#serial-monitor-plot--false)
    - [Serial Plotter (`plot = true`)](#serial-plotter-plot--true)
    - [Serial Plotter Multiple sensor values](#serial-plotter-multiple-sensor-values)
  - [ComChan in Windows](#comchan-in-windows)
- [Feedback Form](#feedback-form)
- [OSCG Guidelines](#oscg-guidelines)
  - [Contributor Guidelines](#contributor-guidelines)
  - [Mentor Guidelines](#mentor-guidelines)
- [Stargazers over time (Graph)](#stargazers-over-time-graph)


<!-- END doctoc generated TOC please keep comment here to allow auto update -->

---

## Installation

Choose your preferred installation method:

### From crates.io

> [!NOTE]
> The easiest way to install ComChan is via `cargo install`

```bash
# Install from source
cargo install comchan

# Install the binary directly (faster)
cargo binstall comchan
```

Verify the installation:

```bash
comchan --version
```

### From AUR

For Arch Linux users, ComChan is available in the AUR (thanks to [orhun](https://github.com/orhun)!):

```bash
# Using yay
yay -S comchan

# Using paru
paru -S comchan
```

### Using Homebrew

ComChan can be installed via Homebrew taps:

```bash
brew install Vaishnav-Sabari-Girish/taps/comchan
```

### From source

Build from source for the latest development version:

```bash
# Clone from GitHub
git clone git@github.com:Vaishnav-Sabari-Girish/ComChan.git

# Or clone from Codeberg
git clone ssh://git@codeberg.org/Vaishnav-Sabari-Girish/ComChan.git

# Build and run
cd ComChan
cargo build --release
cargo run
```

---

## Documentation

📚 The full documentation for ComChan can be found at **[vaishnav.world/ComChan](https://vaishnav.world/ComChan)**

---

## Common Commands

### Basic Serial Monitor

Monitor serial output from your device:

```bash
comchan -p <port> -r <baud_rate>
# OR
comchan --port <port> --baud <baud_rate>
```

**Example:**
```bash
comchan -p /dev/ttyUSB0 -r 9600
```

### Verbose Mode

Get detailed information about the serial connection:

```bash
comchan -p <port> -r <baud_rate> -v
# OR
comchan --port <port> --baud <baud_rate> --verbose
```

### Log Mode

Save serial output to a log file:

```bash
comchan -p <port> -r <baud_rate> -l <log_file_name>
# OR
comchan --port <port> --baud <baud_rate> --log <log_file_name>
```

📄 [View example log file](./test.log)

### Serial Plotter

Visualize sensor data in real-time:

```bash
comchan --port <port> --baud <baud_rate> --plot
# OR
comchan -p <port> -r <baud_rate> --plot
```

### Automatically Detect Serial Ports

Let ComChan find your serial device automatically:

```bash
# With default baud rate (9600)
comchan --auto

# With custom baud rate
comchan --auto --baud <baud_rate>
# OR
comchan --auto -r <baud_rate>
```

**Example:**
```bash
comchan --auto --baud 115200
```

### Use a Configuration File

Starting from version 0.1.9, you can use a configuration file instead of command-line flags:

```bash
# Generate default configuration file
comchan --generate-config
```

This creates a config file at `~/.config/comchan/comchan.toml`

**Example Configuration:**

```toml
# ComChan Configuration File
#
# This file contains default settings for comchan serial monitor.
# Command line arguments will override these settings.
#
# To use auto-detection, set port = "auto"
# Available parity options: "none", "odd", "even"
# Available flow control options: "none", "software", "hardware"

port = "auto"
baud = 9600
data_bits = 8
stop_bits = 1
parity = "none"
flow_control = "none"
timeout_ms = 500
reset_delay_ms = 1000
verbose = false
plot = false
plot_points = 100
```

> [!NOTE]
> The default baud rate is `9600`. You can customize it in the config file or override it with command-line flags (`--auto`, `--port`/`-p`, `--baud`/`-r`, `--plot`).

---

## Features

### Current Features ✅

- **Read Serial Data** - Monitor incoming serial data from any serial port
- **Write to Serial Port** - Send data to your serial device
- **Basic Logging** - Save serial output to log files
- **Auto-Detect Serial Ports** - Automatically find connected serial devices
- **Configuration Files** - Use `.toml` files instead of command-line flags
- **Terminal-Based Serial Plotter** - Visualize data in real-time with the `--plot` flag
- **Multiple Sensor Plotting** - Plot multiple sensor values simultaneously with legends

### Planned Features 🚧

- **Export Serial Data** - Write serial data to files (`.txt`, `.csv`, and more)

### Legends

- ✅ Implemented Features
- 🚧 Yet to be implemented

---

## Examples

### "Hello World" Program

Basic serial monitoring in action:

![Hello World Demo](./docs/src/videos/basic_serial_mon.gif)

📝 [View Arduino code](./code_tests/test_comchan_arduino_uno/test_comchan_arduino_uno.ino)

---

### User Input

Interactive serial communication:

![User Input Demo](./docs/src/videos/basic_user_input.gif)

📝 [View Arduino code](./code_tests/test_user_input/test_user_input.ino)

---

### Serial Plotter

Real-time data visualization:

![Serial Plotter Demo](./docs/src/videos/plotter.gif)

📝 [View Arduino code](./code_tests/random_sensor_vals/random_sensor_vals.ino)

---

### Auto Serial Port Detector

Automatic port detection in action:

![Auto-detect Demo](./docs/src/videos/auto.gif)

---

### Using the Configuration File

#### Serial Monitor (`plot = false`)

![Config Monitor Demo](./docs/src/videos/config_mon.gif)

#### Serial Plotter (`plot = true`)

![Config Plotter Demo](./docs/src/videos/config_plot.gif)

#### Serial Plotter Multiple Sensor Values

Plot multiple sensors simultaneously with automatic legends:

![Multiple Sensor Plot](./docs/src/videos/multiple_sensor_plot.gif)

📝 [View Arduino code](./code_tests/random_sensor_vals_multiple/random_sensor_vals_multiple.ino)

---

### Full Working Demo

Complete workflow demonstration:

![Full Demo](./docs/src/videos/full.gif)

---

### ComChan in Windows

As of Version 0.2.2, ComChan works perfectly on Windows with no limitations!

[![ComChan Windows Demo](https://img.youtube.com/vi/23sSd4_bcxM/0.jpg)](https://www.youtube.com/watch?v=23sSd4_bcxM)

**Windows Installation:**

1. Download the `.exe` file from the [releases page](https://github.com/Vaishnav-Sabari-Girish/ComChan/releases)
2. Open Command Prompt or PowerShell
3. Navigate to the download location:
   ```powershell
   cd Downloads
   ```
4. Run ComChan:
   ```powershell
   comchan.exe --help
   ```

---

## Contributing

We welcome contributions to ComChan! 🎉

Please see our **[CONTRIBUTING.md](CONTRIBUTING.md)** for guidelines on how to contribute.

---

## Feedback Form

We value your feedback! ComChan uses Bashforms for terminal-based feedback submission.

To submit feedback, run this in your terminal:

```bash
ssh -t bashform.me f comchan
```

---

## OSCG Guidelines

### Contributor Guidelines

📖 [View Contributor Guidelines](./OSCG_CONTRIBUTOR_Guidelines.md)

### Mentor Guidelines

📖 [View Mentor Guidelines](./OSCG_MENTOR_Guidelines.md)

---

## Community

### Stargazers over time (Graph)

[![Stargazers over time](https://starchart.cc/Vaishnav-Sabari-Girish/ComChan.svg?variant=adaptive)](https://starchart.cc/Vaishnav-Sabari-Girish/ComChan)


---

<div align="center">

**Made with ❤️ by the ComChan Community**

[⬆ Back to Top](#comchan-communication-channel)

</div>
