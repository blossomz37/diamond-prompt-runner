# Quickstart — Run Diamond Prompt Runner from Source

This guide gets you from zero to a running Diamond Prompt Runner
desktop window in about 10 minutes. Use this if you want to try the
app from source without downloading an installer, or if you're
evaluating it before installing.

**Not trying to contribute code?** Most users should just download the
installer from the
[latest release](https://github.com/blossomz37/diamond-prompt-runner/releases/latest).It's faster and you don't need any toolchain setup. On the
release page, download the `.dmg` file from the **Assets** section.
The two "Source code" entries are repository snapshots that GitHub
attaches automatically to every release and are not what you want.

**Planning to contribute?** Read [CONTRIBUTING.md](CONTRIBUTING.md)
instead. It covers platform-specific build dependencies and
development conventions in more detail.

## What you need

- **Node.js 20 LTS** — [nodejs.org](https://nodejs.org)
- **Rust stable** — install via [rustup.rs](https://rustup.rs) (one
  command)
- **Git**
- **Platform build tools:**
  - **macOS** — `xcode-select --install`
  - **Windows** — Visual Studio Build Tools 2022 with the C++
    workload ([Microsoft download](https://visualstudio.microsoft.com/visual-cpp-build-tools/))
  - **Linux** — see [CONTRIBUTING.md](CONTRIBUTING.md) for the system
    package list (Ubuntu 22.04+ or equivalent)
- **An OpenRouter API key** — free account at
  [openrouter.ai](https://openrouter.ai). You need this to actually
  run prompts; the app will open without it, but you can't execute
  anything.

## Clone, install, run

```bash
git clone https://github.com/blossomz37/diamond-prompt-runner.git
cd diamond-prompt-runner
npm install
npm run tauri:dev
```

That's it. The first run will:

1. Install frontend dependencies (~1 min)
2. Compile the Rust backend and all dependencies (~5–10 min on a cold
   machine)
3. Start the Vite dev server on port 1420
4. Open a Diamond Prompt Runner desktop window

Subsequent runs are much faster — Rust incremental compile is typically
15–30 seconds. Keep the terminal open while you use the app; closing
it stops the dev server and quits the app.

**Stop the app:** press `Ctrl+C` in the terminal, or close the
Diamond Prompt Runner window.

## First time using the app

When the window opens you'll land on the project browser:

1. Paste your OpenRouter API key into the credentials field on the
   project browser screen and click **Save key**. The key is stored
   in your OS keychain (Keychain on macOS, Credential Manager on
   Windows, Secret Service on Linux), not in any config file in the
   repo.
2. Open one of the sample projects in the `Sample Projects/` folder
   (e.g. **Neon & Nightmares**) to see how blocks, templates, and
   pipelines are wired together.
3. Run a block or pipeline from inside the project to confirm the
   full round-trip (UI → Rust → OpenRouter → back) works.

## Troubleshooting

**Port 1420 already in use** — Vite refuses to start if port 1420 is
taken. Close whatever's using it (often a previous `npm run dev`
instance that didn't exit cleanly), or kill it:

```bash
lsof -ti:1420 | xargs kill -9     # macOS / Linux
```

**Rust build fails with linker errors on macOS** — you're missing
Xcode Command Line Tools. Run `xcode-select --install` and retry.

**Rust build fails with "link.exe not found" on Windows** — you're
missing the Visual Studio C++ Build Tools. Install them, restart the
terminal, and retry.

**WebKit errors on Linux** — you're on a distro older than Ubuntu
22.04 or Debian 12. Upgrade, or see [CONTRIBUTING.md](CONTRIBUTING.md)
for the full package list.

**First build is stuck compiling forever** — it's not stuck, it's
just slow. Diamond Prompt Runner pulls in ~400 Rust crates on a cold
build. Go make coffee. The second run will be fast.

**Gatekeeper warning on macOS** — only applies to the installed app,
not the dev build. You won't see it when running via `npm run tauri:dev`.

## Updating to newer code

```bash
git pull
npm install
npm run tauri:dev
```

`npm install` is only necessary if [package.json](package.json) or
[package-lock.json](package-lock.json) changed. When in doubt, run
it — it's a no-op if nothing changed.

## Next steps

- Read [CLAUDE.md](CLAUDE.md) for the architecture overview and data
  model.
- Read [README.md](README.md) for product direction and the current
  milestone status.
- Read [CONTRIBUTING.md](CONTRIBUTING.md) if you want to submit
  changes.
