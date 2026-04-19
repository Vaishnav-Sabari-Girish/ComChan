use clap::Parser;
use inline_colorization::*;
use serde::{Deserialize, Serialize};
use std::fs;
use std::path::PathBuf;

#[derive(Debug, Deserialize, Serialize)]
pub struct Config {
    pub port: Option<String>,
    pub baud: Option<u32>,
    pub data_bits: Option<u8>,
    pub stop_bits: Option<u8>,
    pub parity: Option<String>,
    pub flow_control: Option<String>,
    pub timeout_ms: Option<u64>,
    pub reset_delay_ms: Option<u64>,
    pub log_file: Option<String>,
    pub verbose: Option<bool>,
    pub plot: Option<bool>,
    pub plot_points: Option<usize>,
}

impl Default for Config {
    fn default() -> Self {
        Config {
            port: Some("auto".to_string()),
            baud: Some(9600),
            data_bits: Some(8),
            stop_bits: Some(1),
            parity: Some("none".to_string()),
            flow_control: Some("none".to_string()),
            timeout_ms: Some(500),
            reset_delay_ms: Some(1000),
            log_file: None,
            verbose: Some(false),
            plot: Some(false),
            plot_points: Some(100),
        }
    }
}

#[derive(Parser)]
#[command(
    name = "comchan",
    version = "0.3.2",
    author = "Vaishnav-Sabari-Girish",
    about = "Blazingly Fast Minimal Serial Monitor with Plotting"
)]
pub struct Args {
    #[arg(short = 'p', long = "port", help = "Serial port to connect to")]
    pub port: Option<String>,

    #[arg(short = 'r', long = "baud", help = "Baud Rate of the Serial Monitor")]
    pub baud: Option<u32>,

    #[arg(short = 'd', long = "data-bits")]
    pub data_bits: Option<u8>,

    #[arg(short = 's', long = "stop-bits")]
    pub stop_bits: Option<u8>,

    #[arg(long = "parity")]
    pub parity: Option<String>,

    #[arg(long = "flow-control")]
    pub flow_control: Option<String>,

    #[arg(short = 't', long = "timeout")]
    pub timeout_ms: Option<u64>,

    #[arg(long = "reset-delay")]
    pub reset_delay_ms: Option<u64>,

    #[arg(short = 'l', long = "log", help = "Log Serial data into a file")]
    pub log_file: Option<String>,

    #[arg(long = "list-ports", action = clap::ArgAction::SetTrue, help = "List all available ports")]
    pub list_ports: bool,

    #[arg(long = "auto", action = clap::ArgAction::SetTrue, help = "Auto-detect USB serial port")]
    pub auto: Option<bool>,

    #[arg(short = 'v', long = "verbose", action = clap::ArgAction::SetTrue)]
    pub verbose: Option<bool>,

    #[arg(long = "plot", action = clap::ArgAction::SetTrue, help = "Launch the serial plotter")]
    pub plot: bool,

    #[arg(long = "plot-points")]
    pub plot_points: Option<usize>,

    #[arg(long = "config", short = 'c', help = "Path to config file")]
    pub config_file: Option<PathBuf>,

    #[arg(long = "generate-config", action = clap::ArgAction::SetTrue)]
    pub generate_config: bool,
}

/// The resolved, merged configuration used at runtime.
pub struct MergedConfig {
    pub port: Option<String>,
    pub baud: u32,
    pub data_bits: u8,
    pub stop_bits: u8,
    pub parity: String,
    pub flow_control: String,
    pub timeout_ms: u64,
    pub reset_delay_ms: u64,
    pub log_file: Option<String>,
    pub list_ports: bool,
    pub verbose: bool,
    pub plot: bool,
    pub plot_points: usize,
}

// ── Platform helpers ─────────────────────────────────────────────────────────

pub fn get_default_config_path() -> Result<PathBuf, Box<dyn std::error::Error>> {
    let config_dir = if cfg!(target_os = "windows") {
        if let Some(appdata) = std::env::var_os("APPDATA") {
            PathBuf::from(appdata).join("comchan")
        } else {
            return Err("APPDATA environment variable not found".into());
        }
    } else if cfg!(target_os = "macos") {
        dirs::home_dir()
            .ok_or("Could not find home directory")?
            .join("Library")
            .join("Application Support")
            .join("comchan")
    } else {
        dirs::home_dir()
            .ok_or("Could not find home directory")?
            .join(".config")
            .join("comchan")
    };

    Ok(config_dir.join("comchan.toml"))
}

