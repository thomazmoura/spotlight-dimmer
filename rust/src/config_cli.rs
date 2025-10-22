mod autostart;
mod config;
mod overlay;
mod platform;
mod tmux_overlay;
mod tmux_watcher;
mod windows_terminal;

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
        "tmux-enable" => cmd_tmux_enable(),
        "tmux-disable" => cmd_tmux_disable(),
        "tmux-config" => cmd_tmux_config(&args[2..]),
        "tmux-auto-config" => cmd_tmux_auto_config(&args[2..]),
        "tmux-status" => cmd_tmux_status(),
        "tmux-title-pattern" => cmd_tmux_title_pattern(&args[2..]),
        "reset" => cmd_reset(),
        "list-profiles" => cmd_list_profiles(),
        "set-profile" => cmd_set_profile(&args[2..]),
        "save-profile" => cmd_save_profile(&args[2..]),
        "delete-profile" => cmd_delete_profile(&args[2..]),
        "autostart-enable" => cmd_autostart_enable(),
        "autostart-disable" => cmd_autostart_disable(),
        "autostart-status" => cmd_autostart_status(),
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
    println!(
        "    color <r> <g> <b> [a]       Set inactive overlay color (RGB 0-255, alpha 0.0-1.0)"
    );
    println!();
    println!("ACTIVE OVERLAY COMMANDS (highlights active display):");
    println!("    active-enable               Enable active display overlay");
    println!("    active-disable              Disable active display overlay");
    println!(
        "    active-color <r> <g> <b> [a] Set active overlay color (RGB 0-255, alpha 0.0-1.0)"
    );
    println!();
    println!("PARTIAL DIMMING COMMANDS (dims empty areas around focused window):");
    println!("    partial-enable              Enable partial dimming on active display");
    println!("    partial-disable             Disable partial dimming on active display");
    println!();
    println!("TMUX INTEGRATION COMMANDS (dims inactive tmux panes in Windows Terminal):");
    println!("    tmux-enable                 Enable tmux pane focusing mode");
    println!("    tmux-disable                Disable tmux pane focusing mode");
    println!(
        "    tmux-config <fw> <fh> <pl> <pt> Set terminal geometry (font width/height, padding left/top)"
    );
    println!("    tmux-auto-config [profile] [--dry-run]");
    println!(
        "                                Automatically detect geometry from Windows Terminal settings"
    );
    println!(
        "                                Optional: profile name (default: uses defaults section)"
    );
    println!("    tmux-status                 Show tmux configuration");
    println!("    tmux-title-pattern <pattern> Set window title pattern for tmux detection");
    println!();
    println!("GENERAL COMMANDS:");
    println!("    status                      Show current configuration");
    println!("    reset                       Reset configuration to defaults");
    println!("    help                        Show this help message");
    println!();
    println!("AUTO-START COMMANDS:");
    println!("    autostart-enable            Enable auto-start at Windows login");
    println!("    autostart-disable           Disable auto-start at Windows login");
    println!("    autostart-status            Check if auto-start is enabled");
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
    println!(
        "  Status: {}",
        if config.is_dimming_enabled {
            "Enabled"
        } else {
            "Disabled"
        }
    );
    println!("  Color:");
    println!("    Red:     {}", config.overlay_color.r);
    println!("    Green:   {}", config.overlay_color.g);
    println!("    Blue:    {}", config.overlay_color.b);
    println!(
        "    Alpha:   {:.2} ({}% opacity)",
        config.overlay_color.a,
        (config.overlay_color.a * 100.0) as u8
    );
    println!();

    println!("ACTIVE OVERLAY (highlights active display):");
    println!(
        "  Status: {}",
        if config.is_active_overlay_enabled {
            "Enabled"
        } else {
            "Disabled"
        }
    );
    if let Some(active_color) = &config.active_overlay_color {
        println!("  Color:");
        println!("    Red:     {}", active_color.r);
        println!("    Green:   {}", active_color.g);
        println!("    Blue:    {}", active_color.b);
        println!(
            "    Alpha:   {:.2} ({}% opacity)",
            active_color.a,
            (active_color.a * 100.0) as u8
        );
    } else {
        println!("  Color:   Not configured");
    }
    println!();

    println!("PARTIAL DIMMING (dims empty areas around focused window):");
    println!(
        "  Status: {}",
        if config.is_partial_dimming_enabled {
            "Enabled"
        } else {
            "Disabled"
        }
    );
    println!("  Note:   Uses inactive overlay color for partial overlays");
    println!();

    println!("TMUX PANE FOCUSING (dims inactive tmux panes in Windows Terminal):");
    println!(
        "  Status: {}",
        if config.is_tmux_mode_enabled {
            "Enabled"
        } else {
            "Disabled"
        }
    );
    println!("  Terminal Geometry:");
    println!(
        "    Font: {}x{} pixels",
        config.terminal_font_width, config.terminal_font_height
    );
    println!(
        "    Padding: left={}, top={}",
        config.terminal_padding_left, config.terminal_padding_top
    );
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
            println!(
                "  Empty areas around the focused window will be dimmed on the active display"
            );
            println!(
                "  Uses the inactive overlay color (RGB({}, {}, {}) Alpha {:.2})",
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
            println!(
                "    Status: {}",
                if config.is_dimming_enabled {
                    "Enabled"
                } else {
                    "Disabled"
                }
            );
            println!(
                "    Color: RGB({}, {}, {}) Alpha {:.2}",
                config.overlay_color.r,
                config.overlay_color.g,
                config.overlay_color.b,
                config.overlay_color.a
            );
            println!();
            println!("  Active Overlay:");
            println!(
                "    Status: {}",
                if config.is_active_overlay_enabled {
                    "Enabled"
                } else {
                    "Disabled"
                }
            );
            if let Some(active_color) = &config.active_overlay_color {
                println!(
                    "    Color: RGB({}, {}, {}) Alpha {:.2}",
                    active_color.r, active_color.g, active_color.b, active_color.a
                );
            } else {
                println!("    Color: Not configured");
            }
            println!();
            println!("  Partial Dimming:");
            println!(
                "    Status: {}",
                if config.is_partial_dimming_enabled {
                    "Enabled"
                } else {
                    "Disabled"
                }
            );
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
            println!(
                "    Inactive Overlay: {}",
                if profile.is_dimming_enabled {
                    "Enabled"
                } else {
                    "Disabled"
                }
            );
            println!(
                "    Inactive Color: RGB({}, {}, {}) Alpha {:.2}",
                profile.overlay_color.r,
                profile.overlay_color.g,
                profile.overlay_color.b,
                profile.overlay_color.a
            );
            println!(
                "    Active Overlay: {}",
                if profile.is_active_overlay_enabled {
                    "Enabled"
                } else {
                    "Disabled"
                }
            );
            if let Some(active_color) = &profile.active_overlay_color {
                println!(
                    "    Active Color: RGB({}, {}, {}) Alpha {:.2}",
                    active_color.r, active_color.g, active_color.b, active_color.a
                );
            }
            println!(
                "    Partial Dimming: {}",
                if profile.is_partial_dimming_enabled {
                    "Enabled"
                } else {
                    "Disabled"
                }
            );
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
        Ok(_) => match config.save() {
            Ok(_) => {
                println!("✓ Profile '{}' applied successfully", profile_name);
                println!();
                println!(
                    "  Inactive Overlay: {}",
                    if config.is_dimming_enabled {
                        "Enabled"
                    } else {
                        "Disabled"
                    }
                );
                println!(
                    "  Inactive Color: RGB({}, {}, {}) Alpha {:.2}",
                    config.overlay_color.r,
                    config.overlay_color.g,
                    config.overlay_color.b,
                    config.overlay_color.a
                );
                println!(
                    "  Active Overlay: {}",
                    if config.is_active_overlay_enabled {
                        "Enabled"
                    } else {
                        "Disabled"
                    }
                );
                if let Some(active_color) = &config.active_overlay_color {
                    println!(
                        "  Active Color: RGB({}, {}, {}) Alpha {:.2}",
                        active_color.r, active_color.g, active_color.b, active_color.a
                    );
                }
                println!(
                    "  Partial Dimming: {}",
                    if config.is_partial_dimming_enabled {
                        "Enabled"
                    } else {
                        "Disabled"
                    }
                );
                println!();
                println!("  Changes will be applied automatically within 2 seconds");
            }
            Err(e) => {
                eprintln!("✗ Failed to save configuration: {}", e);
                std::process::exit(1);
            }
        },
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
            println!(
                "  Use 'set-profile {}' to restore these settings",
                profile_name
            );
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
        Ok(_) => match config.save() {
            Ok(_) => {
                println!("✓ Profile '{}' deleted successfully", profile_name);
            }
            Err(e) => {
                eprintln!("✗ Failed to save configuration: {}", e);
                std::process::exit(1);
            }
        },
        Err(e) => {
            eprintln!("✗ {}", e);
            std::process::exit(1);
        }
    }
}

