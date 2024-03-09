use clap::Parser;
use std::{collections::HashSet, fs::create_dir, path::Path};
use types::{
    Cli, FabricLibraries, FabricManifest, LibraryObject, MojangClientManifest,
    MojangVersionManifest, OsType,
};

mod types;

#[tokio::main]
async fn main() {
    let args = Cli::parse();
    println!("Welcome to Neko Manifest CLI!");
    println!("Version: {}", env!("CARGO_PKG_VERSION"));
    println!("----------------------------");
    println!("Server name: {}", args.server_name);
    println!("Selected loader: {}", args.loader);
    println!("Selected loader version: {}", args.loader_version);
    println!("Selected mc version: {}", args.mc_version);
    println!("----------------------------");

    if args.loader == "fabric" {
        println!("Creating fabric manifest...");
        let _ =
            create_fabric_manifest(args.server_name, args.loader_version, args.mc_version).await;
    } else if args.loader == "forge" {
        println!("Creating forge manifest...");
    } else {
        println!("Supported loader not found");
    }
}

fn resolve_maven(file: &str) -> String {
    let mut parts = file.split(':');
    let path = parts.next().expect("No path").replace(".", "/");
    let name = parts.next().expect("No name");
    let version = parts.next().expect("No version");
    let mut filename = format!("{}/{}/{}/{}-{}.jar", path, name, version, name, version);
    if let Some(more) = parts.next() {
        filename.push_str(&format!("-{}", more));
    }
    filename
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

    let mut libraries = HashSet::new();
    for lib in &fabric_manifest.libraries {
        let lib_obj = LibraryObject {
            path: format!("libraries/{}", resolve_maven(&lib.name)),
            os: vec![OsType::Windows, OsType::Linux, OsType::MacOs],
        };

        libraries.insert(lib_obj);
    }

    let mojang_libs = mojang_manifest
        .libraries
        .into_iter()
        .map(|lib| FabricLibraries {
            name: lib.name.clone(),
            url: lib.downloads.artifact.url.clone(),
            sha1: lib.downloads.artifact.sha1.clone(),
        });

    let fabric_libs = fabric_manifest
        .libraries
        .into_iter()
        .map(|lib| FabricLibraries {
            name: lib.name.clone(),
            url: format!("{}{}", lib.url, resolve_maven(&lib.name)),
            sha1: "".to_string(),
        });

    let libs = mojang_libs.chain(fabric_libs);

    // Проверка наличия папки, если нет, то создаём
    if !Path::new(&server_name).exists() {
        create_dir(&server_name)?;
    }

    for lib in libs {
        let normal_path = resolve_maven(&lib.name);

        let path = format!("{}/libraries/{}", server_name, normal_path);

        let file_path: &Path = Path::new(&path).parent().unwrap();

        if !file_path.exists() {
            std::fs::create_dir_all(file_path).unwrap();
        }

        println!("{}: {} -> {:?}", lib.name, lib.url, file_path);
    }

    Ok(())
}
