pub mod active_profile;
pub mod sso;

pub fn parse_grace_period(s: &str) -> Result<std::time::Duration, String> {
    humantime::parse_duration(s).map_err(|e| format!("invalid duration: {e}"))
}

pub fn sibling_bin(name: &str) -> std::path::PathBuf {
    std::env::current_exe()
        .ok()
        .and_then(|p| p.parent().map(|d| d.join(name)))
        .unwrap_or_else(|| std::path::PathBuf::from(name))
}