fn cmd_autostart_enable() {
    match autostart::enable() {
        Ok(_) => {
            println!("✓ Auto-start at login enabled");
            println!("  Spotlight Dimmer will now start automatically when you log in to Windows");
            println!("  You can disable this in Windows Settings → Apps → Startup");
        }
        Err(e) => {
            eprintln!("✗ Failed to enable auto-start: {}", e);
            std::process::exit(1);
        }
    }
}

fn cmd_autostart_disable() {
    match autostart::disable() {
        Ok(_) => {
            println!("✓ Auto-start at login disabled");
            println!("  Spotlight Dimmer will no longer start automatically when you log in");
        }
        Err(e) => {
            eprintln!("✗ Failed to disable auto-start: {}", e);
            std::process::exit(1);
        }
    }
}

fn cmd_autostart_status() {
    match autostart::is_enabled() {
        Ok(enabled) => {
            if enabled {
                println!("✓ Auto-start is ENABLED");
                println!("  Spotlight Dimmer will start automatically when you log in to Windows");
                println!("  To disable, run: spotlight-dimmer-config autostart-disable");
                println!("  Or toggle in: Windows Settings → Apps → Startup");
            } else {
                println!("○ Auto-start is DISABLED");
                println!("  Spotlight Dimmer will NOT start automatically when you log in");
                println!("  To enable, run: spotlight-dimmer-config autostart-enable");
            }
        }
        Err(e) => {
            eprintln!("✗ Failed to check auto-start status: {}", e);
            std::process::exit(1);
        }
    }
}

