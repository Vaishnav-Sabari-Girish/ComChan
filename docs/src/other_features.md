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



## Added Multiple Sensor Support for Serial plotter 

As of Version 0.2.0, you can now plot the values of multiple sensors at once.


---

## Features of `ComChan` Serial Plotter

* **Real-time plotting**: Instantly visualize incoming data from your microcontroller.
* **Color-coded lines**: The data stream is rendered with a distinct color.
* **Terminal-friendly**: No GUI required. Works completely inside the terminal.
* **Multiple Sensor Plotting** : As of Version 0.2.0, you can now plot the values of multiple sensors at once.

---

## Formatting Data for Plotting

To make your data compatible with the plotter, ensure you label the data to be printed (For the legends), in the format given below:

```cpp
Serial.print("Label 1 : ");
Serial.println(sensor_value);

Serial.print("Label 2 : ");
Serial.println(sensor_value_2);
```

This makes sure that the data being taken is of 2 different sensors.

In the below output, the example that was taken is for 3 sensors (Magnetometer, Gyroscope and Accelerometer).


![multiple_plot](./videos/multiple_sensor_plot.gif)

Below if the code file for this 

```cpp 
void setup() {
  Serial.begin(9600);
}

void loop() {
  Serial.print("Magnetometer : ");
  Serial.println(random(100));
  Serial.print("Gyroscope : ");
  Serial.println(random(100));
  Serial.print("Accelerometer : ");
  Serial.println(random(100));
  delay(1000);
}
```

The above code generates 3 sets of random numbers for the plotter to plot.

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

* Zoom & pan capabilities
* Export plots as image files
* Custom axis labeling
* Interactive TUI integration

