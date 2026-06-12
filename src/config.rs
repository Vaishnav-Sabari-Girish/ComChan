use clap::{CommandFactory, Parser, ValueEnum};
use clap_complete::Shell;
use clap_complete_nushell::Nushell;
use inline_colorization::*;
use serde::{Deserialize, Deserializer, Serialize, Serializer};
use std::fs;
use std::path::PathBuf;
use std::str::FromStr;

#[derive(Clone, Copy, Debug, ValueEnum)]
pub enum GenShell {
    Bash,
    Zsh,
    Fish,
    Elvish,
    PowerShell,
    Nu,
}

// Removed ValueEnum, Serialize, and Deserialize derive macros
#[derive(Clone, Debug, PartialEq)]
pub enum BrailleModel {
    Cube,
    Tetrahedron,
    Octahedron,
    Custom(String), // This holds the path to your .wrfm file!
}

// Teach Clap how to parse the string from the CLI
impl FromStr for BrailleModel {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.to_lowercase().as_str() {
            "cube" => Ok(BrailleModel::Cube),
            "tetrahedron" => Ok(BrailleModel::Tetrahedron),
            "octahedron" => Ok(BrailleModel::Octahedron),
            _ => {
                if s.to_lowercase().ends_with(".wrfm") {
                    Ok(BrailleModel::Custom(s.to_string()))
                } else {
                    Err(format!(
                        "Invalid Braille Model '{}', Must be 'cube', 'tetrahedron', 'octahedron', or a valid '.wrfm' file path",
                        s
                    ))
                }
            }
        }
    }
}

// Teach Serde how to read it from config.toml
impl<'de> Deserialize<'de> for BrailleModel {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        let s = String::deserialize(deserializer)?;
        Ok(BrailleModel::from_str(&s).unwrap_or(BrailleModel::Cube))
    }
}

// Teach Serde how to write it to config.toml
impl Serialize for BrailleModel {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        let s = match self {
            BrailleModel::Cube => "cube",
            BrailleModel::Tetrahedron => "tetrahedron",
            BrailleModel::Octahedron => "octahedron",
            BrailleModel::Custom(path) => path.as_str(),
        };
        serializer.serialize_str(s)
    }
}

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
    pub zephyr: Option<bool>,
    pub export_limit: Option<usize>,
    pub plot_title: Option<String>,
    pub simulate: Option<bool>,
    pub csv_file: Option<String>,
    pub replay_file: Option<String>,
    pub hex_mode: Option<bool>,
    pub hex_pretty: Option<bool>,
    pub obj_file: Option<String>,
    pub braille: Option<BrailleModel>,
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
            zephyr: Some(false),
            export_limit: Some(1_000_000), // Defaults to 1 million points per sensor
            plot_title: None,
            simulate: Some(false),
            csv_file: None,
            replay_file: None,
            hex_mode: Some(false),
            hex_pretty: Some(false),
            obj_file: None,
            braille: Some(BrailleModel::Cube),
        }
    }
}

#[derive(Parser)]
#[command(
    name = "comchan",
    version = "0.10.0",
    author = "Vaishnav-Sabari-Girish",
    about = "Blazingly Fast Minimal Serial Monitor with Plotting"
)]
pub struct Args {
    #[arg(long = "completions", value_enum, help = "Generate Shell completions")]
    pub completions: Option<GenShell>,

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

    #[arg(long = "zephyr", action = clap::ArgAction::SetTrue, help = "Enables Zephyr Shell mode")]
    pub zephyr: bool,

