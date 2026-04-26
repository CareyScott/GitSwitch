<p align="center">
  <img src="src-tauri/icons/icon-readme.png" width="128" alt="GitSwitch icon" />
</p>

<h1 align="center">GitSwitch</h1>

<p align="center">
  Switch between GitHub and Bitbucket identities in one click.
</p>

---

If you work across multiple Git accounts — a personal GitHub, a work Bitbucket, a client repo — you've probably committed under the wrong name at least once. GitSwitch sits in your menubar and makes switching your active git identity instant.

## What it does

- **Stores your accounts** — add your GitHub and Bitbucket accounts once, with the credentials needed to authenticate
- **Switches your identity** — one click sets your global `git config` name and email to that account
- **Shows who's active** — always displays which identity your commits are currently going out as
- **Validates credentials** — checks that a stored token is still valid against the GitHub or Bitbucket API

## Installation

> GitSwitch is currently in early development. To run it, you'll need to build from source (instructions below). Pre-built releases are coming soon.

### Requirements

- macOS 12 or later, **or** Windows 10/11
- [Node.js](https://nodejs.org) (v18 or later)
- [Rust](https://rustup.rs)
- On Windows: [Visual Studio C++ Build Tools](https://visualstudio.microsoft.com/visual-cpp-build-tools/) (Desktop development with C++ workload) and [WebView2 Runtime](https://developer.microsoft.com/microsoft-edge/webview2/) (preinstalled on Windows 11)

### Build and run

```bash
git clone https://github.com/CareyScott/GitSwitch.git
cd GitSwitch
npm install
npm run tauri dev
```

To build and install:

```bash
# macOS — copies to /Applications
npm run install:app

# Windows — runs the NSIS installer it produces
npm run install:app:win
```

## Adding an account

1. Click **Add Account** in the top right
2. Choose GitHub or Bitbucket
3. Enter your display name, username, email, and a personal access token (GitHub) or app password (Bitbucket)
4. Optionally click **Test Credentials** to verify the token works
5. Click **Save**

### Where to generate tokens

- **GitHub** — Settings → Developer Settings → Personal Access Tokens → Generate new token
- **Bitbucket** — Personal Settings → App Passwords → Create app password

## Your credentials stay on your machine

GitSwitch never sends your tokens anywhere. Account metadata (label, username, email) is stored in a plain JSON file in your OS's config directory; **tokens are stored separately in your operating system's keychain** so they never sit on disk in plaintext.

| OS      | Account metadata                                       | Tokens                              |
| ------- | ------------------------------------------------------ | ----------------------------------- |
| macOS   | `~/Library/Application Support/git-switch/accounts.json` | macOS Keychain (`git-switch` service) |
| Windows | `%APPDATA%\git-switch\accounts.json`                   | Windows Credential Manager          |
| Linux   | `~/.config/git-switch/accounts.json`                   | Secret Service (gnome-keyring / KWallet) |

No cloud sync, no telemetry, no servers involved. When you click **Switch**, GitSwitch writes directly to your global `git config` — the same file the `git` command reads.

If you upgrade from an earlier version that stored tokens in `accounts.json`, GitSwitch will silently migrate them into the OS keychain on first launch.

---

Built with [Tauri](https://tauri.app), React, and TypeScript.
