'use strict';

const { existsSync, mkdirSync, copyFileSync } = require('node:fs');
const { join } = require('node:path');

const key = `${process.platform}-${process.arch}`;
const name = process.platform === 'win32' ? 'sweeploom.exe' : 'sweeploom';
const target = process.env.CARGO_TARGET_DIR || join(__dirname, '..', 'target');
const from = join(target, 'release', name);
const destDir = join(__dirname, '..', 'vendor', key);
const dest = join(destDir, name);

if (!existsSync(from)) {
  process.stderr.write(
    `sweeploom prepack: ${from} missing — npm pack will rely on PATH / cargo install.\n`
  );
  process.exit(0);
}
mkdirSync(destDir, { recursive: true });
copyFileSync(from, dest);
process.stderr.write(`sweeploom prepack: copied ${from} -> ${dest}\n`);
