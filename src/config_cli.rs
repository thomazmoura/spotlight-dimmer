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
        "active-enable" => cmd_active_enable(),
        "active-disable" => cmd_active_disable(),
        "active-color" => cmd_active_color(&args[2..]),
        "partial-enable" => cmd_partial_enable(),
        "partial-disable" => cmd_partial_disable(),
        "reset" => cmd_reset(),
        "list-profiles" => cmd_list_profiles(),
        "set-profile" => cmd_set_profile(&args[2..]),
        "save-profile" => cmd_save_profile(&args[2..]),
        "delete-profile" => cmd_delete_profile(&args[2..]),
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
    println!("INACTIVE OVERLAY COMMANDS (dims non-active displays):");
    println!("    enable                      Enable inactive display dimming");
    println!("    disable                     Disable inactive display dimming");
    println!("    color <r> <g> <b> [a]       Set inactive overlay color (RGB 0-255, alpha 0.0-1.0)");
    println!();
    println!("ACTIVE OVERLAY COMMANDS (highlights active display):");
    println!("    active-enable               Enable active display overlay");
    println!("    active-disable              Disable active display overlay");
    println!("    active-color <r> <g> <b> [a] Set active overlay color (RGB 0-255, alpha 0.0-1.0)");
    println!();
    println!("PARTIAL DIMMING COMMANDS (dims empty areas around focused window):");
    println!("    partial-enable              Enable partial dimming on active display");
    println!("    partial-disable             Disable partial dimming on active display");
    println!();
    println!("GENERAL COMMANDS:");
    println!("    status                      Show current configuration");
    println!("    reset                       Reset configuration to defaults");
    println!("    help                        Show this help message");
    println!();
    println!("PROFILE COMMANDS:");
    println!("    list-profiles               List all saved profiles");
    println!("    set-profile <name>          Load and apply a saved profile");
    println!("    save-profile <name>         Save current settings as a profile");
    println!("    delete-profile <name>       Delete a saved profile");
    println!();
    println!("EXAMPLES:");
    println!("    # Dim inactive displays only (traditional behavior)");
    println!("    spotlight-dimmer-config enable");
    println!("    spotlight-dimmer-config color 0 0 0 0.7");
    println!();
    println!("    # Highlight active display only");
    println!("    spotlight-dimmer-config active-enable");
    println!("    spotlight-dimmer-config active-color 50 100 255 0.15");
    println!();
    println!("    # Use partial dimming to highlight focused window");
    println!("    spotlight-dimmer-config enable");
    println!("    spotlight-dimmer-config partial-enable");
    println!();
    println!("    # Combine all three modes");
    println!("    spotlight-dimmer-config enable");
    println!("    spotlight-dimmer-config active-enable");
    println!("    spotlight-dimmer-config partial-enable");
    println!();
    println!("    # Use profiles for quick switching");
    println!("    spotlight-dimmer-config set-profile light-mode");
    println!("    spotlight-dimmer-config set-profile dark-mode");
    println!();
    println!("NOTE: Configuration changes are applied automatically (no restart needed)");
}

fn cmd_status() {
    let config = Config::load();

    println!("Current Configuration:");
    println!();
    println!("INACTIVE OVERLAY (dims non-active displays):");
    println!("  Status: {}", if config.is_dimming_enabled { "Enabled" } else { "Disabled" });
    println!("  Color:");
    println!("    Red:     {}", config.overlay_color.r);
    println!("    Green:   {}", config.overlay_color.g);
    println!("    Blue:    {}", config.overlay_color.b);
    println!("    Alpha:   {:.2} ({}% opacity)", config.overlay_color.a, (config.overlay_color.a * 100.0) as u8);
    println!();

    println!("ACTIVE OVERLAY (highlights active display):");
    println!("  Status: {}", if config.is_active_overlay_enabled { "Enabled" } else { "Disabled" });
    if let Some(active_color) = &config.active_overlay_color {
        println!("  Color:");
        println!("    Red:     {}", active_color.r);
        println!("    Green:   {}", active_color.g);
        println!("    Blue:    {}", active_color.b);
        println!("    Alpha:   {:.2} ({}% opacity)", active_color.a, (active_color.a * 100.0) as u8);
    } else {
        println!("  Color:   Not configured");
    }
    println!();

    println!("PARTIAL DIMMING (dims empty areas around focused window):");
    println!("  Status: {}", if config.is_partial_dimming_enabled { "Enabled" } else { "Disabled" });
    println!("  Note:   Uses inactive overlay color for partial overlays");
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
            println!("✓ Inactive overlay (dimming) enabled");
            println!("  Changes will be applied automatically within 2 seconds");
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
            println!("✓ Inactive overlay (dimming) disabled");
            println!("  Changes will be applied automatically within 2 seconds");
        }
        Err(e) => {
            eprintln!("✗ Failed to save configuration: {}", e);
            std::process::exit(1);
        }
    }
}