fn cmd_tmux_enable() {
    let mut cfg = Config::load();
    cfg.is_tmux_mode_enabled = true;

    match cfg.save() {
        Ok(_) => {
            println!("✓ Tmux pane focusing ENABLED");
            println!("  Inactive tmux panes will be dimmed when Windows Terminal is focused");
            println!();
            println!("  Setup required:");
            println!("    1. Add this to your ~/.tmux.conf:");
            println!(
                "       set-hook -g pane-focus-in 'run-shell \"tmux display -p \\\"#{{pane_left}},#{{pane_top}},#{{pane_right}},#{{pane_bottom}},#{{window_width}},#{{window_height}}\\\" > ~/.spotlight-dimmer/tmux-active-pane.txt\"'"
            );
            println!("    2. Reload tmux config: tmux source-file ~/.tmux.conf");
            println!("    3. Configure terminal geometry with: tmux-config command");
        }
        Err(e) => {
            eprintln!("✗ Failed to save configuration: {}", e);
            std::process::exit(1);
        }
    }
}

fn cmd_tmux_disable() {
    let mut cfg = Config::load();
    cfg.is_tmux_mode_enabled = false;

    match cfg.save() {
        Ok(_) => {
            println!("✓ Tmux pane focusing DISABLED");
            println!("  Inactive tmux panes will NOT be dimmed");
        }
        Err(e) => {
            eprintln!("✗ Failed to save configuration: {}", e);
            std::process::exit(1);
        }
    }
}

fn cmd_tmux_config(args: &[String]) {
    if args.len() != 4 {
        eprintln!("✗ Invalid arguments");
        eprintln!();
        eprintln!("USAGE:");
        eprintln!("    spotlight-dimmer-config tmux-config <font_width> <font_height> <padding_left> <padding_top>");
        eprintln!();
        eprintln!("PARAMETERS:");
        eprintln!("    font_width    Character width in pixels (e.g., 9)");
        eprintln!("    font_height   Character height in pixels (e.g., 20)");
        eprintln!("    padding_left  Left padding in pixels (e.g., 0)");
        eprintln!("    padding_top   Top padding including title bar in pixels (e.g., 35)");
        eprintln!();
        eprintln!("EXAMPLE:");
        eprintln!("    # For a terminal with 9x20 font and 35px title bar:");
        eprintln!("    spotlight-dimmer-config tmux-config 9 20 0 35");
        std::process::exit(1);
    }

    let font_width: u32 = match args[0].parse() {
        Ok(v) => v,
        Err(_) => {
            eprintln!("✗ Invalid font width: must be a positive integer");
            std::process::exit(1);
        }
    };

    let font_height: u32 = match args[1].parse() {
        Ok(v) => v,
        Err(_) => {
            eprintln!("✗ Invalid font height: must be a positive integer");
            std::process::exit(1);
        }
    };

    let padding_left: i32 = match args[2].parse() {
        Ok(v) => v,
        Err(_) => {
            eprintln!("✗ Invalid padding left: must be an integer");
            std::process::exit(1);
        }
    };

    let padding_top: i32 = match args[3].parse() {
        Ok(v) => v,
        Err(_) => {
            eprintln!("✗ Invalid padding top: must be an integer");
            std::process::exit(1);
        }
    };

    let mut cfg = Config::load();
    cfg.terminal_font_width = font_width;
    cfg.terminal_font_height = font_height;
    cfg.terminal_padding_left = padding_left;
    cfg.terminal_padding_top = padding_top;

    match cfg.save() {
        Ok(_) => {
            println!("✓ Terminal geometry configured");
            println!("  Font: {}x{} pixels", font_width, font_height);
            println!("  Padding: left={}, top={}", padding_left, padding_top);
        }
        Err(e) => {
            eprintln!("✗ Failed to save configuration: {}", e);
            std::process::exit(1);
        }
    }
}

