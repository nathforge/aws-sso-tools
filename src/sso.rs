use std::collections::HashMap;
use std::path::PathBuf;
use std::time::SystemTime;

#[derive(thiserror::Error, Debug)]
pub enum Error {
    #[error("profile {0:?} not found in ~/.aws/config")]
    NoProfile(String),
    #[error("profile {0:?} has no SSO configuration")]
    NoSsoConfig(String),
    #[error("no SSO token cache found")]
    NoToken,
    #[error(transparent)]
    Io(#[from] std::io::Error),
    #[error("{0}")]
    Parse(String),
}

pub fn expires_at(profile: &str) -> Result<SystemTime, Error> {
    let config = aws_config()?;

    let section_key = if profile == "default" {
        "default".to_owned()
    } else {
        format!("profile {profile}")
    };
    let section = config
        .get(&section_key)
        .ok_or_else(|| Error::NoProfile(profile.to_owned()))?;

    // sso_session (new style) takes priority over sso_start_url (legacy)
    let cache_key = if let Some(session_name) = section.get("sso_session") {
        let session_section = config
            .get(&format!("sso-session {session_name}"))
            .ok_or_else(|| Error::Parse(format!("sso-session {session_name:?} not found")))?;
        if !session_section.contains_key("sso_start_url") {
            return Err(Error::NoSsoConfig(profile.to_owned()));
        }
        session_name.clone()
    } else if let Some(url) = section.get("sso_start_url") {
        url.clone()
    } else {
        return Err(Error::NoSsoConfig(profile.to_owned()));
    };

    let json = std::fs::read_to_string(token_path(&cache_key)?).map_err(|e| {
        if e.kind() == std::io::ErrorKind::NotFound {
            Error::NoToken
        } else {
            Error::Io(e)
        }
    })?;
    parse_expires_at(&json)
}

fn home_dir() -> Result<PathBuf, Error> {
    if let Ok(home) = std::env::var("HOME") {
        if !home.is_empty() {
            return Ok(PathBuf::from(home));
        }
    }
    passwd_home_dir().ok_or_else(|| Error::Parse("cannot determine home directory".into()))
}

fn passwd_home_dir() -> Option<PathBuf> {
    use std::ffi::CStr;
    let uid = unsafe { libc::getuid() };
    let mut pwd = unsafe { std::mem::zeroed::<libc::passwd>() };
    let mut result: *mut libc::passwd = std::ptr::null_mut();
    // Start at 4 KiB and double on ERANGE, up to 64 KiB.
    let mut buf = vec![0 as libc::c_char; 4096];
    loop {
        let ret = unsafe {
            libc::getpwuid_r(uid, &mut pwd, buf.as_mut_ptr(), buf.len(), &mut result)
        };
        if ret == libc::ERANGE && buf.len() < 65536 {
            buf.resize(buf.len() * 2, 0);
            continue;
        }
        if ret != 0 || result.is_null() {
            return None;
        }
        break;
    }
    let dir = unsafe { CStr::from_ptr((*result).pw_dir) };
    dir.to_str().ok().map(PathBuf::from)
}

fn aws_config() -> Result<HashMap<String, HashMap<String, String>>, Error> {
    let path = home_dir()?.join(".aws/config");
    let content = std::fs::read_to_string(path).map_err(Error::Io)?;
    Ok(parse_ini(&content))
}

// Parses the subset of INI used by AWS config (same rules as Python's RawConfigParser):
// sections [name], key=value pairs, # and ; comment lines. Inline comments and
// multi-line continuation values are not needed for ~/.aws/config and not handled.
fn parse_ini(content: &str) -> HashMap<String, HashMap<String, String>> {
    let mut sections: HashMap<String, HashMap<String, String>> = HashMap::new();
    let mut current = String::new();
    for line in content.lines() {
        let line = line.trim();
        if line.is_empty() || line.starts_with(';') || line.starts_with('#') {
            continue;
        }
        if let Some(inner) = line.strip_prefix('[').and_then(|l| l.strip_suffix(']')) {
            current = inner.trim().to_owned();
        } else if let Some((k, v)) = line.split_once('=') {
            sections
                .entry(current.clone())
                .or_default()
                .insert(k.trim().to_owned(), v.trim().to_owned());
        }
    }
    sections
}

fn token_path(cache_key: &str) -> Result<PathBuf, Error> {
    use sha1::{Digest, Sha1};
    let hash = Sha1::digest(cache_key.as_bytes());
    use std::fmt::Write as _;
    let mut hex = String::with_capacity(40);
    for b in &hash {
        write!(hex, "{b:02x}").unwrap();
    }
    Ok(home_dir()?.join(format!(".aws/sso/cache/{hex}.json")))
}

fn parse_expires_at(json: &str) -> Result<SystemTime, Error> {
    #[derive(serde::Deserialize)]
    struct Token {
        #[serde(rename = "expiresAt")]
        expires_at: String,
    }
    let token: Token = serde_json::from_str(json)
        .map_err(|e| Error::Parse(format!("parsing token cache: {e}")))?;
    chrono::DateTime::parse_from_rfc3339(&token.expires_at)
        .map(|dt| SystemTime::from(dt.with_timezone(&chrono::Utc)))
        .map_err(|e| Error::Parse(format!("parsing expiry time: {e}")))
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::time::UNIX_EPOCH;

    // --- parse_ini ---

    #[test]
    fn parse_ini_basic_section_and_key() {
        let ini = "[default]\nregion = us-east-1\n";
        let result = parse_ini(ini);
        assert_eq!(result["default"]["region"], "us-east-1");
    }

    #[test]
    fn parse_ini_multiple_sections() {
        let ini = "[default]\nregion = us-east-1\n\n[profile dev]\nregion = eu-west-1\n";
        let result = parse_ini(ini);
        assert_eq!(result["default"]["region"], "us-east-1");
        assert_eq!(result["profile dev"]["region"], "eu-west-1");
    }

    #[test]
    fn parse_ini_trims_whitespace() {
        let ini = "[default]\n  region  =  us-east-1  \n";
        let result = parse_ini(ini);
        assert_eq!(result["default"]["region"], "us-east-1");
    }

    #[test]
    fn parse_ini_skips_hash_comments() {
        let ini = "# this is a comment\n[default]\nregion = us-east-1\n";
        let result = parse_ini(ini);
        assert!(!result.contains_key("# this is a comment"));
        assert_eq!(result["default"]["region"], "us-east-1");
    }

    #[test]
    fn parse_ini_skips_semicolon_comments() {
        let ini = "; this is a comment\n[default]\nregion = us-east-1\n";
        let result = parse_ini(ini);
        assert_eq!(result["default"]["region"], "us-east-1");
    }

    #[test]
    fn parse_ini_aws_profile_format() {
        let ini = "\
[default]
sso_session = my-session
region = us-east-1

[sso-session my-session]
sso_start_url = https://my-sso.awsapps.com/start
sso_region = us-east-1
";
        let result = parse_ini(ini);
        assert_eq!(result["default"]["sso_session"], "my-session");
        assert_eq!(
            result["sso-session my-session"]["sso_start_url"],
            "https://my-sso.awsapps.com/start"
        );
    }

    #[test]
    fn parse_ini_value_with_equals_sign() {
        // split_once('=') correctly takes only the first '=' as the separator
        let ini = "[default]\nurl = https://example.com/path?foo=bar\n";
        let result = parse_ini(ini);
        assert_eq!(result["default"]["url"], "https://example.com/path?foo=bar");
    }

    // --- parse_expires_at ---

    #[test]
    fn parse_expires_at_valid_rfc3339() {
        let json = r#"{"expiresAt": "2024-01-15T12:00:00Z"}"#;
        let result = parse_expires_at(json).unwrap();
        let secs = result.duration_since(UNIX_EPOCH).unwrap().as_secs();
        assert_eq!(secs, 1705320000);
    }

    #[test]
    fn parse_expires_at_invalid_json() {
        let result = parse_expires_at("not json");
        assert!(matches!(result, Err(Error::Parse(_))));
    }

    #[test]
    fn parse_expires_at_missing_field() {
        let json = r#"{"someOtherField": "value"}"#;
        let result = parse_expires_at(json);
        assert!(matches!(result, Err(Error::Parse(_))));
    }

    #[test]
    fn parse_expires_at_invalid_datetime() {
        let json = r#"{"expiresAt": "not-a-date"}"#;
        let result = parse_expires_at(json);
        assert!(matches!(result, Err(Error::Parse(_))));
    }

    // Serialize all tests that mutate HOME; env vars are global process state.
    static ENV_LOCK: std::sync::Mutex<()> = std::sync::Mutex::new(());

    // --- token_path ---

    #[test]
    fn token_path_produces_40_char_hex_filename() {
        let _guard = ENV_LOCK.lock().unwrap();
        unsafe { std::env::set_var("HOME", std::env::temp_dir()) };
        let path = token_path("https://my-sso.awsapps.com/start").unwrap();
        let filename = path.file_name().unwrap().to_str().unwrap();
        assert!(filename.ends_with(".json"));
        let stem = &filename[..filename.len() - 5];
        assert_eq!(stem.len(), 40);
        assert!(stem.chars().all(|c| c.is_ascii_hexdigit()));
    }

    #[test]
    fn token_path_is_deterministic() {
        let _guard = ENV_LOCK.lock().unwrap();
        unsafe { std::env::set_var("HOME", std::env::temp_dir()) };
        let a = token_path("my-session").unwrap();
        let b = token_path("my-session").unwrap();
        assert_eq!(a, b);
    }

    #[test]
    fn token_path_differs_for_different_keys() {
        let _guard = ENV_LOCK.lock().unwrap();
        unsafe { std::env::set_var("HOME", std::env::temp_dir()) };
        let a = token_path("session-a").unwrap();
        let b = token_path("session-b").unwrap();
        assert_ne!(a, b);
    }
}
