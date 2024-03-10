use std::collections::HashSet;

use clap::Parser;
use serde::{Deserialize, Serialize};

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
#[allow(non_snake_case)]
pub struct MojangVersions {
    pub id: String,
    pub r#type: String,
    pub url: String,
    pub time: String,
    pub releaseTime: String,
    pub sha1: String,
    pub complianceLevel: u64,
}

#[derive(Deserialize)]
#[allow(non_snake_case)]
pub struct FabricManifest {
    pub mainClass: String,
    pub arguments: FabricArguments,
    pub libraries: Vec<FabricLibraries>,
}

#[derive(Deserialize)]
pub struct FabricArguments {
    pub game: Vec<String>,
    pub jvm: Vec<String>,
}

#[derive(Serialize, Deserialize)]
pub struct FabricLibraries {
    pub name: String,
    pub url: String,
    pub sha1: Option<String>,
}

#[derive(Deserialize)]
#[allow(non_snake_case)]
pub struct MojangClientManifest {
    pub assetIndex: MojangAssetIndex,
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
    pub rules: Option<Vec<MojangLibraryRule>>,
}

#[derive(Deserialize)]
pub struct MojangLibraryDownloads {
    pub artifact: MojangArtifact,
    // #[serde(rename = "classifiers")]
    // pub classifiers: HashMap<String, MojangArtifact>,
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

#[derive(Serialize, Deserialize, PartialEq, Eq, Hash)]
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
    pub path: Option<String>,
    pub sha1: String,
    pub size: u64,
    pub url: String,
}

#[derive(Deserialize)]
pub struct MojangDownloads {
    pub client: MojangArtifact,
    pub client_mappings: MojangArtifact,
}

#[derive(Serialize)]
#[allow(non_snake_case)]
pub struct NekoManifest {
    pub mainclass: String,
    pub assetIndex: String,
    pub libraries: HashSet<LibraryObject>,
    pub jvm: Vec<String>,
    pub game: Vec<String>,
    pub verify: Vec<String>,
    pub ignore: Vec<String>,
}

#[derive(Serialize, Deserialize, PartialEq, Eq, Hash)]
pub struct LibraryObject {
    pub path: String,
    pub os: Vec<OsType>,
}

#[derive(Deserialize)]
#[allow(non_snake_case)]
pub struct ForgeManifest {
    pub mainClass: String,
    pub libraries: Vec<MojangLibrary>,
    pub minecraftArguments: String,
    pub arguments: FabricArguments,
}
