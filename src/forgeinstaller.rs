use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize)]
#[allow(non_snake_case)]
pub struct ForgeClientManifest {
    pub id: String,
    pub inheritsFrom: String,
    pub mainClass: String,
    pub libraries: Vec<ForgeLibrary>,
    pub arguments: ForgeArguments,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ForgeArguments {
    pub game: Vec<String>,
    pub jvm: Vec<String>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct ForgeLibrary {
    pub name: String,
    pub downloads: Option<ForgeLibraryDownloads>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct ForgeLibraryDownloads {
    pub artifact: ForgeArtifact,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct ForgeArtifact {
    pub path: String,
    pub url: String,
    pub sha1: String,
    pub size: Option<u64>,
}