fn cmd_tmux_status() {
    let cfg = Config::load();

    println!("TMUX PANE FOCUSING:");
    if cfg.is_tmux_mode_enabled {
        println!("  Status: ENABLED");
    } else {
        println!("  Status: DISABLED");
    }
    println!();

    println!("TERMINAL GEOMETRY:");
    println!(
        "  Font: {}x{} pixels",
        cfg.terminal_font_width, cfg.terminal_font_height
    );
    println!(
        "  Padding: left={}, top={}",
        cfg.terminal_padding_left, cfg.terminal_padding_top
    );
    println!();

    println!("WINDOW TITLE DETECTION:");
    println!("  Pattern: \"{}\"", cfg.tmux_title_pattern);
    println!("  (Overlays only show when Windows Terminal title contains this pattern)");
    println!();

    println!("TMUX PANE FILE:");
    match cfg.get_tmux_pane_file_path() {
        Ok(path) => println!("  Path: {}", path.display()),
        Err(e) => println!("  Path: Error - {}", e),
    }
}

fn cmd_tmux_auto_config(args: &[String]) {
    // Parse arguments
    let mut profile_name: Option<&str> = None;
    let mut dry_run = false;

    for arg in args {
        if arg == "--dry-run" {
            dry_run = true;
        } else if profile_name.is_none() {
            profile_name = Some(arg.as_str());
        } else {
            eprintln!("✗ Too many arguments");
            eprintln!();
            eprintln!("USAGE:");
            eprintln!("    spotlight-dimmer-config tmux-auto-config [profile] [--dry-run]");
            eprintln!();
            eprintln!("EXAMPLES:");
            eprintln!("    # Auto-detect from defaults section");
            eprintln!("    spotlight-dimmer-config tmux-auto-config");
            eprintln!();
            eprintln!("    # Auto-detect from specific profile");
            eprintln!("    spotlight-dimmer-config tmux-auto-config \"Ubuntu-22.04\"");
            eprintln!();
            eprintln!("    # Preview without saving");
            eprintln!("    spotlight-dimmer-config tmux-auto-config --dry-run");
            std::process::exit(1);
        }
    }

    println!("🔍 Auto-detecting Windows Terminal configuration...");
    println!();

    // Parse Windows Terminal settings
    let settings = match windows_terminal::parse_settings(profile_name) {
        Ok(s) => s,
        Err(e) => {
            eprintln!("✗ Failed to parse Windows Terminal settings: {}", e);
            eprintln!();
            eprintln!("TROUBLESHOOTING:");
            eprintln!("  1. Make sure Windows Terminal is installed");
            eprintln!("  2. Check that settings.json exists in:");
            eprintln!("     %LOCALAPPDATA%\\Packages\\Microsoft.WindowsTerminal_8wekyb3d8bbwe\\LocalState\\");
            if let Some(profile) = profile_name {
                eprintln!("  3. Verify profile '{}' exists in your settings", profile);
            }
            std::process::exit(1);
        }
    };

    println!("📖 Found settings:");
    if let Some(profile) = profile_name {
        println!("  Source: Profile '{}'", profile);
    } else {
        println!("  Source: Defaults section");
    }
    println!(
        "  Font: {} at {} pt",
        settings.font_face, settings.font_size_pt
    );
    println!(
        "  Padding: left={}, top={}, right={}, bottom={}",
        settings.padding_left,
        settings.padding_top,
        settings.padding_right,
        settings.padding_bottom
    );
    println!();

    // Calculate font metrics
    println!("📐 Calculating font metrics...");
    let metrics = match windows_terminal::calculate_font_metrics(&settings) {
        Ok(m) => m,
        Err(e) => {
            eprintln!("✗ Failed to calculate font metrics: {}", e);
            eprintln!();
            eprintln!("TROUBLESHOOTING:");
            eprintln!(
                "  1. Make sure the font '{}' is installed on your system",
                settings.font_face
            );
            eprintln!("  2. Try running as administrator if font access is restricted");
            std::process::exit(1);
        }
    };

    println!("  Character width: {} pixels", metrics.width_px);
    println!("  Character height: {} pixels", metrics.height_px);
    println!();

    // Show current configuration for comparison
    let current_cfg = Config::load();
    println!("📋 Current configuration:");
    println!(
        "  Font: {}x{} pixels",
        current_cfg.terminal_font_width, current_cfg.terminal_font_height
    );
    println!(
        "  Padding: left={}, top={}",
        current_cfg.terminal_padding_left, current_cfg.terminal_padding_top
    );
    println!();

    // Show what will be applied
    println!("✨ New configuration:");
    println!("  Font: {}x{} pixels", metrics.width_px, metrics.height_px);
    println!(
        "  Padding: left={}, top={}",
        settings.padding_left, settings.padding_top
    );
    println!();

    // Check if anything changed
    let font_changed = current_cfg.terminal_font_width != metrics.width_px
        || current_cfg.terminal_font_height != metrics.height_px;
    let padding_changed = current_cfg.terminal_padding_left != settings.padding_left
        || current_cfg.terminal_padding_top != settings.padding_top;

    if !font_changed && !padding_changed {
        println!("ℹ️  Configuration already matches Windows Terminal settings");
        println!("   No changes needed");
        return;
    }

    if dry_run {
        println!("🔍 DRY RUN MODE - Configuration NOT saved");
        println!("   Run without --dry-run to apply these settings");
        return;
    }

    // Apply configuration
    let mut cfg = Config::load();
    cfg.terminal_font_width = metrics.width_px;
    cfg.terminal_font_height = metrics.height_px;
    cfg.terminal_padding_left = settings.padding_left;
    cfg.terminal_padding_top = settings.padding_top;

    match cfg.save() {
        Ok(_) => {
            println!("✅ Terminal geometry configured successfully!");
            println!();
            if font_changed {
                println!(
                    "   Font size: {}x{} → {}x{} pixels",
                    current_cfg.terminal_font_width,
                    current_cfg.terminal_font_height,
                    metrics.width_px,
                    metrics.height_px
                );
            }
            if padding_changed {
                println!(
                    "   Padding: left={}, top={} → left={}, top={}",
                    current_cfg.terminal_padding_left,
                    current_cfg.terminal_padding_top,
                    settings.padding_left,
                    settings.padding_top
                );
            }
            println!();
            println!("💡 Next steps:");
            println!("   1. Make sure tmux mode is enabled: spotlight-dimmer-config tmux-enable");
            println!("   2. Configure tmux hook in ~/.tmux.conf (see tmux-enable output)");
            println!("   3. Reload tmux: tmux source-file ~/.tmux.conf");
        }
        Err(e) => {
            eprintln!("✗ Failed to save configuration: {}", e);
            std::process::exit(1);
        }
    }
}

