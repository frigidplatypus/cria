//! Common utilities for integration tests
pub fn get_env_var(keys: &[&str], default: &str) -> String {
    for &key in keys {
        if let Ok(val) = std::env::var(key) {
            if !val.is_empty() {
                return val;
            }
        }
    }
    default.to_string()
}

pub fn should_skip_integration_test() -> bool {
    // Skip if no API token is provided (indicates no server available)
    let token = get_env_var(&["VIKUNJA_TOKEN", "VIKUNJA_API_TOKEN"], "");
    if token.is_empty() {
        return true;
    }
    
    // Skip if SKIP_INTEGRATION_TESTS is set
    if std::env::var("SKIP_INTEGRATION_TESTS").is_ok() {
        return true;
    }
    
    false
}
