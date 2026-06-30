use ratatui::style::Color;

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

    pub fn has_data(&self) -> bool {
        !self.data.is_empty()
    }
}

fn strip_log_prefixes(mut line: &str) -> &str {
    line = line.trim();

    if line.starts_with('[') {
        if let Some(end_idx) = line.find(']') {
            let inside = &line[1..end_idx];

            if inside
                .chars()
                .all(|c| c.is_ascii_digit() || c == ':' || c == '.' || c == ',')
            {
                line = line[end_idx + 1..].trim_start();

                if line.starts_with('<')
                    && let Some(lvl_end) = line.find('>')
                {
                    line = line[lvl_end + 1..].trim_start();
                }

                if let Some(colon_idx) = line.find(": ")
                    && !line[..colon_idx].contains(' ')
                {
                    line = line[colon_idx + 2..].trim_start();
                }
            }
        }
    } else {
        let parts: Vec<&str> = line.split_whitespace().collect();
        if parts.len() >= 2 {
            let looks_like_time = parts[0].parse::<f64>().is_ok();

            let clean_level =
                parts[1].trim_matches(|c| c == '[' || c == ']' || c == '<' || c == '>');

            let is_level = matches!(
                clean_level,
                "INFO"
                    | "WARN"
                    | "ERROR"
                    | "DEBUG"
                    | "TRACE"
                    | "info"
                    | "warn"
                    | "error"
                    | "debug"
                    | "trace"
            );

            if looks_like_time
                && is_level
                && let Some(level_idx) = line.find(parts[1])
            {
                line = line[level_idx + parts[1].len()..].trim_start();

                // 2. Strip defmt module paths and separators (`]`, `└─`, `├─`)
                if let Some(bracket_idx) = line.find(']') {
                    if !line[..bracket_idx].trim().contains(' ') {
                        line = line[bracket_idx + 1..].trim_start();
                    }
                } else if let Some(tree_idx) = line.find("└─") {
                    line = line[tree_idx + "└─".len()..].trim_start();
                } else if let Some(tree_idx) = line.find("├─") {
                    line = line[tree_idx + "├─".len()..].trim_start();
                }
            }
        }
    }

    line
}

/// Entry point to parse a raw serial line into zero or more (sensor_name, value) pairs.
pub fn parse_sensor_data(line: &str) -> Vec<(String, f64)> {
    // 1. Strip ANSI escape sequences (Colors) entirely from the line
    let clean_line = strip_ansi(line);
    let mut working_line = clean_line.trim();

    // ── NEW: Strip defmt/RTOS timestamps before any parsing happens ──
    working_line = strip_log_prefixes(working_line);

    // 2. Preprocess and strip metadata if it originates from a Zephyr logger
    working_line = strip_zephyr_headers(working_line);

    // 3. Try parsing as comma-separated multiple items
    if working_line.contains(',')
        && let Some(results) = parse_comma_separated(working_line)
    {
        return results;
    }

    // 3.5 Try parsing space-separated key-value pairs
    if let Some(results) = parse_space_separated_kv(working_line) {
        return results;
    }

    // 4. Try parsing single key-value pairs (Name: Value or Name=Value)
    if let Some(result) = parse_single_key_value(working_line) {
        return vec![result];
    }

    // 5. Try parsing space-separated numbers
    if let Some(results) = parse_space_separated_numbers(working_line) {
        return results;
    }

    // 6. Try parsing a single bare number
    if let Some(result) = parse_bare_number(working_line) {
        return vec![result];
    }

    // 7. Fallback to keyword heuristics
    if let Some(result) = parse_keyword_fallback(working_line) {
        return vec![result];
    }

    Vec::new()
}

/// Safely removes ANSI color codes (like \x1b[0m) so they don't break float parsing
fn strip_ansi(s: &str) -> String {
    let mut clean_str = String::with_capacity(s.len());
    let mut in_escape = false;
    for c in s.chars() {
        if c == '\x1b' {
            in_escape = true;
        } else if in_escape {
            if c.is_ascii_alphabetic() {
                in_escape = false;
            }
        } else {
            clean_str.push(c);
        }
    }
    clean_str
}

/// Preprocessor that strips Zephyr log headers, returning the raw payload slice.
fn strip_zephyr_headers(line: &str) -> &str {
    let is_zephyr_log = line.contains("<inf>")
        || line.contains("<err>")
        || line.contains("<wrn>")
        || line.contains("<dbg>");

    if is_zephyr_log && let Some(gt_pos) = line.find('>') {
        // Just return everything after the '>', keeping the module name and label intact.
        return line[gt_pos + 1..].trim_start();
    }
    line
}

