mod config;
mod overlay;
mod platform;

use config::{Config, OverlayColor};
use std::env;

fn main() {
    let args: Vec<String> = env::args().collect();

    if args.len() < 2 {
        print_usage();
        return;
    }

    match args[1].as_str() {
        "status" => cmd_status(),
        "enable" => cmd_enable(),
        "disable" => cmd_disable(),
        "color" => cmd_color(&args[2..]),
        "reset" => cmd_reset(),
        "help" | "--help" | "-h" => print_usage(),
        _ => {
            eprintln!("Unknown command: {}", args[1]);
            print_usage();
            std::process::exit(1);
        }
    }
}

fn print_usage() {
    println!("Spotlight Dimmer Configuration Tool");
    println!();
    println!("USAGE:");
    println!("    spotlight-dimmer-config <COMMAND> [OPTIONS]");
    println!();
    println!("COMMANDS:");
    println!("    status              Show current configuration");
    println!("    enable              Enable dimming (auto-starts on next run)");
    println!("    disable             Disable dimming");
    println!("    color <r> <g> <b> [a]  Set overlay color (RGB 0-255, alpha 0.0-1.0)");
    println!("    reset               Reset configuration to defaults");
    println!("    help                Show this help message");
    println!();
    println!("EXAMPLES:");
    println!("    spotlight-dimmer-config status");
    println!("    spotlight-dimmer-config enable");
    println!("    spotlight-dimmer-config color 0 0 0 0.7    # 70% black overlay");
    println!("    spotlight-dimmer-config color 50 50 50 0.3 # 30% gray overlay");
    println!();
    println!("NOTE: Configuration changes require restarting spotlight-dimmer.exe");
}

fn cmd_status() {
    let config = Config::load();

    println!("Current Configuration:");
    println!("  Status: {}", if config.is_dimming_enabled { "Enabled" } else { "Disabled" });
    println!("  Overlay Color:");
    println!("    Red:     {}", config.overlay_color.r);
    println!("    Green:   {}", config.overlay_color.g);
    println!("    Blue:    {}", config.overlay_color.b);
    println!("    Alpha:   {:.2} ({}% opacity)", config.overlay_color.a, (config.overlay_color.a * 100.0) as u8);
    println!();

    if let Ok(path) = Config::config_path() {
        println!("Config file: {:?}", path);
    }
}

fn cmd_enable() {
    let mut config = Config::load();
    config.is_dimming_enabled = true;

    match config.save() {
        Ok(_) => {
            println!("✓ Dimming enabled");
            println!("  Restart spotlight-dimmer.exe for changes to take effect");
        }
        Err(e) => {
            eprintln!("✗ Failed to save configuration: {}", e);
            std::process::exit(1);
        }
    }
}

fn cmd_disable() {
    let mut config = Config::load();
    config.is_dimming_enabled = false;

    match config.save() {
        Ok(_) => {
            println!("✓ Dimming disabled");
            println!("  Restart spotlight-dimmer.exe for changes to take effect");
        }
        Err(e) => {
            eprintln!("✗ Failed to save configuration: {}", e);
            std::process::exit(1);
        }
    }
}

fn cmd_color(args: &[String]) {
    if args.len() < 3 {
        eprintln!("Error: color command requires at least 3 arguments (r g b)");
        eprintln!("Usage: spotlight-dimmer-config color <r> <g> <b> [a]");
        std::process::exit(1);
    }

    let r = args[0].parse::<u8>().unwrap_or_else(|_| {
        eprintln!("Error: red value must be 0-255");
        std::process::exit(1);
    });

    let g = args[1].parse::<u8>().unwrap_or_else(|_| {
        eprintln!("Error: green value must be 0-255");
        std::process::exit(1);
    });

    let b = args[2].parse::<u8>().unwrap_or_else(|_| {
        eprintln!("Error: blue value must be 0-255");
        std::process::exit(1);
    });

    let a = if args.len() > 3 {
        let value = args[3].parse::<f32>().unwrap_or_else(|_| {
            eprintln!("Error: alpha value must be 0.0-1.0");
            std::process::exit(1);
        });

        if value < 0.0 || value > 1.0 {
            eprintln!("Error: alpha value must be between 0.0 and 1.0");
            std::process::exit(1);
        }

        value
    } else {
        0.5 // Default alpha
    };

    let mut config = Config::load();
    config.overlay_color = OverlayColor { r, g, b, a };

    match config.save() {
        Ok(_) => {
            println!("✓ Overlay color updated:");
            println!("  RGB: ({}, {}, {})", r, g, b);
            println!("  Alpha: {:.2} ({}% opacity)", a, (a * 100.0) as u8);
            println!("  Restart spotlight-dimmer.exe for changes to take effect");
        }
        Err(e) => {
            eprintln!("✗ Failed to save configuration: {}", e);
            std::process::exit(1);
        }
    }
}

fn cmd_reset() {
    let config = Config::default();

    match config.save() {
        Ok(_) => {
            println!("✓ Configuration reset to defaults:");
            println!("  Status: {}", if config.is_dimming_enabled { "Enabled" } else { "Disabled" });
            println!("  Overlay Color: RGB({}, {}, {}) Alpha {:.2}",
                config.overlay_color.r,
                config.overlay_color.g,
                config.overlay_color.b,
                config.overlay_color.a
            );
            println!("  Restart spotlight-dimmer.exe for changes to take effect");
        }
        Err(e) => {
            eprintln!("✗ Failed to save configuration: {}", e);
            std::process::exit(1);
        }
    }
}