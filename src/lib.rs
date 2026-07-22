pub mod active_profile;
pub mod sso;

pub fn sibling_bin(name: &str) -> std::path::PathBuf {
    let exe_name = if cfg!(windows) { format!("{name}.exe") } else { name.to_string() };
    std::env::current_exe()
        .ok()
        .and_then(|p| p.parent().map(|d| d.join(&exe_name)))
        .unwrap_or_else(|| std::path::PathBuf::from(&exe_name))
}
