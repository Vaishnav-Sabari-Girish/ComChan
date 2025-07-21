# 🔧 Using a Configuration File

Starting from **ComChan v0.1.9**, you can use a configuration file to streamline your workflow by saving frequently used options. This avoids the need to repeatedly pass flags through the command line every time you run `comchan`.

## 📁 Generate the Default Config File

You can generate a default configuration file by running:

```bash
comchan --generate-config
```

This will create a file at:

```bash
~/.config/comchan/comchan.toml
```

---

## 📝 Example `comchan.toml`

```toml
# ComChan Configuration File
#
# This file contains default settings for ComChan serial monitor.
# Command-line arguments will override these settings.
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

---

## ⚙️ Configuration Fields Explained

| Key              | Type      | Description                                                                                                                                                                                                    |
| ---------------- | --------- | -------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------- |
| `port`           | `string`  | The serial port to connect to. Set to `"auto"` to let ComChan pick the first available serial port automatically. You can also manually specify a port like `"/dev/ttyUSB0"` or `"COM3"` depending on your OS. |
| `baud`           | `integer` | The baud rate used for communication. Common values include `9600`, `115200`, etc. Default is `9600`.                                                                                                          |
| `data_bits`      | `integer` | Number of data bits per frame. Valid values are usually `5`, `6`, `7`, or `8`. Default is `8`.                                                                                                                 |
| `stop_bits`      | `integer` | Number of stop bits. Set to `1` or `2`. Default is `1`.                                                                                                                                                        |
| `parity`         | `string`  | Parity checking mode. Options are `"none"`, `"odd"`, or `"even"`. Default is `"none"`.                                                                                                                         |
| `flow_control`   | `string`  | Flow control method. Valid options: `"none"`, `"software"` (XON/XOFF), or `"hardware"` (RTS/CTS). Default is `"none"`.                                                                                         |
| `timeout_ms`     | `integer` | Timeout in milliseconds for reading from the serial port. Prevents indefinite blocking. Default is `500`.                                                                                                      |
| `reset_delay_ms` | `integer` | Optional delay (in ms) after opening the port. Useful for microcontrollers that reset on port open (e.g., Arduino). Default is `1000`.                                                                         |
| `verbose`        | `boolean` | Enables detailed logging if set to `true`. Helpful for debugging configuration or runtime behavior. Default is `false`.                                                                                        |
| `plot`           | `boolean` | Enables the real-time serial plotter if set to `true`. Default is `false`.                                                                                                                                     |
| `plot_points`    | `integer` | Maximum number of data points shown on the plot at once. Only applicable if `plot = true`. Default is `100`.                                                                                                   |

---


> **NOTE**
>
> The default baud rate is `9600`, but you can customize it in the config file as needed
>
> All values in the config file **can still be overridden** at runtime via command-line arguments like `--port`, `--baud`, `--plot`, etc.


# Outputs 

## Serial monitor (`plot = false`)

![plotfalse](./videos/config_mon.gif)


## Serial Plotter (`plot = true`)

![plottrue](./videos/config_plot.gif)
