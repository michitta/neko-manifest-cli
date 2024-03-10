use clap::Parser;
use std::{collections::HashSet, path::Path};
use tokio::{
    fs::{self, create_dir, File},
    io::AsyncWriteExt,
};
use types::{
    Cli, FabricLibraries, FabricManifest, LibraryObject, MojangClientManifest,
    MojangVersionManifest, NekoManifest, OsType,
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
        println!("Starting fabric manifest creation...");
        create_fabric_manifest(args.server_name, args.loader_version, args.mc_version)
            .await
            .unwrap();
    } else if args.loader == "forge" {
        println!("Starting forge manifest creation...");
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
            .await
            .unwrap();

    let version_manifest = get_version_manifest
        .versions
        .into_iter()
        .find(|v| v.id == mc_version)
        .expect("Version not found");

    let mojang_manifest = reqwest::get(&version_manifest.url)
        .await?
        .json::<MojangClientManifest>()
        .await
        .unwrap();

    let fabric_manifest = reqwest::get(format!(
        "https://meta.fabricmc.net/v2/versions/loader/{}/{}/profile/json",
        mc_version, fabric_version
    ))
    .await?
    .json::<FabricManifest>()
    .await
    .unwrap();

    let mut libraries = HashSet::new();

    for lib in &fabric_manifest.libraries {
        let lib_obj = LibraryObject {
            path: format!("libraries/{}", resolve_maven(&lib.name)),
            os: vec![OsType::Windows, OsType::Linux, OsType::MacOs],
        };

        libraries.insert(lib_obj);
    }

    for lib in &mojang_manifest.libraries {
        if lib.downloads.artifact.path.is_none() {
            continue;
        }

        let mut oss = HashSet::new();
        oss.extend([OsType::Windows, OsType::Linux, OsType::MacOs]);

        if !lib.rules.is_none() {
            for rule in lib.rules.as_ref().unwrap() {
                if rule.action == "disallow" {
                    oss.remove(&rule.os.name);
                }
            }
        }

        let includes_windows = lib
            .downloads
            .artifact
            .path
            .clone()
            .unwrap()
            .contains("windows");
        let includes_linux = lib
            .downloads
            .artifact
            .path
            .clone()
            .unwrap()
            .contains("linux");
        let includes_osx = lib
            .downloads
            .artifact
            .path
            .clone()
            .unwrap()
            .contains("macos");

        if includes_windows {
            oss.remove(&OsType::Linux);
            oss.remove(&OsType::MacOs);
        } else if includes_linux {
            oss.remove(&OsType::MacOs);
            oss.remove(&OsType::Windows);
        } else if includes_osx {
            oss.remove(&OsType::Linux);
            oss.remove(&OsType::Windows);
        }

        libraries.insert(LibraryObject {
            path: format!("libraries/{}", resolve_maven(&lib.name)),
            os: oss.into_iter().collect(),
        });
    }

    let mojang_libs = mojang_manifest
        .libraries
        .into_iter()
        .map(|lib| FabricLibraries {
            name: lib.name.clone(),
            url: lib.downloads.artifact.url.clone(),
            sha1: Some(lib.downloads.artifact.sha1.clone()),
        });

    let fabric_libs = fabric_manifest
        .libraries
        .into_iter()
        .map(|lib| FabricLibraries {
            name: lib.name.clone(),
            url: format!("{}{}", lib.url, resolve_maven(&lib.name)),
            sha1: Some("".to_string()),
        });

    let libs = mojang_libs
        .chain(fabric_libs)
        .collect::<Vec<FabricLibraries>>();

    // Проверка наличия папки, если нет, то создаём
    if !Path::new(&server_name).exists() {
        create_dir(&server_name).await?;
    }

    for lib in &libs {
        let normal_path = resolve_maven(&lib.name);

        let path = format!("{}/libraries/{}", server_name, normal_path);

        let file_path: &Path = Path::new(&path).parent().unwrap();

        if !file_path.exists() {
            std::fs::create_dir_all(file_path).unwrap();
        }

        let res = reqwest::get(&lib.url)
            .await?
            .bytes()
            .await
            .expect("Failed to get file");

        let mut file = fs::File::create(&path)
            .await
            .expect("Failed to create file");

        file.write_all(&res).await?;

        file.flush().await?;

        println!("{}: {} -> {:?}", lib.name, lib.url, file_path);
    }

    let jvm = vec![
        "-XX:+IgnoreUnrecognizedVMOptions".to_string(),
        "-XX:HeapDumpPath=MojangTricksIntelDriversForPerformance_javaw.exe_minecraft.exe.heapdump"
            .to_string(),
        "-XX:+DisableAttachMechanism".to_string(),
        "-XX:+UnlockExperimentalVMOptions".to_string(),
        "-Xss1M".to_string(),
        "-XX:+UseG1GC".to_string(),
        "-XX:G1NewSizePercent=20".to_string(),
        "-XX:G1ReservePercent=20".to_string(),
        "-XX:MaxGCPauseMillis=50".to_string(),
        "-XX:G1HeapRegionSize=32M".to_string(),
    ];

    let mut manifest = File::create(format!("{}/manifest.json", server_name))
        .await
        .expect("Failed to create manifest");

    libraries.insert({
        LibraryObject {
            path: "minecraft.jar".to_owned(),
            os: vec![OsType::Windows, OsType::Linux, OsType::MacOs],
        }
    });

    let neko_manifest = NekoManifest {
        mainclass: fabric_manifest.mainClass,
        assetIndex: mojang_manifest.assetIndex.id,
        libraries,
        jvm,
        game: fabric_manifest.arguments.game,
        verify: vec![
            "mods".to_string(),
            "libraries".to_string(),
            "minecraft.jar".to_string(),
        ],
        ignore: vec!["options.txt".to_string()],
    };

    let manifest_json =
        serde_json::to_string(&neko_manifest).expect("Failed to serialize manifest");

    manifest
        .write_all(manifest_json.as_bytes())
        .await
        .expect("Failed to write manifest to file");

    Ok(())
}