/// Strategy 1: Handles multi-item rows separated by commas (e.g., "Temp: 24, Humid: 60")
fn parse_comma_separated(line: &str) -> Option<Vec<(String, f64)>> {
    let parts: Vec<&str> = line.split(',').collect();
    let mut results = Vec::new();
    let mut found_any = false;

    for (i, part) in parts.iter().enumerate() {
        let part = part.trim();

        // Match "Key : Value" or "Key = Value" patterns within the comma group
        if let Some((name, val)) = parse_kv_split(part, ':').or_else(|| parse_kv_split(part, '=')) {
            results.push((name.to_string(), val));
            found_any = true;
            continue;
        }

        // Match bare numbers inside commas (e.g., "23.5, 71.2")
        if let Ok(val) = part.parse::<f64>() {
            results.push((format!("Channel {}", i), val));
            found_any = true;
        }
    }

    if found_any { Some(results) } else { None }
}

/// Strategy 2: Handles standalone "Name: Value" or "Name=Value" lines without commas
fn parse_single_key_value(line: &str) -> Option<(String, f64)> {
    parse_kv_split(line, ':')
        .or_else(|| parse_kv_split(line, '='))
        .map(|(name, val)| (name.to_string(), val))
}

/// Strategy 3: Handles purely whitespace-separated lists of values (e.g., "12.4 45.2 78.1")
fn parse_space_separated_numbers(line: &str) -> Option<Vec<(String, f64)>> {
    let words: Vec<&str> = line.split_whitespace().collect();
    let numeric: Vec<f64> = words.iter().filter_map(|w| w.parse::<f64>().ok()).collect();

    if numeric.len() > 1 {
        let results = numeric
            .into_iter()
            .enumerate()
            .map(|(i, v)| (format!("Channel {}", i), v))
            .collect();
        Some(results)
    } else {
        None
    }
}

/// Strategy 3.5: Handles space-separated Key:Value pairs (e.g., "ax:1.2 ay:3.4")
fn parse_space_separated_kv(line: &str) -> Option<Vec<(String, f64)>> {
    let parts: Vec<&str> = line.split_whitespace().collect();
    let mut results = Vec::new();

    for part in parts {
        if let Some((name, val)) = parse_kv_split(part, ':').or_else(|| parse_kv_split(part, '=')) {
            results.push((name.to_string(), val));
        }
    }

    // Only return success if we actually found multiple valid key-value pairs
    if results.len() > 1 {
        Some(results)
    } else {
        None
    }
}

/// Strategy 4: Handles single plain value transmissions (e.g., "45.19")
fn parse_bare_number(line: &str) -> Option<(String, f64)> {
    line.parse::<f64>()
        .ok()
        .map(|val| ("Value".to_string(), val))
}

/// Strategy 5: Searches text chunks for substring keyword hints to classify untagged readings
fn parse_keyword_fallback(line: &str) -> Option<(String, f64)> {
    let words: Vec<&str> = line.split_whitespace().collect();
    for word in &words {
        let cleaned = word.trim_matches(|c: char| !c.is_ascii_digit() && c != '.' && c != '-');
        if let Ok(value) = cleaned.parse::<f64>() {
            let ll = line.to_lowercase();
            let sensor_name = if ll.contains("temp") {
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
            return Some((sensor_name.to_string(), value));
        }
    }
    None
}

/// Internal helper to split a single chunk at a character delimiter and safely parse the right flank
fn parse_kv_split(part: &str, delimiter: char) -> Option<(&str, f64)> {
    // FIX 1: Use `rfind` instead of `find` so we split at the *last* colon/equals.
    // This safely handles Zephyr logs like "dht22_logger: Temperature: 25.0"
    let pos = part.rfind(delimiter)?;
    let (name, val_str) = part.split_at(pos);
    let clean_val_str = val_str.get(1..)?.trim();

    // FIX 2: Include 'e' and 'E' to support scientific notation!
    let end_idx = clean_val_str
        .find(|c: char| {
            !c.is_ascii_digit() && c != '.' && c != '-' && c != '+' && c != 'e' && c != 'E'
        })
        .unwrap_or(clean_val_str.len());

    let numeric_str = &clean_val_str[..end_idx];

    if let Ok(val) = numeric_str.parse::<f64>() {
        Some((name.trim(), val))
    } else {
        None
    }
}
