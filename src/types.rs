use std::collections::HashMap;

use serde::Deserialize;

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
    id: String,
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

#[derive(Deserialize)]
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
    main_class: String,
    libraries: Vec<Library>,
    minecraft_arguments: Option<String>,
    arguments: Arguments,
}

#[derive(Deserialize)]
struct Library {
    name: String,
    downloads: Downloads,
}

#[derive(Deserialize)]
struct Downloads {
    artifact: Artifact,
}

#[derive(Deserialize)]
struct Artifact {
    path: String,
    sha1: String,
    size: u64,
    url: String,
}

#[derive(Deserialize)]
struct Arguments {
    game: Vec<String>,
    jvm: Vec<String>,
}