    #[arg(
        long = "export-limit",
        help = "Max points to keep in memory for export per sensor"
    )]
    pub export_limit: Option<usize>,

    #[arg(
        long = "plot-title",
        help = "Set the plot title for the exported SVG file"
    )]
    pub plot_title: Option<String>,

    #[arg(long = "simulate", action = clap::ArgAction::SetTrue, help = "Simulate Serial Data with no need for hardware (Use for testing ComChan)")]
    pub simulate: bool,

    #[arg(
        long = "csv",
        help = "Export numeric data to a CSV file while streaming serial data"
    )]
    pub csv_file: Option<String>,

    #[arg(
        long = "replay",
        help = "Replay a previous session from its *.log or *.csv file"
    )]
    pub replay_file: Option<String>,

    #[arg(short = 'x', long = "hex", action = clap::ArgAction::SetTrue, help = "Display incoming serial data in hex dump format")]
    pub hex_mode: Option<bool>,

    #[arg(long = "hex-pretty", action = clap::ArgAction::SetTrue, help = "Display incoming serial data in a clean, buffered hex-dump format")]
    pub hex_pretty: Option<bool>,

    #[arg(long = "obj", help = "Path to .obj file")]
    pub obj_file: Option<String>,

    #[arg(
        long = "braille",
        help = "Select a built-in Braille 3D model (cube, tetrahedron, octahedron) or provide a path to a custom .wrfm file"
    )]
    pub braille: Option<BrailleModel>,

    #[arg(long, default_value_t = false, help = "Exports the plot in Dark Mode")]
    pub dark: bool,

    #[arg(
        long,
        default_value_t = false,
        requires = "elf",
        help = "View RTT logs"
    )]
    pub rtt: bool,

    #[arg(long, requires = "rtt", help = "The Path to the compiled .elf file")]
    pub elf: Option<String>,

    #[arg(long, requires = "rtt", help = "Chip name for probe-rs")]
    pub chip: Option<String>,
}

/// The resolved, merged configuration used at runtime.
#[derive(Clone)]
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
    pub zephyr: bool,
    pub export_limit: usize,
    pub plot_title: String,
    pub simulate: bool,
    pub csv_file: Option<String>,
    pub replay_file: Option<String>,
    pub hex_mode: bool,
    pub hex_pretty: bool,
    pub obj_file: Option<String>,
    pub braille: BrailleModel,
    pub dark_mode: bool,
    pub rtt: bool,
    pub elf: Option<String>,
    pub chip: Option<String>,
}

// Generate completions
pub fn print_completions(shell: GenShell) {
    let mut cmd = Args::command();
    let bin_name = cmd.get_name().to_string();
    let mut stdout = std::io::stdout();

    match shell {
        GenShell::Bash => clap_complete::generate(Shell::Bash, &mut cmd, bin_name, &mut stdout),
        GenShell::Elvish => clap_complete::generate(Shell::Elvish, &mut cmd, bin_name, &mut stdout),
        GenShell::Zsh => clap_complete::generate(Shell::Zsh, &mut cmd, bin_name, &mut stdout),
        GenShell::PowerShell => {
            clap_complete::generate(Shell::PowerShell, &mut cmd, bin_name, &mut stdout)
        }
        GenShell::Fish => clap_complete::generate(Shell::Fish, &mut cmd, bin_name, &mut stdout),
        GenShell::Nu => clap_complete::generate(Nushell, &mut cmd, bin_name, &mut stdout),
    }
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
        zephyr: args.zephyr || config.zephyr.unwrap_or(false),
        export_limit: args
            .export_limit
            .or(config.export_limit)
            .unwrap_or(1_000_000),
        plot_title: args
            .plot_title
            .or(config.plot_title)
            .unwrap_or_else(|| "Sensor Data".to_string()),
        simulate: args.simulate || config.simulate.unwrap_or(false),
        csv_file: args.csv_file.or(config.csv_file),
        replay_file: args.replay_file.or(config.replay_file),
        hex_mode: args.hex_mode.unwrap_or(false) || config.hex_mode.unwrap_or(false),
        hex_pretty: args.hex_pretty.unwrap_or(false) || config.hex_pretty.unwrap_or(false),
        obj_file: args.obj_file.or(config.obj_file),
        braille: args
            .braille
            .or(config.braille)
            .unwrap_or(BrailleModel::Cube),
        dark_mode: args.dark,
        rtt: args.rtt,
        elf: args.elf,
        chip: args.chip,
    }
}
