use std::env;

/// Configuration loader that reads from environment variables
pub struct Config {
    pub app_conf_path: Option<String>,
}

impl Config {
    pub fn new() -> Self {
        Self {
            app_conf_path: env::var("APP_CONF").ok(),
        }
    }
    
    pub fn get_app_conf_path(&self) -> Option<&str> {
        self.app_conf_path.as_deref()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::Mutex;
    use std::env;

    // Global mutex to ensure tests run serially when accessing environment variables
    static ENV_MUTEX: Mutex<()> = Mutex::new(());

    #[test]
    fn test_env_var_priority() {
        let _guard = ENV_MUTEX.lock().unwrap();
        
        // Store original value
        let original_value = env::var("APP_CONF").ok();
        
        // Set test value
        env::set_var("APP_CONF", "/custom/path.conf");
        
        // Test the configuration
        let config = Config::new();
        assert_eq!(config.get_app_conf_path(), Some("/custom/path.conf"));
        
        // Clean up: restore original value or remove if it wasn't set
        match original_value {
            Some(val) => env::set_var("APP_CONF", val),
            None => env::remove_var("APP_CONF"),
        }
    }

    #[test]
    fn test_no_env_var() {
        let _guard = ENV_MUTEX.lock().unwrap();
        
        // Store original value
        let original_value = env::var("APP_CONF").ok();
        
        // Ensure variable is not set
        env::remove_var("APP_CONF");
        
        // Test the configuration
        let config = Config::new();
        assert_eq!(config.get_app_conf_path(), None);
        
        // Clean up: restore original value if it existed
        if let Some(val) = original_value {
            env::set_var("APP_CONF", val);
        }
    }

    #[test]
    fn test_multiple_env_vars() {
        let _guard = ENV_MUTEX.lock().unwrap();
        
        // Store original values
        let original_app_conf = env::var("APP_CONF").ok();
        let original_other_var = env::var("OTHER_VAR").ok();
        
        // Set test values
        env::set_var("APP_CONF", "/test/path.conf");
        env::set_var("OTHER_VAR", "test_value");
        
        // Test the configuration
        let config = Config::new();
        assert_eq!(config.get_app_conf_path(), Some("/test/path.conf"));
        
        // Clean up: restore original values
        match original_app_conf {
            Some(val) => env::set_var("APP_CONF", val),
            None => env::remove_var("APP_CONF"),
        }
        match original_other_var {
            Some(val) => env::set_var("OTHER_VAR", val),
            None => env::remove_var("OTHER_VAR"),
        }
    }
}