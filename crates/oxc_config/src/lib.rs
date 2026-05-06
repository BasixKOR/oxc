//! Shared configuration utilities for Oxc command-line tools.

mod discovery;

pub use discovery::{
    ConfigConflict, ConfigDiscovery, ConfigFileNames, DiscoveredConfigFile, is_js_config_path,
};
