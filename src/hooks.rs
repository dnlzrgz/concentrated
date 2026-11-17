use std::{
    io,
    process::{Child, Command, Stdio},
    thread,
};

use crate::app::SessionEnd;

/// Launches `command` in the background and returns immediately. Fires and forget
/// strategy.
pub fn run(command: &str, end: SessionEnd) {
    if let Ok(mut child) = spawn(command, end) {
        thread::spawn(move || {
            let _ = child.wait();
        });
    }
}

fn spawn(command: &str, end: SessionEnd) -> io::Result<Child> {
    let mut cmd = shell(command);
    cmd.env("CONCENTRATED_SESSION", end.finished.id())
        .env("CONCENTRATED_NEXT_SESSION", end.next.id())
        // The hook must not read keys or draw over the TUI.
        .stdin(Stdio::null())
        .stdout(Stdio::null())
        .stderr(Stdio::null());

    #[cfg(unix)]
    {
        use std::os::unix::process::CommandExt;
        // Own process group, so signals aimed at the app's group don't reach the hook.
        cmd.process_group(0);
    }

    cmd.spawn()
}

#[cfg(unix)]
fn shell(command: &str) -> Command {
    let mut cmd = Command::new("sh");
    cmd.arg("-c").arg(command);
    cmd
}

#[cfg(windows)]
fn shell(command: &str) -> Command {
    use std::os::windows::process::CommandExt;

    let mut cmd = Command::new("cmd");
    cmd.arg("/C").raw_arg(command);
    cmd
}

#[cfg(all(test, unix))]
mod tests {
    use super::*;
    use crate::app::SessionType;

    const END: SessionEnd = SessionEnd {
        finished: SessionType::Work,
        next: SessionType::LongBreak,
    };

    #[test]
    fn hook_receives_the_session_ids() {
        let out = std::env::temp_dir().join(format!("concentrated-hook-{}", std::process::id()));
        let command = format!(
            "echo \"$CONCENTRATED_SESSION $CONCENTRATED_NEXT_SESSION\" > '{}'",
            out.display()
        );

        spawn(&command, END).unwrap().wait().unwrap();

        assert_eq!(std::fs::read_to_string(&out).unwrap(), "work long-break\n");
        let _ = std::fs::remove_file(out);
    }

    #[test]
    fn run_never_fails_on_a_broken_command() {
        run("not-a-command", END);
        run("", END);
    }
}
