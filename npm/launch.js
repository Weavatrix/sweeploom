'use strict';

const { spawnSync } = require('node:child_process');
const { existsSync } = require('node:fs');
const { join } = require('node:path');

const TARGETS = {
  'win32-x64': 'sweeploom.exe',
  'linux-x64': 'sweeploom',
  'linux-arm64': 'sweeploom',
  'darwin-x64': 'sweeploom',
  'darwin-arm64': 'sweeploom'
};

function bundled() {
  const key = `${process.platform}-${process.arch}`;
  const name = TARGETS[key];
  if (!name) {
    return null;
  }
  const path = join(__dirname, '..', 'vendor', key, name);
  return existsSync(path) ? path : null;
}

function onPath() {
  const result = spawnSync(process.platform === 'win32' ? 'where' : 'which', ['sweeploom'], {
    encoding: 'utf8',
    windowsHide: true
  });
  if (result.status !== 0) {
    return null;
  }
  const line = (result.stdout || '').split(/\r?\n/).find((item) => item.trim());
  return line ? line.trim() : null;
}

function resolveBinary() {
  return bundled() || onPath();
}

function run(extraArgs) {
  const executable = resolveBinary();
  if (!executable) {
    process.stderr.write(
      'sweeploom: no native binary. cargo install sweeploom, or reinstall the npm package after a release build.\n'
    );
    return 1;
  }
  const args = extraArgs.concat(process.argv.slice(2));
  const result = spawnSync(executable, args, {
    stdio: 'inherit',
    windowsHide: true
  });
  if (result.error) {
    process.stderr.write(`sweeploom: ${result.error.message}\n`);
    return 1;
  }
  return result.status === null ? 1 : result.status;
}

module.exports = { run, resolveBinary };
