#!/usr/bin/env node

import { resolve } from 'path';
import { accessSync, chmodSync, constants } from 'fs';
import { spawn } from 'child_process';

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
    accessSync(path, constants.R_OK);
    return true;
  } catch {
    return false;
  }
});
if (!executable) {
  throw new Error(`Executable not found for platform: ${process.platform}-${process.arch}`);
}

if (process.platform != 'win32') {
  try {
    accessSync(executable, constants.X_OK);
  } catch {
    chmodSync(executable, 0o755);
  }
}

const args = process.argv.slice(2);

const server = spawn(executable, args);
process.stdin.pipe(server.stdin);
server.stdout.pipe(process.stdout);
server.stderr.pipe(process.stderr);
server.once('exit', (code) => {
  process.exit(code);
});
