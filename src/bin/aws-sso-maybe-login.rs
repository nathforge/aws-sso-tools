use clap::Parser;
use std::process::{Command, Stdio};

#[derive(Parser)]
struct Args {
    #[arg(long, default_value_t = 0)]
    grace_period: u64,

    #[arg(long)]
    profile: Option<String>,

    #[arg(short, long)]
    verbose: bool,
}

fn main() {
    let args = Args::parse();

    let mut should_login_cmd = Command::new(aws_sso_tools::sibling_bin("aws-sso-should-login"));
    should_login_cmd
        .args(["--grace-period", &args.grace_period.to_string()])
        .stdin(Stdio::inherit())
        .stdout(Stdio::inherit())
        .stderr(Stdio::inherit());
    if let Some(profile) = &args.profile {
        should_login_cmd.args(["--profile", profile]);
    }
    if args.verbose {
        should_login_cmd.arg("--verbose");
    }

    let status = should_login_cmd
        .status()
        .unwrap_or_else(|e| {
            eprintln!("error: failed to run aws-sso-should-login: {e}");
            std::process::exit(1);
        });

    match status.code() {
        Some(0) => {}
        Some(1) => std::process::exit(0), // login not required
        _ => std::process::exit(status.code().unwrap_or(1)),
    }

    let mut login_cmd = Command::new(aws_sso_tools::sibling_bin("aws-sso-login-showing-code"));
    login_cmd
        .stdin(Stdio::inherit())
        .stdout(Stdio::inherit())
        .stderr(Stdio::inherit());
    if let Some(profile) = &args.profile {
        login_cmd.args(["--profile", profile]);
    }
    if args.verbose {
        login_cmd.arg("--verbose");
    }

    let status = login_cmd
        .status()
        .unwrap_or_else(|e| {
            eprintln!("error: failed to run aws-sso-login-showing-code: {e}");
            std::process::exit(1);
        });

    std::process::exit(status.code().unwrap_or(1));
}
