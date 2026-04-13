#!/usr/bin/env node
/**
 * Script: generate-updater-json.mjs
 * Purpose: Builds Tauri updater latest.json metadata from local release artifacts.
 * Last modified: 2026-04-12
 */
/**
 * Generate latest.json updater metadata for Diamond Prompt Runner.
 *
 * Reads the .app.tar.gz signature produced by `tauri build --bundles app`
 * and emits a latest.json that Tauri's updater endpoint expects.
 *
 * Usage:
 *   npm run updater:json
 */

import { readFileSync } from "fs";
import { writeFileSync } from "fs";
import { join } from "path";

const BUNDLE_DIR = join("src-tauri", "target", "release", "bundle");
const MACOS_DIR = join(BUNDLE_DIR, "macos");

// Read version from tauri.conf.json
const tauriConf = JSON.parse(
  readFileSync(join("src-tauri", "tauri.conf.json"), "utf-8")
);
const version = tauriConf.version;
const productName = tauriConf.productName; // "Diamond Prompt Runner"

// GitHub asset filenames use dots instead of spaces; the GitHub Actions
// release workflow builds a universal binary, which tauri-action names
// `<ProductName>_universal.app.tar.gz`.
const dotName = productName.replace(/ /g, ".");
const tarball = `${dotName}_universal.app.tar.gz`;
const sigFile = `${productName}.app.tar.gz.sig`;

const sigPath = join(MACOS_DIR, sigFile);
let signature;
try {
  signature = readFileSync(sigPath, "utf-8").trim();
} catch {
  console.error(`ERROR: Signature file not found at ${sigPath}`);
  console.error(
    'Run `TAURI_SIGNING_PRIVATE_KEY="$(cat ~/.tauri/diamond-runner.key)" TAURI_SIGNING_PRIVATE_KEY_PASSWORD="" npm run tauri build -- --bundles app` first.'
  );
  process.exit(1);
}

const ghRepo = "blossomz37/diamond-prompt-runner";
const downloadUrl = `https://github.com/${ghRepo}/releases/download/v${version}/${tarball}`;

const latest = {
  version,
  notes: `Release v${version}`,
  pub_date: new Date().toISOString(),
  platforms: {
    "darwin-aarch64": {
      signature,
      url: downloadUrl,
    },
    "darwin-x86_64": {
      signature,
      url: downloadUrl,
    },
  },
};

const outPath = join(BUNDLE_DIR, "latest.json");
writeFileSync(outPath, JSON.stringify(latest, null, 2) + "\n");
console.log(`Wrote ${outPath}`);
console.log(`  version: ${version}`);
console.log(`  url:     ${downloadUrl}`);
