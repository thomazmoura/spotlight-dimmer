#!/usr/bin/env node

const { execSync } = require('child_process');
const fs = require('fs');
const path = require('path');

// ANSI color codes for console output
const colors = {
  cyan: '\x1b[36m',
  yellow: '\x1b[33m',
  green: '\x1b[32m',
  gray: '\x1b[90m',
  reset: '\x1b[0m'
};

function log(message, color = 'reset') {
  console.log(`${colors[color]}${message}${colors.reset}`);
}

function stopRunningInstances() {
  if (process.platform !== 'win32') {
    return;
  }

  log('\nStopping any running instances...', 'yellow');

  try {
    // Try to kill any running spotlight-dimmer processes
    execSync('taskkill /F /IM spotlight-dimmer.exe', { stdio: 'ignore' });
    log('  Stopped running instances', 'green');
  } catch (error) {
    // Process not running, that's fine
    log('  No running instances found', 'gray');
  }
}

function main() {
  log('\nUninstalling Spotlight Dimmer...', 'cyan');

  stopRunningInstances();

  log('\nCleanup complete!', 'green');
  log('Note: Configuration files in %APPDATA%\\spotlight-dimmer\\ were preserved.', 'yellow');
  log('Delete them manually if you want to remove all traces.\n', 'yellow');
}

main();