fn cmd_active_enable() {
    let mut config = Config::load();
    config.is_active_overlay_enabled = true;

    // If no active color is set, use a default subtle blue highlight
    if config.active_overlay_color.is_none() {
        config.active_overlay_color = Some(OverlayColor {
            r: 50,
            g: 100,
            b: 255,
            a: 0.15,
        });
        println!("  Using default active overlay color: RGB(50, 100, 255) Alpha 0.15");
    }

    match config.save() {
        Ok(_) => {
            println!("✓ Active overlay (highlighting) enabled");
            println!("  Changes will be applied automatically within 2 seconds");
        }
        Err(e) => {
            eprintln!("✗ Failed to save configuration: {}", e);
            std::process::exit(1);
        }
    }
}

fn cmd_active_disable() {
    let mut config = Config::load();
    config.is_active_overlay_enabled = false;

    match config.save() {
        Ok(_) => {
            println!("✓ Active overlay (highlighting) disabled");
            println!("  Changes will be applied automatically within 2 seconds");
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

        if !(0.0..=1.0).contains(&value) {
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
            println!("✓ Inactive overlay color updated:");
            println!("  RGB: ({}, {}, {})", r, g, b);
            println!("  Alpha: {:.2} ({}% opacity)", a, (a * 100.0) as u8);
            println!("  Changes will be applied automatically within 2 seconds");
        }
        Err(e) => {
            eprintln!("✗ Failed to save configuration: {}", e);
            std::process::exit(1);
        }
    }
}

fn cmd_active_color(args: &[String]) {
    if args.len() < 3 {
        eprintln!("Error: active-color command requires at least 3 arguments (r g b)");
        eprintln!("Usage: spotlight-dimmer-config active-color <r> <g> <b> [a]");
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

        if !(0.0..=1.0).contains(&value) {
            eprintln!("Error: alpha value must be between 0.0 and 1.0");
            std::process::exit(1);
        }

        value
    } else {
        0.15 // Default alpha for active overlay (subtle)
    };

    let mut config = Config::load();
    config.active_overlay_color = Some(OverlayColor { r, g, b, a });

    match config.save() {
        Ok(_) => {
            println!("✓ Active overlay color updated:");
            println!("  RGB: ({}, {}, {})", r, g, b);
            println!("  Alpha: {:.2} ({}% opacity)", a, (a * 100.0) as u8);
            println!("  Changes will be applied automatically within 2 seconds");
        }
        Err(e) => {
            eprintln!("✗ Failed to save configuration: {}", e);
            std::process::exit(1);
        }
    }
}

fn cmd_partial_enable() {
    let mut config = Config::load();
    config.is_partial_dimming_enabled = true;

    match config.save() {
        Ok(_) => {
            println!("✓ Partial dimming enabled");
            println!("  Empty areas around the focused window will be dimmed on the active display");
            println!("  Uses the inactive overlay color (RGB({}, {}, {}) Alpha {:.2})",
                config.overlay_color.r,
                config.overlay_color.g,
                config.overlay_color.b,
                config.overlay_color.a
            );
            println!("  Changes will be applied automatically within 2 seconds");
        }
        Err(e) => {
            eprintln!("✗ Failed to save configuration: {}", e);
            std::process::exit(1);
        }
    }
}

fn cmd_partial_disable() {
    let mut config = Config::load();
    config.is_partial_dimming_enabled = false;

    match config.save() {
        Ok(_) => {
            println!("✓ Partial dimming disabled");
            println!("  Changes will be applied automatically within 2 seconds");
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
            println!();
            println!("  Inactive Overlay:");
            println!("    Status: {}", if config.is_dimming_enabled { "Enabled" } else { "Disabled" });
            println!("    Color: RGB({}, {}, {}) Alpha {:.2}",
                config.overlay_color.r,
                config.overlay_color.g,
                config.overlay_color.b,
                config.overlay_color.a
            );
            println!();
            println!("  Active Overlay:");
            println!("    Status: {}", if config.is_active_overlay_enabled { "Enabled" } else { "Disabled" });
            if let Some(active_color) = &config.active_overlay_color {
                println!("    Color: RGB({}, {}, {}) Alpha {:.2}",
                    active_color.r,
                    active_color.g,
                    active_color.b,
                    active_color.a
                );
            } else {
                println!("    Color: Not configured");
            }
            println!();
            println!("  Partial Dimming:");
            println!("    Status: {}", if config.is_partial_dimming_enabled { "Enabled" } else { "Disabled" });
            println!();
            println!("  Changes will be applied automatically within 2 seconds");
        }
        Err(e) => {
            eprintln!("✗ Failed to save configuration: {}", e);
            std::process::exit(1);
        }
    }
}

fn cmd_list_profiles() {
    let config = Config::load();
    let profiles = config.list_profiles();

    if profiles.is_empty() {
        println!("No profiles saved.");
        return;
    }

    println!("Saved Profiles:");
    println!();
    for name in profiles {
        if let Some(profile) = config.get_profile(&name) {
            println!("  {}:", name);
            println!("    Inactive Overlay: {}", if profile.is_dimming_enabled { "Enabled" } else { "Disabled" });
            println!("    Inactive Color: RGB({}, {}, {}) Alpha {:.2}",
                profile.overlay_color.r,
                profile.overlay_color.g,
                profile.overlay_color.b,
                profile.overlay_color.a
            );
            println!("    Active Overlay: {}", if profile.is_active_overlay_enabled { "Enabled" } else { "Disabled" });
            if let Some(active_color) = &profile.active_overlay_color {
                println!("    Active Color: RGB({}, {}, {}) Alpha {:.2}",
                    active_color.r,
                    active_color.g,
                    active_color.b,
                    active_color.a
                );
            }
            println!("    Partial Dimming: {}", if profile.is_partial_dimming_enabled { "Enabled" } else { "Disabled" });
            println!();
        }
    }
}

fn cmd_set_profile(args: &[String]) {
    if args.is_empty() {
        eprintln!("Error: set-profile command requires a profile name");
        eprintln!("Usage: spotlight-dimmer-config set-profile <name>");
        eprintln!("       Use 'list-profiles' to see available profiles");
        std::process::exit(1);
    }

    let profile_name = &args[0];
    let mut config = Config::load();

    match config.load_profile(profile_name) {
        Ok(_) => {
            match config.save() {
                Ok(_) => {
                    println!("✓ Profile '{}' applied successfully", profile_name);
                    println!();
                    println!("  Inactive Overlay: {}", if config.is_dimming_enabled { "Enabled" } else { "Disabled" });
                    println!("  Inactive Color: RGB({}, {}, {}) Alpha {:.2}",
                        config.overlay_color.r,
                        config.overlay_color.g,
                        config.overlay_color.b,
                        config.overlay_color.a
                    );
                    println!("  Active Overlay: {}", if config.is_active_overlay_enabled { "Enabled" } else { "Disabled" });
                    if let Some(active_color) = &config.active_overlay_color {
                        println!("  Active Color: RGB({}, {}, {}) Alpha {:.2}",
                            active_color.r,
                            active_color.g,
                            active_color.b,
                            active_color.a
                        );
                    }
                    println!("  Partial Dimming: {}", if config.is_partial_dimming_enabled { "Enabled" } else { "Disabled" });
                    println!();
                    println!("  Changes will be applied automatically within 2 seconds");
                }
                Err(e) => {
                    eprintln!("✗ Failed to save configuration: {}", e);
                    std::process::exit(1);
                }
            }
        }
        Err(e) => {
            eprintln!("✗ {}", e);
            eprintln!("  Use 'list-profiles' to see available profiles");
            std::process::exit(1);
        }
    }
}

fn cmd_save_profile(args: &[String]) {
    if args.is_empty() {
        eprintln!("Error: save-profile command requires a profile name");
        eprintln!("Usage: spotlight-dimmer-config save-profile <name>");
        std::process::exit(1);
    }

    let profile_name = args[0].clone();
    let mut config = Config::load();

    config.save_profile(profile_name.clone());

    match config.save() {
        Ok(_) => {
            println!("✓ Current settings saved as profile '{}'", profile_name);
            println!("  Use 'set-profile {}' to restore these settings", profile_name);
        }
        Err(e) => {
            eprintln!("✗ Failed to save profile: {}", e);
            std::process::exit(1);
        }
    }
}

fn cmd_delete_profile(args: &[String]) {
    if args.is_empty() {
        eprintln!("Error: delete-profile command requires a profile name");
        eprintln!("Usage: spotlight-dimmer-config delete-profile <name>");
        std::process::exit(1);
    }

    let profile_name = &args[0];
    let mut config = Config::load();

    match config.delete_profile(profile_name) {
        Ok(_) => {
            match config.save() {
                Ok(_) => {
                    println!("✓ Profile '{}' deleted successfully", profile_name);
                }
                Err(e) => {
                    eprintln!("✗ Failed to save configuration: {}", e);
                    std::process::exit(1);
                }
            }
        }
        Err(e) => {
            eprintln!("✗ {}", e);
            std::process::exit(1);
        }
    }
}