pub fn get_platform_name() -> &'static str {
    if cfg!(target_os = "windows") {
        "Windows"
    } else if cfg!(target_os = "macos") {
        "macOS"
    } else if cfg!(target_os = "linux") {
        "Linux"
    } else {
        "Unix-like"
    }
}

// ── Config file loading ───────────────────────────────────────────────────────

fn find_config_file(specified_path: Option<PathBuf>) -> Option<PathBuf> {
    if let Some(path) = specified_path {
        return path.exists().then_some(path);
    }

    let current = PathBuf::from("comchan.toml");
    if current.exists() {
        return Some(current);
    }

    if let Ok(default_path) = get_default_config_path()
        && default_path.exists()
    {
        return Some(default_path);
    }

    if let Some(home_dir) = dirs::home_dir() {
        let home_config = home_dir.join(".comchan.toml");
        if home_config.exists() {
            return Some(home_config);
        }
    }

    None
}

pub fn load_config(config_path: Option<PathBuf>) -> Result<Config, Box<dyn std::error::Error>> {
    if let Some(path) = find_config_file(config_path) {
        let content = fs::read_to_string(&path)?;
        let config: Config = toml::from_str(&content)
            .map_err(|e| format!("Failed to parse config file {}: {}", path.display(), e))?;
        println!(
            "{color_blue}󰅍 Loaded config from: {}{color_reset}",
            path.display()
        );
        Ok(config)
    } else {
        Ok(Config::default())
    }
}

pub fn generate_default_config(path: Option<PathBuf>) -> Result<(), Box<dyn std::error::Error>> {
    let config_path = if let Some(p) = path {
        p
    } else {
        get_default_config_path()?
    };

    let toml_content = toml::to_string_pretty(&Config::default())?;

    let platform_comment = format!("# Platform: {} config directory", get_platform_name());
    let location_comment = if cfg!(target_os = "windows") {
        "# Default location: %APPDATA%\\comchan\\comchan.toml"
    } else if cfg!(target_os = "macos") {
        "# Default location: ~/Library/Application Support/comchan/comchan.toml"
    } else {
        "# Default location: ~/.config/comchan/comchan.toml"
    };

    let commented_config = format!(
        r#"# ComChan Configuration File
#
{platform_comment}
{location_comment}
#
# Command line arguments override these settings.
# Set port = "auto" to auto-detect the first USB serial port.
# Parity:       "none" | "odd" | "even"
# Flow control: "none" | "software" | "hardware"

{toml_content}
"#
    );

    if let Some(parent) = config_path.parent() {
        fs::create_dir_all(parent)?;
    }
    fs::write(&config_path, commented_config)?;

    println!(
        "{color_green} Generated default config for {}: {}{color_reset}",
        get_platform_name(),
        config_path.display()
    );
    println!("{color_blue}󰌵 Edit the file to customize your default settings{color_reset}");
    Ok(())
}

pub fn merge_config_and_args(config: Config, args: Args) -> MergedConfig {
    MergedConfig {
        port: args.port.or(config.port),
        baud: args.baud.or(config.baud).unwrap_or(9600),
        data_bits: args.data_bits.or(config.data_bits).unwrap_or(8),
        stop_bits: args.stop_bits.or(config.stop_bits).unwrap_or(1),
        parity: args
            .parity
            .or(config.parity)
            .unwrap_or_else(|| "none".into()),
        flow_control: args
            .flow_control
            .or(config.flow_control)
            .unwrap_or_else(|| "none".into()),
        timeout_ms: args.timeout_ms.or(config.timeout_ms).unwrap_or(500),
        reset_delay_ms: args
            .reset_delay_ms
            .or(config.reset_delay_ms)
            .unwrap_or(1000),
        log_file: args.log_file.or(config.log_file),
        list_ports: args.list_ports,
        verbose: args.verbose.or(config.verbose).unwrap_or(false),
        plot: args.plot || config.plot.unwrap_or(false),
        plot_points: args.plot_points.or(config.plot_points).unwrap_or(100),
    }
}
