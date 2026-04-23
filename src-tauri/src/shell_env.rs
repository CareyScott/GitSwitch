// Harvests environment variables from the user's login+interactive shell so
// the app sees the same PATH when launched from Finder/Dock as it would from
// a terminal. Needed because `git` may not be on launchd's default PATH.

#[cfg(target_os = "macos")]
const HARVEST_VARS: &[&str] = &["PATH"];

#[cfg(target_os = "macos")]
pub fn hydrate_from_login_shell() {
    use std::process::Command;
    use std::time::Duration;

    let any_missing = HARVEST_VARS.iter().any(|v| {
        std::env::var(v).map(|s| s.is_empty()).unwrap_or(true)
    });
    if !any_missing {
        return;
    }

    let shell = std::env::var("SHELL").unwrap_or_else(|_| "/bin/zsh".into());

    let mut script = String::new();
    for v in HARVEST_VARS {
        script.push_str(&format!(
            "printf '%s=%s\\0' '{name}' \"${name}\"; ",
            name = v
        ));
    }

    let mut child = match Command::new(&shell)
        .args(["-ilc", &script])
        .stdin(std::process::Stdio::null())
        .stdout(std::process::Stdio::piped())
        .stderr(std::process::Stdio::null())
        .spawn()
    {
        Ok(c) => c,
        Err(_) => return,
    };

    let timeout = Duration::from_secs(5);
    let start = std::time::Instant::now();
    let output = loop {
        match child.try_wait() {
            Ok(Some(_)) => break child.wait_with_output(),
            Ok(None) => {
                if start.elapsed() > timeout {
                    let _ = child.kill();
                    return;
                }
                std::thread::sleep(Duration::from_millis(25));
            }
            Err(_) => return,
        }
    };

    let output = match output {
        Ok(o) => o,
        Err(_) => return,
    };
    if !output.status.success() {
        return;
    }

    for chunk in output.stdout.split(|b| *b == 0) {
        if chunk.is_empty() {
            continue;
        }
        let s = match std::str::from_utf8(chunk) {
            Ok(s) => s,
            Err(_) => continue,
        };
        let Some((k, v)) = s.split_once('=') else {
            continue;
        };
        if v.is_empty() {
            continue;
        }
        if std::env::var(k).map(|cur| !cur.is_empty()).unwrap_or(false) {
            continue;
        }
        std::env::set_var(k, v);
    }
}

#[cfg(not(target_os = "macos"))]
pub fn hydrate_from_login_shell() {}
