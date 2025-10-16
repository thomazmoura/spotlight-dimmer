use auto_launch::AutoLaunchBuilder;

/// Get the AutoLaunch instance for Spotlight Dimmer
fn get_auto_launch() -> Result<auto_launch::AutoLaunch, String> {
    // Get the current executable path
    let exe_path =
        std::env::current_exe().map_err(|e| format!("Failed to get executable path: {}", e))?;

    // Get the directory containing the executable
    let exe_dir = exe_path
        .parent()
        .ok_or("Failed to get executable directory")?;

    // Construct the path to spotlight-dimmer.exe
    let spotlight_dimmer_path = exe_dir.join("spotlight-dimmer.exe");

    // Verify the executable exists
    if !spotlight_dimmer_path.exists() {
        return Err(format!(
            "spotlight-dimmer.exe not found at: {}",
            spotlight_dimmer_path.display()
        ));
    }

    // Convert path to string
    let exe_path_str = spotlight_dimmer_path
        .to_str()
        .ok_or("Failed to convert path to string")?;

    // Create AutoLaunch instance
    AutoLaunchBuilder::new()
        .set_app_name("Spotlight Dimmer")
        .set_app_path(exe_path_str)
        .build()
        .map_err(|e| format!("Failed to create AutoLaunch instance: {}", e))
}

/// Enable auto-start at login
pub fn enable() -> Result<(), String> {
    let app = get_auto_launch()?;

    app.enable()
        .map_err(|e| format!("Failed to enable auto-start: {}", e))?;

    Ok(())
}

/// Disable auto-start at login
pub fn disable() -> Result<(), String> {
    let app = get_auto_launch()?;

    app.disable()
        .map_err(|e| format!("Failed to disable auto-start: {}", e))?;

    Ok(())
}

/// Check if auto-start is currently enabled
pub fn is_enabled() -> Result<bool, String> {
    let app = get_auto_launch()?;

    app.is_enabled()
        .map_err(|e| format!("Failed to check auto-start status: {}", e))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_get_auto_launch() {
        // This test just ensures the function doesn't panic
        // Actual functionality depends on the executable path
        let result = get_auto_launch();
        // We expect this to potentially fail in test environment
        // but it shouldn't panic
        match result {
            Ok(_) => println!("AutoLaunch instance created successfully"),
            Err(e) => println!("Expected error in test environment: {}", e),
        }
    }
}
