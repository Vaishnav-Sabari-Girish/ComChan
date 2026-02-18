# Exporting Plots to SVG

ComChan allows you to capture high-quality snapshots of your real-time data plots as SVG images. This feature is invaluable for documentation, reports, and sharing data visualizations without needing external tools.

> **Note**: This feature requires ComChan version **0.2.7** or later.

## How to Export

1.  **Start the Plotter**:
    Launch ComChan in plot mode using either auto-detection or manual port specification:
    ```bash
    # Auto-detect port and start plotting
    comchan --auto --plot

    # Manual port specification
    comchan -p /dev/ttyUSB0 -r 9600 --plot
    ```

2.  **Capture Snapshot**:
    While the plot is running and displaying data, simply press the **`s`** key on your keyboard.

    *   The plot will continue running uninterrupted.
    *   A snapshot of the *current view* (including all visible data points) will be saved.

3.  **Locate the File**:
    The SVG file will be saved in your current working directory with a timestamped filename:
    ```
    comchan_plot_YYYYMMDD_HHMMSS.svg
    ```
    Example: `comchan_plot_20231024_143000.svg`

## Features of the Exported SVG

The generated SVG file is a vector graphic, meaning it can be scaled infinitely without losing quality. It includes:

*   **Plot Title**: Includes the timestamp of the capture.
*   **Axes**: labeled X (Sample) and Y (Value) axes with dynamic scaling.
*   **Data Lines**: All active sensor data lines are plotted with distinct colors.
*   **Legend**: A legend identifying each sensor stream.
*   **Grid**: A light grid background for easier reading of values.

## Example

Here is an example of what an exported plot looks like:

![Example Export](./images/example_export_preview.png)
*(Note: Actual export is an interactive SVG file)*

## Troubleshooting

*   **Nothing happens when I press 's'**: Ensure the terminal window has focus. Also, check if you have write permissions in the directory where you launched `comchan`.
*   **"No data to plot" error**: You must have received at least one data point from your device before you can export a plot.