fn cmd_tmux_title_pattern(args: &[String]) {
    if args.is_empty() {
        eprintln!("Usage: spotlight-dimmer-config tmux-title-pattern <pattern>");
        eprintln!();
        eprintln!("Examples:");
        eprintln!("  spotlight-dimmer-config tmux-title-pattern tmux");
        eprintln!("  spotlight-dimmer-config tmux-title-pattern \"my-session\"");
        eprintln!("  spotlight-dimmer-config tmux-title-pattern \"tmux:\"");
        eprintln!();
        eprintln!("💡 Configure tmux to include a detectable pattern in the title:");
        eprintln!("   Add to ~/.tmux.conf:");
        eprintln!("     set-option -g set-titles on");
        eprintln!("     set-option -g set-titles-string \"tmux:#S/#W\"");
        std::process::exit(1);
    }

    let pattern = args.join(" ");

    let mut cfg = Config::load();
    let old_pattern = cfg.tmux_title_pattern.clone();
    cfg.tmux_title_pattern = pattern.clone();

    match cfg.save() {
        Ok(_) => {
            println!("✅ TMUX title pattern updated!");
            println!();
            println!("   Old pattern: \"{}\"", old_pattern);
            println!("   New pattern: \"{}\"", pattern);
            println!();
            println!("💡 The tmux overlays will now only appear when the Windows Terminal");
            println!("   title contains: \"{}\"", pattern);
            println!();
            println!("   Make sure your tmux is configured to set terminal titles:");
            println!("     set-option -g set-titles on");
            println!("     set-option -g set-titles-string \"tmux:#S/#W\"");
        }
        Err(e) => {
            eprintln!("✗ Failed to save configuration: {}", e);
            std::process::exit(1);
        }
    }
}
