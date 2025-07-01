use cosmic::cosmic_config;

use crate::config::OboeteConfig;

/// Flags given to our COSMIC application to use in it's "init" function.
#[derive(Clone, Debug)]
pub struct Flags {
    pub config_handler: Option<cosmic_config::Config>,
    pub config: OboeteConfig,
}

pub fn flags() -> Flags {
    let (config_handler, config) = (OboeteConfig::config_handler(), OboeteConfig::config());

    Flags {
        config_handler,
        config,
    }
}
