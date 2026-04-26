<p align="center">
  <img src="src-tauri/icons/icon-readme.png" width="128" alt="GitSwitch icon" />
</p>

<h1 align="center">GitSwitch</h1>

<p align="center">
  Switch between GitHub and Bitbucket identities in one click.
</p>

---

If you work across multiple Git accounts — a personal GitHub, a work Bitbucket, a client repo — you've probably committed under the wrong name at least once. GitSwitch is a small desktop app that makes switching your active git identity instant, and (unlike just changing `git config`) also makes `git push` actually authenticate as the right user.

## What it does

- **Stores your accounts** — add your GitHub and Bitbucket accounts once, with the credentials needed to authenticate. Tokens go into your OS keychain, not on disk.
- **Sign in with GitHub** — one-click OAuth device flow for adding GitHub accounts. No personal access token typing required. (Manual PAT entry still works for fine-grained tokens or other providers.)
- **Switches both identity *and* auth** — one click sets your global `git config user.name` / `user.email` **and** writes the account's token into your OS git credential helper (Git Credential Manager on Windows, osxkeychain on macOS), then pins git to use that credential for the host. So `git push` over HTTPS authenticates as the right user automatically.
- **Detects untracked identities** — if your `.gitconfig` already has a name/email that GitSwitch doesn't manage, it shows up as a "Detected" card so you can add it as a managed account in one click.
- **Validates credentials** — checks that a stored token is still valid against the GitHub or Bitbucket API. On success, also re-runs the credential pinning so a stale GCM entry can't intercept your push.
- **Shows who's active** — always displays which identity your commits are currently going out as.
- **Open accounts folder** — quick titlebar button that reveals the on-disk config folder for inspection or backup.

## Installation

> GitSwitch is currently in early development. To run it, you'll need to build from source (instructions below). Pre-built releases are coming soon.

### Requirements

- macOS 12 or later, **or** Windows 10/11
- [Node.js](https://nodejs.org) (v18 or later)
- [Rust](https://rustup.rs)
- On Windows: [Visual Studio C++ Build Tools](https://visualstudio.microsoft.com/visual-cpp-build-tools/) (Desktop development with C++ workload) and [WebView2 Runtime](https://developer.microsoft.com/microsoft-edge/webview2/) (preinstalled on Windows 11)

### Build and run

```bash
git clone <your-fork-of-this-repo>
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

### Optional: use your own GitHub OAuth App

GitSwitch ships with a default Client ID for the **Sign in with GitHub**
button, so device-flow login works out of the box. If you want to use your
own OAuth App instead (to see authorization analytics under your account, or
to avoid sharing rate limits), register one and override the Client ID at
build time:

1. Go to [github.com/settings/developers](https://github.com/settings/developers) → **OAuth Apps** → **New OAuth App**
2. Fill in any Homepage URL and Authorization callback URL (device flow doesn't use them — the form just requires values)
3. ✅ Check **Enable Device Flow**
4. Save, then copy the **Client ID** (looks like `Ov23li...` or `Iv1.xxxxx`)
5. Set it as an environment variable before building:

   ```bash
   # macOS / Linux
   export GITHUB_CLIENT_ID=Ov23liXXXXXXXXXXXX
   npm run tauri dev

   # Windows (PowerShell)
   $env:GITHUB_CLIENT_ID = "Ov23liXXXXXXXXXXXX"
   npm run tauri dev
   ```

## Adding an account

You have two options:

### Option A: Sign in with GitHub (recommended for github.com)

1. Click **Add Account** in the top right
2. Click **Sign in with GitHub** at the top of the dialog
3. Click **Copy code & open GitHub** — your browser opens to `github.com/login/device`
4. Paste the 8-character code, sign in, and authorize **GitSwitch_OAuth**
5. Within a few seconds, the dialog auto-fills your username, primary email, and a fresh OAuth token, marked as ✓ valid
6. Click **Add Account**

The token has `read:user user:email repo` scope so it can be used for both identity and `git push` / `git clone`.

### Option B: Manual personal access token

1. Click **Add Account** in the top right
2. Choose GitHub or Bitbucket
3. Enter your display name, username, email, and a personal access token (GitHub) or app password (Bitbucket)
4. Optionally click **Test Credentials** to verify the token works
5. Click **Save**

### Where to generate tokens manually

- **GitHub** — Settings → Developer Settings → Personal Access Tokens → Generate new token (needs `repo` scope for push to work)
- **Bitbucket** — Personal Settings → App Passwords → Create app password

## What happens when you click Switch

Switching to an account does three things in order:

1. Updates `git config --global user.name` and `user.email` so future commits are attributed to that account
2. Clears any stale unnamespaced credential for the host (e.g. an old `git:https://github.com` entry from a previous setup) and writes the account's token into your OS credential helper under the account's real username
3. Sets `credential.https://<host>.username` in git config so future `git push` requests ask the helper for *this* user's namespaced credential — preventing GCM from silently returning a stale default

This means after a switch, both `git commit` (identity) and `git push` (auth) use the account you selected — no more wrong-account pushes.

The **Validate credentials** action (in each account's `…` menu) does step 2 and 3 without touching commit identity, so you can fix up GCM entries without changing who you're committing as.

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
