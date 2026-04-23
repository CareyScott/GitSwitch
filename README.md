<p align="center">
  <img src="src-tauri/icons/128x128@2x.png" width="128" alt="GitSwitch icon" />
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

- macOS 12 or later
- [Node.js](https://nodejs.org) (v18 or later)
- [Rust](https://rustup.rs)

### Build and run

```bash
git clone https://github.com/CareyScott/GitSwitch.git
cd GitSwitch
npm install
npm run tauri dev
```

To build and install the app:

```bash
npm run tauri build
cp -r src-tauri/target/release/bundle/macos/GitSwitch.app /Applications/
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

GitSwitch never sends your tokens anywhere. Everything is stored locally in a plain JSON file on your Mac:

```
~/Library/Application Support/git-switch/accounts.json
```

That file is only readable by your user account. No cloud sync, no telemetry, no servers involved. When you click **Switch**, GitSwitch writes directly to your local `~/.gitconfig` — the same file the `git` command reads.

---

Built with [Tauri](https://tauri.app), React, and TypeScript.
