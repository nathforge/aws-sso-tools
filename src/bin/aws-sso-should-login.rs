use clap::Parser;
use std::time::{Duration, SystemTime};

#[derive(Parser)]
struct Args {
    #[arg(long)]
    grace_period: Option<String>,

    #[arg(long)]
    profile: Option<String>,

    #[arg(short, long)]
    verbose: bool,
}

fn main() {
    let args = Args::parse();
    let profile = aws_sso_tools::active_profile::get(args.profile.as_deref());
    let grace = match args.grace_period {
        Some(s) => humantime::parse_duration(&s).unwrap_or_else(|e| {
            eprintln!("invalid --grace-period: {e}");
            std::process::exit(2);
        }),
        None => Duration::ZERO,
    };

    let (should_login, reason) = match aws_sso_tools::sso::expires_at(&profile) {
        Ok(expires_at) => match expires_at.duration_since(SystemTime::now()) {
            Ok(remaining) if remaining < grace => (
                true,
                format!(
                    "SSO token expires in {} (grace period: {})",
                    humantime::format_duration(remaining),
                    humantime::format_duration(grace)
                ),
            ),
            Ok(remaining) => (
                false,
                format!("SSO token valid for {}", humantime::format_duration(remaining)),
            ),
            Err(_) => (true, "SSO token has expired".to_owned()),
        },
        Err(aws_sso_tools::sso::Error::NoProfile(_)) => (
            false,
            format!("profile {profile:?} not found in ~/.aws/config"),
        ),
        Err(aws_sso_tools::sso::Error::NoSsoConfig(_)) => (
            false,
            format!("profile {profile:?} has no SSO configuration"),
        ),
        Err(aws_sso_tools::sso::Error::NoToken) => (true, "no SSO token cache found".to_owned()),
        Err(e) => {
            eprintln!("error: {e}");
            std::process::exit(2);
        }
    };

    if args.verbose {
        let prefix = if should_login { "should login" } else { "should not login" };
        println!("{prefix}: {reason}");
    }

    // Exit 0 = login is needed ("yes"); exit 1 = login is not needed ("no").
    // This is intentionally inverted from the UNIX success/failure convention:
    // the tool answers the yes/no question "should I login?", so 0 means true.
    std::process::exit(if should_login { 0 } else { 1 });
}
