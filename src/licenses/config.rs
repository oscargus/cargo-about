use serde::{de, Deserialize};
use std::{collections::BTreeMap, fmt, path::PathBuf};

fn deserialize_spdx_id<'de, D>(deserializer: D) -> std::result::Result<spdx::LicenseId, D::Error>
where
    D: de::Deserializer<'de>,
{
    struct Visitor;

    impl<'de> de::Visitor<'de> for Visitor {
        type Value = spdx::LicenseId;

        fn expecting(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
            formatter.write_str("SPDX short-identifier")
        }

        fn visit_str<E>(self, v: &str) -> std::result::Result<Self::Value, E>
        where
            E: de::Error,
        {
            spdx::license_id(v).ok_or_else(|| {
                E::custom(format!(
                    "'{}' is not a valid SPDX short-identifier in v{}",
                    v,
                    spdx::license_version()
                ))
            })
        }
    }

    deserializer.deserialize_any(Visitor)
}

fn deserialize_licensee<'de, D>(
    deserializer: D,
) -> std::result::Result<Vec<spdx::Licensee>, D::Error>
where
    D: de::Deserializer<'de>,
{
    struct Visitor;

    impl<'de> de::Visitor<'de> for Visitor {
        type Value = Vec<spdx::Licensee>;

        fn expecting(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
            formatter.write_str("array of SPDX licensees")
        }

        fn visit_seq<S>(self, mut seq: S) -> Result<Self::Value, S::Error>
        where
            S: de::SeqAccess<'de>,
        {
            let mut vec = Vec::new();

            // Update the max while there are additional values.
            while let Some(v) = seq.next_element()? {
                let lic = spdx::Licensee::parse(v).map_err(|e| {
                    de::Error::custom(format!("'{}' is not a valid SPDX licensee: {}", v, e))
                })?;

                vec.push(lic);
            }

            Ok(vec)
        }
    }

    deserializer.deserialize_seq(Visitor)
}

#[derive(Deserialize, Debug)]
#[serde(rename_all = "kebab-case")]
pub struct Additional {
    pub root: PathBuf,
    #[serde(deserialize_with = "deserialize_spdx_id")]
    pub license: spdx::LicenseId,
    pub license_file: PathBuf,
    pub license_start: Option<usize>,
    pub license_end: Option<usize>,
}

#[derive(Deserialize, Debug)]
#[serde(rename_all = "kebab-case")]
pub struct Ignore {
    #[serde(deserialize_with = "deserialize_spdx_id")]
    pub license: spdx::LicenseId,
    pub license_file: PathBuf,
    pub license_start: Option<usize>,
    pub license_end: Option<usize>,
}

#[derive(Deserialize, Debug)]
pub struct KrateConfig {
    #[serde(default)]
    pub additional: Vec<Additional>,
    #[serde(default)]
    pub ignore: Vec<Ignore>,
}

#[derive(Deserialize, Debug, Default)]
pub struct Config {
    /// The list of licenses we will use for all crates, in priority order
    #[serde(deserialize_with = "deserialize_licensee")]
    pub accepted: Vec<spdx::Licensee>,
    #[serde(flatten)]
    pub inner: BTreeMap<String, KrateConfig>,
}