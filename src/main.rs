use std::collections::HashMap;

use reqwest::block;
use types::{FabricManifest, MojangClientManifest, MojangVersionManifest};

mod types;

fn main() {
    println!("Welcome to Neko Manifest Creator!");

    let args: Vec<String> = std::env::args().collect();
    println!("----------------------------");
    println!("Selected loader: {}", args[2]);
    println!("Selected version: {}", args[3]);
    println!("----------------------------");
}

async fn create_fabric_manifest(
    server_name: String,
    fabric_version: String,
    mc_version: String,
) -> Result<(), Box<dyn std::error::Error>> {
    let get_version_manifest =
        reqwest::get("https://piston-meta.mojang.com/mc/game/version_manifest_v2.json")
            .await?
            .json::<MojangVersionManifest>()
            .await?;

    let version_manifest = get_version_manifest
        .versions
        .into_iter()
        .find(|v| v.id == mc_version)
        .expect("Version not found");

    let mojang_manifest = reqwest::get(&version_manifest.url)
        .await?
        .json::<MojangClientManifest>()
        .await?;

    let fabric_manifest = reqwest::get(format!(
        "https://meta.fabricmc.net/v2/versions/loader/{mc_version}/{fabric_version}/profile/json"
    ))
    .await?
    .json::<FabricManifest>()
    .await?;

    let mut libraries = HashMap::new();

    fabric_manifest.libraries.into_iter().for_each(|lib| {
        let lib_obj =
    })

    Ok(())
}
