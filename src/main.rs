use clap::Parser;
use futures::future::join_all;
use std::{any::{self, Any}, collections::HashSet, fmt::Debug, path::Path, sync::Arc};
use tokio::{
    fs::{self, create_dir, File},
    io::AsyncWriteExt, sync::Semaphore,
};
use types::{
    Cli, FabricLibraries, FabricManifest, FileEntry, JavaRuntime, LibraryObject, MojangClientManifest, MojangVersionManifest, NekoManifest, OsType, PlatformVersions, SelectedJavaManifest
};

mod types;
mod downloader;

#[tokio::main]
async fn main() {
    download_java(21).await.ok();
    download_java(17).await.ok();


    // let args = Cli::parse();
    // println!("Welcome to Neko Manifest CLI!");
    // println!("Version: {}", env!("CARGO_PKG_VERSION"));
    // println!("----------------------------");
    // println!("Server name: {}", args.server_name);
    // println!("Selected loader: {}", args.loader);
    // println!("Selected loader version: {}", args.loader_version);
    // println!("Selected mc version: {}", args.mc_version);
    // match args.java_version {
    //     Some(arg) => println!("Downloading java version: {}", arg),
    //     None => {}
    // }
    // println!("----------------------------");

    // if args.loader == "fabric" {
    //     println!("Starting fabric manifest creation...");
    //     create_fabric_manifest(args.server_name, args.loader_version, args.mc_version)
    //         .await
    //         .unwrap();
    // } else if args.loader == "forge" {
    //     println!("Starting forge manifest creation...");
    // } else {
    //     println!("Supported loader not found");
    // }
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

async fn java_downloader(selected_java_manifest: SelectedJavaManifest, name: &str, platform: &str)-> Result<(), Box<dyn std::error::Error>> {
    println!("Called!");
    let semaphore = Arc::new(Semaphore::new(4));

    let mut handles = vec![];

    for (path, entry) in selected_java_manifest.files {
        match entry {
            FileEntry::Directory => {
                println!("[IGNORED] Пустая папка: {}", path);
            }
            FileEntry::File { downloads, executable } => {
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

async fn download_java(version: u64) -> Result<(), Box<dyn std::error::Error>>  {
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
        windows_url = &javas_manifest.windows_x64.java_runtime_gamma[0].manifest.url;
        windows_arm64_url = &javas_manifest.windows_arm64.java_runtime_gamma[0].manifest.url;
        mac_os_url = &javas_manifest.mac_os.java_runtime_gamma[0].manifest.url;
        mac_os_arm64_url = &javas_manifest.mac_os_arm64.java_runtime_gamma[0].manifest.url;
        linux_url = &javas_manifest.linux.java_runtime_gamma[0].manifest.url;
        name = "java_runtime_gamma"
    } else {
        windows_url = &javas_manifest.windows_x64.java_runtime_delta[0].manifest.url;
        windows_arm64_url = &javas_manifest.windows_arm64.java_runtime_delta[0].manifest.url;
        mac_os_url = &javas_manifest.mac_os.java_runtime_delta[0].manifest.url;
        mac_os_arm64_url = &javas_manifest.mac_os_arm64.java_runtime_delta[0].manifest.url;
        linux_url = &javas_manifest.linux.java_runtime_delta[0].manifest.url;
        name = "java_runtime_delta"
    }

    let windows = reqwest::get(windows_url).await?.json::<SelectedJavaManifest>().await.unwrap();
    let windows_arm64 = reqwest::get(windows_arm64_url).await?.json::<SelectedJavaManifest>().await.unwrap();
    let mac_os = reqwest::get(mac_os_url).await?.json::<SelectedJavaManifest>().await.unwrap();
    let mac_os_arm64 = reqwest::get(mac_os_arm64_url).await?.json::<SelectedJavaManifest>().await.unwrap();
    let linux = reqwest::get(linux_url).await?.json::<SelectedJavaManifest>().await.unwrap();

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