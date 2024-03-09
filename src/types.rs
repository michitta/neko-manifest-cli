use std::collections::HashMap;

use clap::Parser;
use serde::Deserialize;

// Cli types
#[derive(Parser)]
pub struct Cli {
    pub server_name: String,
    pub loader: String,
    pub loader_version: String,
    pub mc_version: String,
}

// Tools types

#[derive(Deserialize)]
pub struct MojangVersionManifest {
    pub versions: Vec<MojangVersions>,
}

#[derive(Deserialize)]
pub struct MojangVersions {
    pub id: String,
    pub r#type: String,
    pub url: String,
    pub time: String,
    pub release_time: String,
    pub sha1: String,
    pub compliance_level: u64,
}

#[derive(Deserialize)]
pub struct FabricManifest {
    pub main_class: String,
    pub arguments: Vec<FabricArguments>,
    pub libraries: Vec<FabricLibraries>,
}

#[derive(Deserialize)]
pub struct FabricArguments {
    pub game: Vec<String>,
    pub jvm: Vec<String>,
}

#[derive(Deserialize)]
pub struct FabricLibraries {
    pub name: String,
    pub url: String,
    pub sha1: String,
}

pub struct AssetIndex {
    pub id: String,
}

#[derive(Deserialize)]
pub struct MojangClientManifest {
    pub asset_index: MojangAssetIndex,
    pub libraries: Vec<MojangLibrary>,
    pub downloads: MojangDownloads,
}

#[derive(Deserialize)]
pub struct MojangAssetIndex {
    pub id: String,
}

#[derive(Deserialize)]
pub struct MojangLibrary {
    pub downloads: MojangLibraryDownloads,
    pub name: String,
    pub rules: Vec<MojangLibraryRule>,
}

#[derive(Deserialize)]
pub struct MojangLibraryDownloads {
    pub artifact: MojangArtifact,
    #[serde(rename = "classifiers")]
    pub classifiers: HashMap<String, MojangArtifact>,
}

#[derive(Deserialize)]
pub struct MojangLibraryRule {
    pub action: String,
    pub os: MojangOs,
}

#[derive(Deserialize)]
pub struct MojangOs {
    pub name: OsType,
}

#[derive(Deserialize, PartialEq, Eq, Hash)]
pub enum OsType {
    #[serde(rename = "osx")]
    MacOs,
    #[serde(rename = "windows")]
    Windows,
    #[serde(rename = "linux")]
    Linux,
}

#[derive(Deserialize)]
pub struct MojangArtifact {
    pub path: String,
    pub sha1: String,
    pub size: u64,
    pub url: String,
}

#[derive(Deserialize)]
pub struct MojangDownloads {
    pub client: MojangArtifact,
    pub client_mappings: MojangArtifact,
}

pub struct NekoManifest {
    pub main_class: String,
    pub libraries: Vec<Library>,
    pub minecraft_arguments: Option<String>,
    pub arguments: Arguments,
}

#[derive(Deserialize)]
struct Library {
    pub name: String,
    pub downloads: Downloads,
}

#[derive(Deserialize)]
struct Downloads {
    pub artifact: Artifact,
}

#[derive(Deserialize)]
struct Artifact {
    pub path: String,
    pub sha1: String,
    pub size: u64,
    pub url: String,
}

#[derive(Deserialize)]
struct Arguments {
    pub game: Vec<String>,
    pub jvm: Vec<String>,
}

#[derive(Deserialize, PartialEq, Eq, Hash)]
pub struct LibraryObject {
    pub path: String,
    pub os: Vec<OsType>,
}
