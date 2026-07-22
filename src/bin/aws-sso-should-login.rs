use clap::Parser;
use std::time::{Duration, SystemTime};

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
    let profile = aws_sso_tools::active_profile::get(args.profile.as_deref());
    let grace = Duration::from_secs(args.grace_period);

    let (should_login, reason) = match aws_sso_tools::sso::expires_at(&profile) {
        Ok(expires_at) => match expires_at.duration_since(SystemTime::now()) {
            Ok(remaining) if remaining < grace => (
                true,
                format!(
                    "SSO token expires in {} (grace period: {})",
                    fmt_duration(remaining),
                    fmt_duration(grace)
                ),
            ),
            Ok(remaining) => (
                false,
                format!("SSO token valid for {}", fmt_duration(remaining)),
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

fn fmt_duration(d: Duration) -> String {
    let secs = d.as_secs();
    if secs < 60 {
        format!("{secs}s")
    } else if secs < 3600 {
        let m = secs / 60;
        let s = secs % 60;
        if s == 0 {
            format!("{m}m")
        } else {
            format!("{m}m {s}s")
        }
    } else {
        let h = secs / 3600;
        let m = (secs % 3600) / 60;
        if m == 0 {
            format!("{h}h")
        } else {
            format!("{h}h {m}m")
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn secs(n: u64) -> Duration {
        Duration::from_secs(n)
    }

    #[test]
    fn fmt_duration_seconds() {
        assert_eq!(fmt_duration(secs(0)), "0s");
        assert_eq!(fmt_duration(secs(1)), "1s");
        assert_eq!(fmt_duration(secs(59)), "59s");
    }

    #[test]
    fn fmt_duration_minutes_boundary() {
        assert_eq!(fmt_duration(secs(60)), "1m");
        assert_eq!(fmt_duration(secs(61)), "1m 1s");
        assert_eq!(fmt_duration(secs(90)), "1m 30s");
    }

    #[test]
    fn fmt_duration_minutes_exact() {
        assert_eq!(fmt_duration(secs(120)), "2m");
        assert_eq!(fmt_duration(secs(3599)), "59m 59s");
    }

    #[test]
    fn fmt_duration_hours_boundary() {
        assert_eq!(fmt_duration(secs(3600)), "1h");
        assert_eq!(fmt_duration(secs(3660)), "1h 1m");
        assert_eq!(fmt_duration(secs(7200)), "2h");
        assert_eq!(fmt_duration(secs(7261)), "2h 1m");
    }

    #[test]
    fn fmt_duration_hours_no_minutes() {
        assert_eq!(fmt_duration(secs(3600 * 8)), "8h");
    }
}
