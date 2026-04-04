//! Data models for the ustore package registry schema.

use serde::{Deserialize, Serialize};
use std::fmt;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SourceRegistry {
    pub schema_version: String,
    pub repository: Repository,
    pub categories: Vec<Category>,
    pub packages: Vec<Package>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Repository {
    pub name: String,
    pub maintainer: String,
    pub url: String,
    pub license: String,
    pub last_updated: String,
    pub total_packages: u32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Category {
    pub id: String,
    pub name: String,
    pub icon: String,
    pub description: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Package {
    pub id: String,
    pub name: String,
    pub description: String,
    pub category: String,
    pub icon: String,
    pub website: String,
    pub publisher: String,
    pub license: String,
    pub tags: Vec<String>,
    pub verified: bool,
    pub added_date: String,
    pub updated_date: String,
    pub popularity: u8,
    pub variants: Vec<Variant>,
    pub dependencies: Vec<String>,
    pub dpkg_name: Option<String>,
    pub binary_name: Option<String>,
    pub desktop_entry: Option<String>,
    pub post_install: Vec<String>,
    pub pre_remove: Vec<String>,
    pub auto_update: bool,
    pub notes: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub struct Variant {
    pub version: String,
    pub arch: String,
    #[serde(rename = "type")]
    pub pkg_type: String,
    pub url: String,
    pub sha256: Option<String>,
    pub size_mb: u32,
    pub min_ubuntu: Option<String>,
}

impl fmt::Display for Package {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let verified = if self.verified { "✔" } else { " " };
        let version = self
            .variants
            .first()
            .map(|v| v.version.as_str())
            .unwrap_or("unknown");
        write!(
            f,
            "[{verified}] {name} ({version}) — {desc}",
            name = self.name,
            desc = self.description
        )
    }
}
