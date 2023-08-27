use crate::errors::ScrapliError;
use once_cell::sync::OnceCell;
use serde::{
    Deserialize,
    Serialize,
};
use std::collections::HashMap;

const ARISTA_EOS_PLATFORM_YAML: &str = include_str!("assets/arista_eos.yaml");
const CISCO_IOSXE_PLATFORM_YAML: &str = include_str!("assets/cisco_iosxe.yaml");
const CISCO_IOSXR_PLATFORM_YAML: &str = include_str!("assets/cisco_iosxr.yaml");
const CISCO_NXOS_PLATFORM_YAML: &str = include_str!("assets/cisco_nxos.yaml");
const JUNIPER_JUNOS_PLATFORM_YAML: &str = include_str!("assets/juniper_junos.yaml");
const NOKIA_SRL_PLATFORM_YAML: &str = include_str!("assets/nokia_srl.yaml");
const NOKIA_SROS_PLATFORM_YAML: &str = include_str!("assets/nokia_sros.yaml");

/// Returns a `HashMap` wherein platform names are keys and the included yaml platform (asset) data
/// string is the value.
pub fn get_platforms() -> &'static HashMap<&'static str, &'static str> {
    static PLATFORMS: OnceCell<HashMap<&str, &str>> = OnceCell::new();

    PLATFORMS.get_or_init(|| {
        HashMap::from([
            ("arista_eos:", ARISTA_EOS_PLATFORM_YAML),
            ("cisco_iosxe", CISCO_IOSXE_PLATFORM_YAML),
            ("cisco_iosxr", CISCO_IOSXR_PLATFORM_YAML),
            ("cisco_nxos", CISCO_NXOS_PLATFORM_YAML),
            ("juniper_junos", JUNIPER_JUNOS_PLATFORM_YAML),
            ("nokia_srl", NOKIA_SRL_PLATFORM_YAML),
            ("nokia_sros", NOKIA_SROS_PLATFORM_YAML),
        ])
    })
}

/// An enum representing the valid driver types -- generic or network.
#[derive(Debug, Serialize, Deserialize)]
pub enum DriverType {
    /// The "generic" flavor of driver.
    Generic,
    /// The "network" (one that knows about privilege levels) flavor of driver.
    Network,
}

impl Default for DriverType {
    fn default() -> Self {
        Self::Generic
    }
}

/// `Definition` is an object that holds a platform type, its default flavor and optional variants
/// -- that is option variations that can be merged over top of the default platform definition.
#[derive(Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub struct Definition {
    /// The type of the platform, for example "nokia_srl" or "cisco_iosxe".
    pub platform_type: String,
    /// Default is the default/base platform definition.
    pub default: Platform,
    /// Variants are optional named variants that can be merged over top of the default platform.
    pub variants: HashMap<String, Platform>,
}

/// `Platform` is a struct that contains JSON or YAML data that represent the attributes required to
/// create a generic or network driver to connect to a given device type.
#[derive(Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub struct Platform {
    /// The type of the platform, for example "nokia_srl" or "cisco_iosxe".
    pub platform_type: String,

    /// The driver type for the platform, either "generic" or "network".
    #[serde(skip)]
    driver_type: DriverType,
}

impl Platform {
    /// Returns an instance of `Platform` generated from the given `platform_name`.
    ///
    /// # Errors
    ///
    /// Can error if the platform data can not be serialized.
    pub fn new(platform_name: &str) -> Result<Self, ScrapliError> {
        let platforms = get_platforms();

        platforms.get(platform_name).map_or_else(
            || {
                Err(ScrapliError {
                    details: format!("unknown platform name '{platform_name}'"),
                })
            },
            |platform_str| match serde_yaml::from_str(platform_str) {
                Ok(platform) => Ok(platform),
                Err(err) => Err(ScrapliError {
                    details: format!("failed serializing embedded platform type, error: {err}"),
                }),
            },
        )
    }

    // fn get_generic_driver() -> Result<(), ScrapliError> {}
    //
    // fn get_network_driver() -> Result<(), ScrapliError> {}
}
