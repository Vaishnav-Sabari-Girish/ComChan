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

    // ── Pattern 0: Zephyr Log format (Specific) ──
    // Look specifically for the log signature to avoid "greedy" colon matching
    let is_zephyr_log = line.contains("<inf>") || line.contains("<err>") || line.contains("<wrn>");
    if is_zephyr_log && let Some(pos) = line.find("> ") {
        let working_line = &line[pos + 2..];
        // Only split at the LAST colon if it's followed by a number
        if let Some(colon_pos) = working_line.rfind(':') {
            let (label, val_part) = working_line.split_at(colon_pos);
            let val_str = val_part[1..].trim();
            let numeric_part = val_str.split_whitespace().next().unwrap_or(val_str);

            if let Ok(val) = numeric_part.parse::<f64>() {
                // Extract the actual sensor label by taking the part after the last internal colon
                let clean_label = label.split(':').next_back().unwrap_or(label).trim();
                results.push((Cow::Owned(clean_label.to_string()), val));
                return results;
            }
        }
    }

    // ── Pattern 1: Comma-separated multiple items ──
    // Handles: "Mag: 45, Gyro: 12" OR "Mag=45, Gyro=12" OR "45, 12"
    if line.contains(',') {
        let parts: Vec<&str> = line.split(',').collect();
        let mut found_any = false;

        for (i, part) in parts.iter().enumerate() {
            let part = part.trim();

            // Sub-pattern A: "Key : Value"
            if let Some(pos) = part.find(':') {
                let (name, val_str) = part.split_at(pos);
                if let Ok(val) = val_str[1..].trim().parse::<f64>() {
                    results.push((Cow::Owned(name.trim().to_string()), val));
                    found_any = true;
                    continue;
                }
            }

            // Sub-pattern B: "Key = Value"
            if let Some(pos) = part.find('=') {
                let (name, val_str) = part.split_at(pos);
                if let Ok(val) = val_str[1..].trim().parse::<f64>() {
                    results.push((Cow::Owned(name.trim().to_string()), val));
                    found_any = true;
                    continue;
                }
            }

            // Sub-pattern C: Bare number in comma list
            if let Ok(val) = part.parse::<f64>() {
                results.push((Cow::Owned(format!("Channel {}", i)), val));
                found_any = true;
            }
        }

        if found_any {
            return results;
        }
    }

    // ── Pattern 2: Single "Name : Value" (No commas) ──
    if let Some(pos) = line.find(':') {
        let (name, val_str) = line.split_at(pos);
        if let Ok(val) = val_str[1..].trim().parse::<f64>() {
            results.push((Cow::Borrowed(name.trim()), val));
            return results;
        }
    }

    // ── Pattern 3: Single "Name = Value" (No commas) ──
    if let Some(pos) = line.find('=') {
        let (name, val_str) = line.split_at(pos);
        if let Ok(val) = val_str[1..].trim().parse::<f64>() {
            results.push((Cow::Borrowed(name.trim()), val));
            return results;
        }
    }

    // ── Pattern 4: Space-separated multiple numbers ──
    let words: Vec<&str> = line.split_whitespace().collect();
    let numeric: Vec<f64> = words.iter().filter_map(|w| w.parse::<f64>().ok()).collect();
    if numeric.len() > 1 {
        for (i, v) in numeric.iter().enumerate() {
            results.push((Cow::Owned(format!("Channel {}", i)), *v));
        }
        return results;
    }

    // ── Pattern 5: Single bare number ──
    if let Ok(value) = line.parse::<f64>() {
        results.push((Cow::Borrowed("Value"), value));
        return results;
    }

    // ── Pattern 6: Keyword heuristics fallback ──
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
            break; // Grab the first heuristic match and exit
        }
    }

    results
}
