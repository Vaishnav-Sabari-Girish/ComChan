# Other Features of ComChan

1. [Serial Plotter](#the-serial-plotter)
2. [Automatic Port Detection](#auto-port-detection)

# The Serial Plotter

A Serial Plotter is a powerful visualization tool that transforms numerical data received through serial communication into real-time graphical representations. Originally introduced in Arduino IDE version 1.6.6, this tool has become an essential component for developers, engineers, and makers working with microcontrollers and embedded systems.

## How to use the Serial Plotter in `ComChan`

To use the terminal based Serial Plotter in `ComChan`, you just have to add a `--plot` flag with the other flags as shown below:

```bash
comchan --port /dev/ttyUSB0 --baud 9600 --plot

# OR 

comchan -p /dev/ttyUSB0 -r 9600 --plot
```

Once you do that, you will get this as your output:

![Serial Plotter output](./videos/plotter.gif)

---

## Features of `ComChan` Serial Plotter

* **Real-time plotting**: Instantly visualize incoming data from your microcontroller.
* **Single stream support**: Currently supports plotting one stream of numerical data at a time.
* **Color-coded lines**: The data stream is rendered with a distinct color.
* **Terminal-friendly**: No GUI required. Works completely inside the terminal.

---

## Formatting Data for Plotting

To make your data compatible with the plotter, ensure that your microcontroller prints a single numerical value per line using `Serial.println()` like this:

```cpp
Serial.println(sensor_value);
```

Currently, only one stream is plotted. If multiple values are sent, only the first one will be considered.

---

## Use Cases

* **Sensor Monitoring**: Plot values from temperature, humidity, or light sensors.
* **PID Tuning**: Visualize control loop behavior in real time.
* **Data Debugging**: Spot anomalies in system behavior quickly.

---

# Auto Port Detection 

You can run ComChan with the `--auto` flag like so:

```bash
comchan --auto
```

This automatically detects and connects to the appropriate serial port, so you don’t have to manually specify it using a `--port` flag. 

This is especially useful if:
- You’re not sure which port your board is connected to.
- You’re frequently connecting/disconnecting devices.
- You want a faster, zero-config setup.

Under the hood, ComChan scans all available serial ports and identifies the most likely candidate based on device names and connection responses. 

> **Note:** If needed, you can still specify the port manually like so:
>
> ```bash
> comchan --port /dev/ttyUSB0
> ```

### Demo
Here’s a quick demonstration:

![Auto](./videos/auto.gif)

As shown above, ComChan detects the port, connects, and begins serial communication — all without any extra configuration.

This feature aims to offer both **convenience** and **control** for developers and embedded enthusiasts alike.


## Coming Soon

* Multi-series plotting support
* Zoom & pan capabilities
* Export plots as image files
* Custom axis labeling
* Interactive TUI integration

