use serde_json::json;
use std::collections::HashMap;

/// Represents a detected spike in sensor data
#[derive(Debug, Clone)]
pub struct SpikeInfo {
    pub sensor_name: String,
    pub timestamp: f64,
    pub value: f64,
    pub mean: f64,
    pub std_dev: f64,
}

/// Buffers sensor data and detects spikes using Z-score method
pub struct SpikeDetector {
    /// Per-sensor data buffers: sensor_name -> Vec<(timestamp, value)>
    buffers: HashMap<String, Vec<(f64, f64)>>,
    /// Maximum number of recent points to keep per sensor
    max_points: usize,
    /// Z-score threshold for spike detection (default: 2.0)
    z_threshold: f64,
    /// All detected spikes
    pub spikes: Vec<SpikeInfo>,
}

impl SpikeDetector {
    pub fn new(max_points: usize) -> Self {
        SpikeDetector {
            buffers: HashMap::new(),
            max_points,
            z_threshold: 2.0,
            spikes: Vec::new(),
        }
    }

    /// Add a data point for a sensor and check for spikes
    pub fn add_point(&mut self, sensor_name: &str, timestamp: f64, value: f64) {
        let buffer = self
            .buffers
            .entry(sensor_name.to_string())
            .or_insert_with(Vec::new);

        buffer.push((timestamp, value));

        // Only check for spikes once we have enough data points
        if buffer.len() >= 5 {
            let (mean, std_dev) = Self::compute_stats(buffer);

            // Detect spike: value deviates from mean by more than z_threshold * std_dev
            if std_dev > 0.0 && ((value - mean) / std_dev).abs() > self.z_threshold {
                self.spikes.push(SpikeInfo {
                    sensor_name: sensor_name.to_string(),
                    timestamp,
                    value,
                    mean,
                    std_dev,
                });
            }
        }

        // Maintain rolling window
        if buffer.len() > self.max_points {
            buffer.remove(0);
        }
    }

    /// Compute mean and standard deviation of buffered values
    fn compute_stats(buffer: &[(f64, f64)]) -> (f64, f64) {
        let n = buffer.len() as f64;
        let sum: f64 = buffer.iter().map(|(_, v)| v).sum();
        let mean = sum / n;
        let variance: f64 = buffer.iter().map(|(_, v)| (v - mean).powi(2)).sum::<f64>() / n;
        let std_dev = variance.sqrt();
        (mean, std_dev)
    }

    /// Check if any spikes were detected
    pub fn has_spikes(&self) -> bool {
        !self.spikes.is_empty()
    }

    /// Get a summary of all buffered sensor data for the AI prompt
    pub fn get_data_summary(&self) -> String {
        let mut summary = String::new();

        for (sensor_name, buffer) in &self.buffers {
            summary.push_str(&format!("\n--- {} ({} points) ---\n", sensor_name, buffer.len()));

            // Include the last 50 points (or all if fewer)
            let start = if buffer.len() > 50 {
                buffer.len() - 50
            } else {
                0
            };

            for (t, v) in &buffer[start..] {
                summary.push_str(&format!("  t={:.1}: {:.4}\n", t, v));
            }
        }

        summary
    }

    /// Format spike information for the AI prompt
    fn format_spikes_for_prompt(&self) -> String {
        let mut spike_info = String::new();
        for spike in &self.spikes {
            spike_info.push_str(&format!(
                "- Sensor '{}' spiked at t={:.1} with value {:.4} (mean={:.4}, std_dev={:.4}, z-score={:.2})\n",
                spike.sensor_name,
                spike.timestamp,
                spike.value,
                spike.mean,
                spike.std_dev,
                (spike.value - spike.mean) / spike.std_dev
            ));
        }
        spike_info
    }
}

/// Send spike data to the OpenAI API and return a plain-English explanation
pub fn explain_spikes(
    detector: &SpikeDetector,
    api_key: &str,
) -> Result<String, Box<dyn std::error::Error>> {
    let spike_summary = detector.format_spikes_for_prompt();
    let data_summary = detector.get_data_summary();

    let prompt = format!(
        "You are an embedded systems expert analyzing serial sensor data from a microcontroller.\n\
        \n\
        The following sensor data was captured, and spikes (anomalies) were detected:\n\
        \n\
        ## Detected Spikes\n\
        {}\n\
        ## Recent Sensor Data\n\
        {}\n\
        \n\
        Provide a concise, plain-English explanation of what likely caused each spike. \
        Consider common embedded system issues such as:\n\
        - Loose connections or wiring issues\n\
        - Power fluctuations or supply noise\n\
        - Sensor calibration drift\n\
        - Environmental interference (EMI, temperature)\n\
        - Software timing issues or buffer overflows\n\
        - Ground loops or analog reference problems\n\
        \n\
        Format your response as a brief summary (2-4 sentences per spike). \
        Start each explanation with the timestamp.",
        spike_summary, data_summary
    );

    let body = json!({
        "model": "gpt-3.5-turbo",
        "messages": [
            {
                "role": "system",
                "content": "You are a helpful embedded systems diagnostics assistant. \
                            Provide brief, actionable explanations for sensor data anomalies."
            },
            {
                "role": "user",
                "content": prompt
            }
        ],
        "max_tokens": 500,
        "temperature": 0.3
    });

    let client = reqwest::blocking::Client::builder()
        .timeout(std::time::Duration::from_secs(30))
        .build()?;

    let response = client
        .post("https://api.openai.com/v1/chat/completions")
        .header("Authorization", format!("Bearer {}", api_key))
        .header("Content-Type", "application/json")
        .json(&body)
        .send()?;

    if !response.status().is_success() {
        let status = response.status();
        let error_body = response.text().unwrap_or_default();
        return Err(format!("OpenAI API error ({}): {}", status, error_body).into());
    }

    let json_response: serde_json::Value = response.json()?;

    let explanation = json_response["choices"][0]["message"]["content"]
        .as_str()
        .unwrap_or("No explanation received from AI.")
        .to_string();

    Ok(explanation)
}
