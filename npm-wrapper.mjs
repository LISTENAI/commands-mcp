#!/usr/bin/env node

import { resolve } from 'path';
import { accessSync, constants } from 'fs';
import { spawnSync } from 'child_process';

const platforms = {
  'darwin-arm64': {
    suffix: '',
  },
  'darwin-x64': {
    suffix: '',
  },
  'linux-x64': {
    suffix: '',
  },
  'win32-x64': {
    suffix: '.exe',
  },
};

const target = `${process.platform}-${process.arch}`;
const config = platforms[target];
if (!config) {
  throw new Error(`Unsupported platform: ${process.platform}-${process.arch}`);
}

const packageDir = decodeURIComponent(new URL('.', import.meta.url).pathname);

const executable = [
  resolve(packageDir, 'binaries', target, `commands-mcp${config.suffix}`),
  resolve(packageDir, 'target', 'release', `commands-mcp${config.suffix}`),
].find((path) => {
  try {
    accessSync(path, constants.X_OK);
    return true;
  } catch {
    return false;
  }
});
if (!executable) {
  throw new Error(`Executable not found for platform: ${process.platform}-${process.arch}`);
}

const args = process.argv.slice(2);

spawnSync(executable, args, {
  stdio: 'inherit',
});
