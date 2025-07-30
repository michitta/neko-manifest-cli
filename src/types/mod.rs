use std::collections::{HashMap, HashSet};

use clap::Parser;
use serde::{Deserialize, Serialize};

// Cli types
#[derive(Parser)]
pub struct Cli {
    pub server_name: String,
    pub loader: String,
    pub loader_version: String,
    pub mc_version: String,
    pub download_java: Option<String>,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct Libraries {
    pub name: String,
    pub url: String,
    pub sha1: Option<String>,
}

pub struct MojangResult {
    pub libraries: Vec<Libraries>,
    pub hash_libs: HashSet<LibraryObject>,
    pub asset_index: String,
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
    pub url: String,
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
    pub url: String,
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

#[derive(Debug, Deserialize, Serialize)]
#[serde(rename_all = "kebab-case")]
pub struct JavaRuntime {
    pub linux: PlatformVersions,
    pub mac_os: PlatformVersions,
    pub mac_os_arm64: PlatformVersions,
    pub windows_x64: PlatformVersions,
    pub windows_arm64: PlatformVersions,
}

#[derive(Debug, Deserialize, Serialize)]
#[serde(rename_all = "kebab-case")]
pub struct PlatformVersions {
    pub java_runtime_alpha: Vec<VersionDetails>,
    pub java_runtime_beta: Vec<VersionDetails>,
    pub java_runtime_delta: Vec<VersionDetails>,
    pub java_runtime_gamma: Vec<VersionDetails>,
    pub java_runtime_gamma_snapshot: Vec<VersionDetails>,
    pub jre_legacy: Vec<VersionDetails>,
    pub minecraft_java_exe: Vec<VersionDetails>,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct VersionDetails {
    pub availability: Availability,
    pub manifest: Manifest,
    pub version: Version,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct Availability {
    pub group: u32,
    pub progress: u32,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct Manifest {
    pub sha1: String,
    pub size: u32,
    pub url: String,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct Version {
    pub name: String,
    pub released: String,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct SelectedJavaManifest {
    pub files: HashMap<String, FileEntry>,
}

#[derive(Debug, Deserialize, Serialize)]
#[serde(tag = "type")]
pub enum FileEntry {
    #[serde(rename = "directory")]
    Directory,

    #[serde(rename = "file")]
    File {
        downloads: DownloadVariants,
        #[serde(default)]
        executable: bool,
    },

    #[serde(other)]
    Ignored, // Игнорируем все неизвестные значения
}

#[derive(Debug, Deserialize, Serialize)]
pub struct DownloadVariants {
    pub lzma: Option<DownloadInfo>,
    pub raw: DownloadInfo,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct DownloadInfo {
    pub sha1: String,
    pub size: u64,
    pub url: String,
}
