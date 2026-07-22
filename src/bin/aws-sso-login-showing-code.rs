use clap::Parser;
use std::io::{BufRead, BufReader, Write};
use std::process::{Command, Stdio};

#[derive(Parser)]
struct Args {
    #[arg(long)]
    profile: Option<String>,

    #[arg(short, long)]
    verbose: bool,
}

fn main() {
    let args = Args::parse();

    // Keep the pipe open while `aws sso login` (Python) handles Ctrl+C and flushes stdout.
    // Without this, we die first, breaking the pipe and causing a Python BrokenPipeError.
    #[cfg(unix)]
    unsafe {
        libc::signal(libc::SIGINT, libc::SIG_IGN);
    }

    let mut cmd = Command::new("aws");
    cmd.args(["sso", "login"]);
    if let Some(profile) = &args.profile {
        cmd.args(["--profile", profile]);
    }
    cmd.stdin(Stdio::inherit())
        .stdout(Stdio::piped())
        .stderr(Stdio::inherit());

    let mut child = cmd.spawn().unwrap_or_else(|e| {
        eprintln!("error: failed to run aws sso login: {e}");
        std::process::exit(1);
    });

    let stdout = child.stdout.take().expect("stdout was piped");
    let mut accumulated = String::new();
    let mut show_code_child: Option<std::process::Child> = None;

    for line in BufReader::new(stdout).lines() {
        let line = match line {
            Ok(l) => l,
            Err(e) => {
                eprintln!("error: reading aws sso login output: {e}");
                break;
            }
        };
        println!("{line}");
        let _ = std::io::stdout().flush();

        if show_code_child.is_none() {
            // `accumulated` grows until the code marker is found. We can't cap
            // it without risking a missed marker, since AWS controls the output format.
            accumulated.push_str(&line);
            accumulated.push('\n');

            if let Some(code) = extract_code(&accumulated) {
                let mut show_cmd = Command::new(aws_sso_tools::sibling_bin("aws-sso-show-code"));
                show_cmd.arg(&code);
                if args.verbose {
                    show_cmd.arg("--verbose");
                }
                show_code_child = show_cmd
                    .spawn()
                    .inspect_err(|e| eprintln!("warning: failed to launch aws-sso-show-code: {e}"))
                    .ok();
                accumulated.clear();
            }
        }
    }

    let status = child.wait().unwrap_or_else(|e| {
        eprintln!("error: {e}");
        std::process::exit(1);
    });

    if let Some(mut proc) = show_code_child {
        proc.kill().ok();
        if let Ok(s) = proc.wait() {
            if !s.success() {
                eprintln!("warning: aws-sso-show-code exited with status {s}");
            }
        }
    } else {
        eprintln!("warning: aws sso login exited without showing a device code");
    }

    std::process::exit(status.code().unwrap_or(1));
}

fn extract_code(output: &str) -> Option<String> {
    let marker = "Then enter the code:\n\n";
    let pos = output.find(marker)?;
    let after = &output[pos + marker.len()..];
    let end = after.find('\n').unwrap_or(after.len());
    let code = after[..end].trim();
    if code.is_empty() {
        None
    } else {
        Some(code.to_owned())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    const REAL_AWS_OUTPUT: &str = "\
Attempting to automatically open the SSO authorization page in your default browser.\n\
If the browser does not open or you wish to use a different device to authorize this request, open the following URL:\n\
\n\
https://device.sso.us-east-1.amazonaws.com/\n\
\n\
Then enter the code:\n\
\n\
ABCD-1234\n\
";

    #[test]
    fn extracts_code_from_real_aws_output() {
        assert_eq!(extract_code(REAL_AWS_OUTPUT), Some("ABCD-1234".to_owned()));
    }

    #[test]
    fn returns_none_before_marker_arrives() {
        let partial = "Attempting to automatically open the SSO authorization page\n";
        assert_eq!(extract_code(partial), None);
    }

    #[test]
    fn returns_none_when_code_line_is_empty() {
        let output = "Then enter the code:\n\n\n";
        assert_eq!(extract_code(output), None);
    }

    #[test]
    fn returns_none_when_code_line_is_whitespace_only() {
        let output = "Then enter the code:\n\n   \n";
        assert_eq!(extract_code(output), None);
    }

    #[test]
    fn trims_whitespace_from_code() {
        let output = "Then enter the code:\n\n  WXYZ-5678  \n";
        assert_eq!(extract_code(output), Some("WXYZ-5678".to_owned()));
    }

    #[test]
    fn handles_code_at_end_of_string_without_trailing_newline() {
        let output = "Then enter the code:\n\nABCD-1234";
        assert_eq!(extract_code(output), Some("ABCD-1234".to_owned()));
    }
}
