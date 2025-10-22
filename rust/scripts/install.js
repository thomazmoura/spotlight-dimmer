#!/usr/bin/env node

const fs = require('fs');
const path = require('path');

// ANSI color codes for console output
const colors = {
  cyan: '\x1b[36m',
  yellow: '\x1b[33m',
  green: '\x1b[32m',
  red: '\x1b[31m',
  reset: '\x1b[0m'
};

function log(message, color = 'reset') {
  console.log(`${colors[color]}${message}${colors.reset}`);
}

function checkPlatform() {
  if (process.platform !== 'win32') {
    log('Error: spotlight-dimmer is currently only supported on Windows.', 'red');
    log('Linux support is planned for future releases.', 'yellow');
    process.exit(1);
  }
}

function verifyBinaries() {
  log('\nInstalling Spotlight Dimmer...', 'cyan');
  log('Verifying pre-built binaries...', 'yellow');

  const binDir = path.join(__dirname, '..', 'bin');
  const executables = ['spotlight-dimmer.exe', 'spotlight-dimmer-config.exe'];
  const icons = ['spotlight-dimmer-icon.ico', 'spotlight-dimmer-icon-paused.ico'];

  // Check executables
  for (const exe of executables) {
    const exePath = path.join(binDir, exe);
    if (!fs.existsSync(exePath)) {
      log(`\nError: ${exe} not found in package!`, 'red');
      log('This package may be corrupted. Please try reinstalling.', 'yellow');
      process.exit(1);
    }
    log(`  ✓ Found ${exe}`, 'green');
  }

  // Check icons
  for (const icon of icons) {
    const iconPath = path.join(binDir, icon);
    if (!fs.existsSync(iconPath)) {
      log(`  Warning: ${icon} not found - system tray icon may not work`, 'yellow');
    } else {
      log(`  ✓ Found ${icon}`, 'green');
    }
  }
}

function createWrappers() {
  log('\nSetting up command wrappers...', 'yellow');

  const binDir = path.join(__dirname, '..', 'bin');

  // Create spotlight-dimmer.cmd
  const dimmerWrapper = `@echo off
"%~dp0spotlight-dimmer.exe" %*
`;
  const dimmerPath = path.join(binDir, 'spotlight-dimmer.cmd');
  fs.writeFileSync(dimmerPath, dimmerWrapper);
  log('  ✓ Created spotlight-dimmer.cmd', 'green');

  // Create spotlight-dimmer-config.cmd
  const configWrapper = `@echo off
"%~dp0spotlight-dimmer-config.exe" %*
`;
  const configPath = path.join(binDir, 'spotlight-dimmer-config.cmd');
  fs.writeFileSync(configPath, configWrapper);
  log('  ✓ Created spotlight-dimmer-config.cmd', 'green');
}

function main() {
  checkPlatform();
  verifyBinaries();
  createWrappers();

  log('\n✓ Installation complete!', 'green');
  log('\nUsage:', 'cyan');
  log('  spotlight-dimmer                 # Start the application', 'yellow');
  log('  spotlight-dimmer-config status   # View configuration', 'yellow');
  log('  spotlight-dimmer-config --help   # See all config options\n', 'yellow');
}

main();
