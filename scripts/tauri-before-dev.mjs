/**
 * Script: tauri-before-dev.mjs
 * Purpose: Ensures the Vite dev server is reachable before Tauri dev startup; starts it if needed.
 * Last modified: 2026-04-12
 */
import net from 'node:net';
import { spawn } from 'node:child_process';

const DEV_URL = process.env.TAURI_DEV_URL ?? 'http://localhost:1420';

function delay(milliseconds) {
  return new Promise((resolve) => setTimeout(resolve, milliseconds));
}

async function isHttpReady(url) {
  const controller = new AbortController();
  const timeout = setTimeout(() => controller.abort(), 1000);

  try {
    const response = await fetch(url, {
      method: 'GET',
      signal: controller.signal,
      headers: {
        Accept: 'text/html'
      }
    });

    return response.ok || response.status < 500;
  } catch {
    return false;
  } finally {
    clearTimeout(timeout);
  }
}

function isPortBusy(hostname, port) {
  return new Promise((resolve) => {
    const socket = net.createConnection({ host: hostname, port });

    socket.once('connect', () => {
      socket.end();
      resolve(true);
    });

    socket.once('error', () => {
      resolve(false);
    });
  });
}

async function waitForHttp(url, attempts = 60) {
  for (let attempt = 0; attempt < attempts; attempt += 1) {
    if (await isHttpReady(url)) {
      return true;
    }

    await delay(500);
  }

  return false;
}

async function main() {
  const url = new URL(DEV_URL);
  const hostname = url.hostname;
  const port = Number(url.port || 80);

  if (await isHttpReady(DEV_URL)) {
    console.log(`[tauri-before-dev] Reusing existing dev server at ${DEV_URL}`);
    return;
  }

  if (await isPortBusy(hostname, port)) {
    console.error(
      `[tauri-before-dev] Port ${port} is already in use, but ${DEV_URL} is not responding as the Vite dev server. Stop the conflicting process or change the configured dev port.`
    );
    process.exit(1);
  }

  const child = spawn('npm', ['run', 'dev'], {
    stdio: 'inherit',
    shell: process.platform === 'win32'
  });

  const ready = await waitForHttp(DEV_URL);
  if (!ready) {
    child.kill('SIGTERM');
    console.error(`[tauri-before-dev] Timed out waiting for ${DEV_URL}`);
    process.exit(1);
  }

  const forwardSignal = (signal) => {
    if (!child.killed) {
      child.kill(signal);
    }
  };

  process.on('SIGINT', () => forwardSignal('SIGINT'));
  process.on('SIGTERM', () => forwardSignal('SIGTERM'));

  await new Promise((resolve, reject) => {
    child.once('exit', (code) => {
      if (code === 0 || code === null) {
        resolve();
        return;
      }

      reject(new Error(`Vite exited with status ${code}`));
    });

    child.once('error', reject);
  });
}

main().catch((error) => {
  console.error(`[tauri-before-dev] ${error instanceof Error ? error.message : String(error)}`);
  process.exit(1);
});