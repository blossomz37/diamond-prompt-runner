# Contributing to Diamond Prompt Runner

Thanks for your interest. This document gets you from a fresh clone to
a running dev build on macOS, Windows, or Linux. For architecture,
commands, and coding conventions, read [CLAUDE.md](CLAUDE.md) and
[AGENTS.md](AGENTS.md) after you have the dev loop working — those
documents are the source of truth for how the codebase is organized.

## Prerequisites

All platforms need these three things:

- **Node.js 20 LTS** — from [nodejs.org](https://nodejs.org) or via `nvm`
- **Rust stable** — install with [rustup](https://rustup.rs)
- **Git**

Plus platform-specific build dependencies below.

### macOS

Install Xcode Command Line Tools:

```bash
xcode-select --install
```

Apple Silicon and Intel Macs are both supported. Minimum macOS version
is whatever the current Tauri 2 release supports (currently macOS 10.15
Catalina+).

### Windows

1. Install **Visual Studio Build Tools 2022** with the **C++ build
   tools** workload. Download from
   [Microsoft's Build Tools page](https://visualstudio.microsoft.com/visual-cpp-build-tools/).
   This includes the Windows 10/11 SDK.
2. **WebView2 runtime** is preinstalled on Windows 10 build 1803+ and
   all Windows 11, so you probably don't need to install it. If you're
   on something older, grab the evergreen installer from Microsoft.
3. Recommended: enable long path support (Windows has a legacy 260-char
   path limit that occasionally bites Rust builds). Open `gpedit.msc`
   → Computer Configuration → Administrative Templates → System →
   Filesystem → **Enable Win32 long paths**.
4. Recommended: add the project directory to Windows Defender
   exclusions. Real-time scanning can slow `cargo build` significantly.

**Note:** Use native Windows, not WSL2. Tauri dev mode under WSL2 is
flaky — WebKit2GTK rendering depends on WSLg passthrough and breaks
unpredictably.

### Linux

Minimum distro version: **Ubuntu 22.04 / Debian 12** or equivalent.
Older distros ship `libwebkit2gtk-4.0`, which Tauri 2 does not support.

Ubuntu / Debian:

```bash
sudo apt update
sudo apt install -y \
  libwebkit2gtk-4.1-dev \
  build-essential \
  curl wget file \
  libssl-dev \
  libayatana-appindicator3-dev \
  librsvg2-dev
```

Fedora / RHEL:

```bash
sudo dnf install -y \
  webkit2gtk4.1-devel \
  openssl-devel \
  libappindicator-gtk3-devel \
  librsvg2-devel \
  @development-tools
```

Arch:

```bash
sudo pacman -S webkit2gtk-4.1 base-devel openssl libayatana-appindicator librsvg
```

**Credential storage note:** Diamond Prompt Runner stores your
OpenRouter API key in the system keyring, which on Linux uses the
Secret Service D-Bus API (provided by `gnome-keyring-daemon` or
`kwalletd`). Most desktop installs have this running by default. On
headless systems, WSL, or minimal distros, the keyring won't be
available — set `OPENROUTER_API_KEY` in a `.env` file at the project
root instead.

## Clone and run

```bash
git clone https://github.com/blossomz37/diamond-prompt-runner.git
cd diamond-prompt-runner
npm install
npm run tauri:dev
```

The first `cargo build` takes 5–10 minutes cold while it compiles the
Rust dependency tree. Later runs are incremental and fast. A Tauri
window should open when the build completes.

## Everyday commands

```bash
npm run tauri:dev       # full desktop app with hot reload (use this)
npm run dev             # frontend only, port 1420
npm run typecheck       # svelte-check type errors
npm run lint            # eslint
npm run test            # vitest frontend tests
npx vitest run src/App.test.ts                      # single frontend test file
cargo test --manifest-path src-tauri/Cargo.toml     # Rust backend tests
cargo test --manifest-path src-tauri/Cargo.toml <name>  # single Rust test
```

Run `npm run typecheck`, `npm run lint`, and the relevant test suites
before submitting a PR.

## How changes are structured

Diamond Prompt Runner follows a **strict two-layer separation**:

- **Frontend** (Svelte 5 + TypeScript) — UI state, presentation, no
  direct filesystem or HTTP access
- **Backend** (Rust + Tauri 2) — all filesystem I/O, credential
  management, HTTP calls, template rendering
- **Bridge** — [src/lib/tauri.ts](src/lib/tauri.ts) is the **only**
  place frontend talks to backend. Any new Tauri command must be
  registered in [src-tauri/src/main.rs](src-tauri/src/main.rs) and
  wrapped in `tauri.ts` before the frontend can call it.

Features are built as **vertical slices**: UI → tauri.ts → Rust as one
cohesive change, not layer-by-layer.

See [CLAUDE.md](CLAUDE.md) for the full architecture, the project data
model on disk, and the key files table.

## Pull request expectations

- **Keep one PR focused on one outcome.** If you find adjacent issues,
  note them and open follow-ups rather than widening scope.
- **Ask before changing architecture, file formats, public behavior,
  or project direction** — open an issue first to discuss.
- **Include tests** for behavior changes. Frontend changes use Vitest
  + Testing Library; backend changes use Rust's built-in test
  framework with `tempfile` for isolated project fixtures.
- **Pass `npm run typecheck` and `npm run lint`.** CI will block PRs
  that fail these.
- **Don't mix unrelated changes** in one commit.

## Getting help

- Product questions and feature requests → open an issue
- Bug reports → open an issue with reproduction steps, platform, and
  Diamond Prompt Runner version
- Architecture or scope questions → open an issue *before* writing code
