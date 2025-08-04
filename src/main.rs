use clap::Parser;
use futures::future::join_all;
use std::{path::Path, sync::Arc};
use tokio::{
    fs::{self, create_dir},
    io::AsyncWriteExt,
    sync::Semaphore,
};
use types::{Cli, FileEntry, JavaRuntime, NekoManifest, SelectedJavaManifest};

use forge::create_forge_manifest;

mod forgeinstaller;
mod mojang;
mod types;
mod utils;

// Mod loaders
mod fabric;
mod forge;
mod neoforge;

use crate::{fabric::create_fabric_manifest, neoforge::create_neoforge_manifest};

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
    match args.download_java {
        Some(_) => {
            println!("Downloading java for selected mc version");
            if args.mc_version.parse::<f32>().unwrap() >= 1.16 {
                download_java(21).await.ok();
            } else {
                download_java(17).await.ok();
            }
        }
        None => {}
    }
    println!("----------------------------");

    // Проверка наличия папки, если нет, то создаём
    if !Path::new(&args.server_name).exists() {
        create_dir(args.server_name.clone())
            .await
            .expect("Failed to create client folder");
    }

    if args.loader == "fabric" {
        println!("Starting fabric manifest creation...");
        create_fabric_manifest(
            args.server_name,
            args.loader_version,
            args.mc_version.clone(),
        )
        .await
        .unwrap();
    } else if args.loader == "forge" {
        println!("Starting forge manifest creation...");
        create_forge_manifest(
            args.server_name,
            args.loader_version,
            args.mc_version.clone(),
        )
        .await
        .unwrap();
    } else if args.loader == "neoforge" {
        println!("Starting neoforge manifest creation...");
        create_neoforge_manifest(
            args.server_name,
            args.loader_version,
            args.mc_version.clone(),
        )
        .await
        .unwrap();
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

async fn java_downloader(
    selected_java_manifest: SelectedJavaManifest,
    name: &str,
    platform: &str,
) -> Result<(), Box<dyn std::error::Error>> {
    println!("Called!");
    let semaphore = Arc::new(Semaphore::new(4));

    let mut handles = vec![];

    for (path, entry) in selected_java_manifest.files {
        match entry {
            FileEntry::Directory => {
                println!("[IGNORED] Пустая папка: {}", path);
            }
            FileEntry::File {
                downloads,
                executable,
            } => {
                let permit = Arc::clone(&semaphore).acquire_owned().await.unwrap();
                let path = format!("{}/{}/{}", platform, name, path);
                let url = downloads.raw.url.clone();

                let handle = tokio::spawn(async move {
                    let _permit = permit; // удерживаем семафор до конца задачи

                    let file_path = Path::new(&path).parent().unwrap().to_path_buf();
                    if !file_path.exists() {
                        fs::create_dir_all(&file_path)
                            .await
                            .expect("Failed to create parent directories");
                    }

                    let response = reqwest::get(&url)
                        .await
                        .expect("Failed to download file")
                        .bytes()
                        .await
                        .expect("Failed to read bytes");

                    let mut file = fs::File::create(&path)
                        .await
                        .expect("Failed to create file");

                    file.write_all(&response)
                        .await
                        .expect("Failed to write file");

                    file.flush().await.expect("Failed to flush file");

                    println!("✅ Saved: {} <- {}", path, url);
                });

                handles.push(handle);
            }
            FileEntry::Ignored => {}
        }
    }

    // Дождаться всех задач
    let results = join_all(handles).await;

    // Проверить результат каждой задачи
    for res in results {
        res?; // пропускаем ошибки из задач
    }

    Ok(())
}

async fn download_java(version: u64) -> Result<(), Box<dyn std::error::Error>> {
    let javas_manifest = reqwest::get("https://launchermeta.mojang.com/v1/products/java-runtime/2ec0cc96c44e5a76b9c8b7c39df7210883d12871/all.json").await?
    .json::<JavaRuntime>()
    .await
    .unwrap();

    let mut windows_url = "";
    let mut windows_arm64_url = "";
    let mut mac_os_url = "";
    let mut mac_os_arm64_url = "";
    let mut linux_url = "";
    let mut name = "";

    if version == 17 {
        windows_url = &javas_manifest.windows_x64.java_runtime_gamma[0]
            .manifest
            .url;
        windows_arm64_url = &javas_manifest.windows_arm64.java_runtime_gamma[0]
            .manifest
            .url;
        mac_os_url = &javas_manifest.mac_os.java_runtime_gamma[0].manifest.url;
        mac_os_arm64_url = &javas_manifest.mac_os_arm64.java_runtime_gamma[0]
            .manifest
            .url;
        linux_url = &javas_manifest.linux.java_runtime_gamma[0].manifest.url;
        name = "java_runtime_gamma"
    } else {
        windows_url = &javas_manifest.windows_x64.java_runtime_delta[0]
            .manifest
            .url;
        windows_arm64_url = &javas_manifest.windows_arm64.java_runtime_delta[0]
            .manifest
            .url;
        mac_os_url = &javas_manifest.mac_os.java_runtime_delta[0].manifest.url;
        mac_os_arm64_url = &javas_manifest.mac_os_arm64.java_runtime_delta[0]
            .manifest
            .url;
        linux_url = &javas_manifest.linux.java_runtime_delta[0].manifest.url;
        name = "java_runtime_delta"
    }

    let windows = reqwest::get(windows_url)
        .await?
        .json::<SelectedJavaManifest>()
        .await
        .unwrap();
    let windows_arm64 = reqwest::get(windows_arm64_url)
        .await?
        .json::<SelectedJavaManifest>()
        .await
        .unwrap();
    let mac_os = reqwest::get(mac_os_url)
        .await?
        .json::<SelectedJavaManifest>()
        .await
        .unwrap();
    let mac_os_arm64 = reqwest::get(mac_os_arm64_url)
        .await?
        .json::<SelectedJavaManifest>()
        .await
        .unwrap();
    let linux = reqwest::get(linux_url)
        .await?
        .json::<SelectedJavaManifest>()
        .await
        .unwrap();

    let platforms = vec![
        ("windows", windows),
        ("windows_arm64", windows_arm64),
        ("mac_os", mac_os),
        ("mac_os_arm64", mac_os_arm64),
        ("linux", linux),
    ];

    for (platform_name, platform) in platforms {
        match java_downloader(platform, name, platform_name).await {
            Ok(_) => println!("Successfully downloaded for {}", platform_name),
            Err(e) => eprintln!("Error downloading for {}: {:?}", platform_name, e),
        }
    }

    Ok(())
}
