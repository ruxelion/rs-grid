#!/usr/bin/env node
// Pre-build hook for Trunk: installs npm deps if needed, then generates
// generated/tailwind.css. Works on Windows, macOS and Linux.
import { execSync } from 'node:child_process';
import { existsSync } from 'node:fs';

const run = (cmd) => execSync(cmd, { stdio: 'inherit' });

if (!existsSync('node_modules')) {
  console.log('[build-css] node_modules absent — running npm install…');
  run('npm install');
}

run('npm run css');
