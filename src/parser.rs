use ratatui::style::Color;
use std::borrow::Cow;

// Color palette for different sensors
pub const COLORS: &[Color] = &[
    Color::Cyan,
    Color::Magenta,
    Color::Yellow,
    Color::Green,
    Color::Red,
    Color::Blue,
    Color::White,
    Color::LightCyan,
    Color::LightMagenta,
    Color::LightYellow,
    Color::LightGreen,
    Color::LightRed,
    Color::LightBlue,
];

pub fn get_color_for_index(index: usize) -> Color {
    COLORS[index % COLORS.len()]
}

/// Holds a rolling window of (x, y) data points for one named sensor stream.
#[derive(Debug, Clone)]
pub struct SensorData {
    pub name: String,
    pub data: Vec<(f64, f64)>,
    pub color: Color,
    pub min_value: f64,
    pub max_value: f64,
    pub current_value: f64,
}

impl SensorData {
    pub fn new(name: String, color: Color) -> Self {
        SensorData {
            name,
            data: Vec::new(),
            color,
            min_value: f64::INFINITY,
            max_value: f64::NEG_INFINITY,
            current_value: 0.0,
        }
    }

    pub fn add_point(&mut self, x: f64, y: f64, max_points: usize) {
        self.data.push((x, y));
        self.current_value = y;

        if y < self.min_value {
            self.min_value = y;
        }
        if y > self.max_value {
            self.max_value = y;
        }

        if self.data.len() > max_points {
            self.data.remove(0);
        }
    }

    /// Returns true if this sensor has at least one data point.
    pub fn has_data(&self) -> bool {
        !self.data.is_empty()
    }
}

/// Parse a raw serial line into zero or more (sensor_name, value) pairs.
///
/// Supported formats (in priority order):
///   1. `SensorName : Value` or `SensorName: Value`
///   2. `SensorName = Value` or `SensorName=Value`
///   3. Comma-separated  `v1,v2,v3`  → Channel 0, Channel 1, …
///   4. Space-separated multiple numbers → Channel 0, Channel 1, …
///   5. Single bare number → "Value"
///   6. Number embedded in text with keyword heuristics (Temperature, Humidity, …)
pub fn parse_sensor_data<'a>(line: &'a str) -> Vec<(Cow<'a, str>, f64)> {
    let mut results = Vec::new();
    let line = line.trim();

    // Pattern 1: "Name : Value"
    if line.contains(':') {
        let parts: Vec<&str> = line.splitn(2, ':').collect();
        if parts.len() == 2 {
            if let Ok(value) = parts[1].trim().parse::<f64>() {
                results.push((Cow::Borrowed(parts[0].trim()), value));
                return results;
            }
        }
    }

    // Pattern 2: "Name = Value"
    if line.contains('=') {
        let parts: Vec<&str> = line.splitn(2, '=').collect();
        if parts.len() == 2 {
            if let Ok(value) = parts[1].trim().parse::<f64>() {
                results.push((Cow::Borrowed(parts[0].trim()), value));
                return results;
            }
        }
    }

    // Pattern 3: Comma-separated values
    if line.contains(',') {
        let values: Vec<&str> = line.split(',').collect();
        let parsed: Vec<f64> = values
            .iter()
            .filter_map(|s| s.trim().parse::<f64>().ok())
            .collect();
        if parsed.len() == values.len() {
            // All tokens were numeric
            for (i, v) in parsed.iter().enumerate() {
                results.push((Cow::Owned(format!("Channel {}", i)), *v));
            }
            return results;
        }
    }

    // Pattern 4: Space-separated multiple numbers
    let words: Vec<&str> = line.split_whitespace().collect();
    let numeric: Vec<f64> = words.iter().filter_map(|w| w.parse::<f64>().ok()).collect();
    if numeric.len() > 1 {
        for (i, v) in numeric.iter().enumerate() {
            results.push((Cow::Owned(format!("Channel {}", i)), *v));
        }
        return results;
    }

    // Pattern 5: Single bare number
    if let Ok(value) = line.parse::<f64>() {
        results.push((Cow::Borrowed("Value"), value));
        return results;
    }

    // Pattern 6: Keyword heuristics
    for word in &words {
        let cleaned = word.trim_matches(|c: char| !c.is_ascii_digit() && c != '.' && c != '-');
        if let Ok(value) = cleaned.parse::<f64>() {
            let ll = line.to_lowercase();
            let sensor_name: &'static str = if ll.contains("temp") {
                "Temperature"
            } else if ll.contains("humid") {
                "Humidity"
            } else if ll.contains("pressure") {
                "Pressure"
            } else if ll.contains("mag") {
                "Magnetometer"
            } else if ll.contains("gyro") {
                "Gyroscope"
            } else if ll.contains("accel") {
                "Accelerometer"
            } else {
                "Sensor"
            };
            results.push((Cow::Borrowed(sensor_name), value));
            break;
        }
    }

    results
}
