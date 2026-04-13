/**
 * Illustrative helper template for Diamond Prompt Runner workflows.
 *
 * Do not assume this should be run unchanged on another machine or repo.
 * Map the binary path, project path, pipeline id, payload, and credential
 * loading behavior to the user's actual setup first.
 *
 * Purpose:
 * Execute a single headless pipeline run via the Diamond Runner CLI binary.
 */
import { spawn } from 'child_process';
import fs from 'fs';
import path from 'path';
import { fileURLToPath } from 'url';

const __filename = fileURLToPath(import.meta.url);
const __dirname = path.dirname(__filename);

function loadDotEnv() {
  const envPath = path.resolve(__dirname, '../.env');
  if (!fs.existsSync(envPath)) return;

  const content = fs.readFileSync(envPath, 'utf8');
  for (const line of content.split(/\r?\n/)) {
    const trimmed = line.trim();
    if (!trimmed || trimmed.startsWith('#')) continue;

    const separatorIndex = trimmed.indexOf('=');
    if (separatorIndex === -1) continue;

    const key = trimmed.slice(0, separatorIndex).trim();
    const value = trimmed.slice(separatorIndex + 1).trim();
    if (!key || process.env[key]) continue;
    process.env[key] = value;
  }
}

loadDotEnv();

const binaryPath = process.argv[2] || './src-tauri/target/debug/diamond-runner';
const projectPath = process.argv[3] || 'path/to/project';
const pipelineId = process.argv[4] || 'pipeline-id';
const payload = process.argv[5] || JSON.stringify({ count: '01' });

console.log(`Starting illustrative headless test for pipeline '${pipelineId}' in '${projectPath}'...`);

const child = spawn(binaryPath, ['cli', 'run-pipeline', projectPath, pipelineId, payload], {
  stdio: 'pipe'
});

child.stdout.on('data', (data) => {
  process.stdout.write(`\x1b[36m[Backend]\x1b[0m ${data}`);
});

child.stderr.on('data', (data) => {
  process.stderr.write(`\x1b[31m[Backend Error]\x1b[0m ${data}`);
});

child.on('close', (code) => {
  if (code === 0) {
    console.log('\n\x1b[32mSuccess: Headless pipeline execution completed.\x1b[0m');
  } else {
    console.log(`\n\x1b[31mFailed: Headless pipeline execution exited with code ${code}.\x1b[0m`);
  }
});
