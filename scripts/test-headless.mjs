import { spawn } from 'child_process';
import path from 'path';

const projectPath = process.argv[2] || "Sample Projects/Neon & Nightmares";
const pipelineId = process.argv[3] || "batch-production";
const payload = process.argv[4] || JSON.stringify({ chapter: "3" });

console.log(`Starting headless test for pipeline '${pipelineId}' in '${projectPath}'...`);

const child = spawn('./src-tauri/target/debug/diamond-runner', [
  'cli', 'run-pipeline', projectPath, pipelineId, payload
], {
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
    console.log(`\n\x1b[32mSuccess: Headless pipeline execution completed.\x1b[0m`);
  } else {
    console.log(`\n\x1b[31mFailed: Headless pipeline execution exited with code ${code}.\x1b[0m`);
  }
});
