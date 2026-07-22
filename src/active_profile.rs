pub fn get(profile_arg: Option<&str>) -> String {
    if let Some(p) = profile_arg {
        return p.to_owned();
    }
    for var in ["AWS_PROFILE", "AWS_DEFAULT_PROFILE"] {
        if let Ok(p) = std::env::var(var) {
            if !p.is_empty() {
                return p;
            }
        }
    }
    "default".to_owned()
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::Mutex;

    // Serialize all env-var tests within this module; env vars are global process state.
    static ENV_LOCK: Mutex<()> = Mutex::new(());

    #[test]
    fn explicit_arg_takes_priority() {
        assert_eq!(get(Some("myprofile")), "myprofile");
    }

    #[test]
    fn falls_back_to_default_when_no_env_vars() {
        let _guard = ENV_LOCK.lock().unwrap();
        unsafe {
            std::env::remove_var("AWS_PROFILE");
            std::env::remove_var("AWS_DEFAULT_PROFILE");
        }
        assert_eq!(get(None), "default");
    }

    #[test]
    fn uses_aws_profile_env_var() {
        let _guard = ENV_LOCK.lock().unwrap();
        unsafe {
            std::env::remove_var("AWS_DEFAULT_PROFILE");
            std::env::set_var("AWS_PROFILE", "from-env");
        }
        let result = get(None);
        unsafe { std::env::remove_var("AWS_PROFILE") };
        assert_eq!(result, "from-env");
    }

    #[test]
    fn aws_default_profile_is_fallback() {
        let _guard = ENV_LOCK.lock().unwrap();
        unsafe {
            std::env::remove_var("AWS_PROFILE");
            std::env::set_var("AWS_DEFAULT_PROFILE", "from-default-env");
        }
        let result = get(None);
        unsafe { std::env::remove_var("AWS_DEFAULT_PROFILE") };
        assert_eq!(result, "from-default-env");
    }

    #[test]
    fn empty_env_var_is_skipped() {
        let _guard = ENV_LOCK.lock().unwrap();
        unsafe {
            std::env::set_var("AWS_PROFILE", "");
            std::env::remove_var("AWS_DEFAULT_PROFILE");
        }
        let result = get(None);
        unsafe { std::env::remove_var("AWS_PROFILE") };
        assert_eq!(result, "default");
    }
}
