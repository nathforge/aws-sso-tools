use std::ffi::CString;
use std::process::{Command, Stdio};

fn main() {
    let args: Vec<String> = std::env::args().skip(1).collect();

    if args.is_empty() {
        eprintln!("usage: aws-sso-run <command> [args...]");
        std::process::exit(1);
    }

    let status = Command::new(aws_sso_tools::sibling_bin("aws-sso-maybe-login"))
        .stdin(Stdio::inherit())
        .stdout(Stdio::inherit())
        .stderr(Stdio::inherit())
        .status()
        .unwrap_or_else(|e| {
            eprintln!("error: failed to run aws-sso-maybe-login: {e}");
            std::process::exit(1);
        });

    if !status.success() {
        std::process::exit(status.code().unwrap_or(1));
    }

    let prog = CString::new(args[0].as_str()).unwrap();
    let cargs: Vec<CString> = args
        .iter()
        .map(|a| CString::new(a.as_str()).unwrap())
        .collect();

    match nix::unistd::execvp(&prog, &cargs) {
        Ok(_) => unreachable!(),
        Err(e) => {
            eprintln!("error: failed to exec {}: {e}", args[0]);
            std::process::exit(1);
        }
    }
}
