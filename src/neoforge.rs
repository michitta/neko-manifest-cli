use std::collections::HashSet;
use std::path::Path;
use tokio::fs::File;
use tokio::io::AsyncWriteExt;

use crate::mojang::parse_mojang;
use crate::types::{LibraryObject, OsType};

use crate::utils::{default_jvm_args, run_loader_installer};
use crate::{resolve_maven, utils::get_loader_install_profile, NekoManifest};

pub async fn create_neoforge_manifest(
    server_name: String,
    loader_version: String,
    mc_version: String,
) -> Result<(), Box<dyn std::error::Error>> {
    let mojang_parsed = parse_mojang(mc_version.clone()).await;

    let neoforge_manifest = get_loader_install_profile("neoforge", &mc_version, &loader_version).await?;

    let mut libraries = HashSet::new();

    libraries.extend(mojang_parsed.hash_libs);

    for lib in &neoforge_manifest.libraries {
        if lib.downloads.is_none() {
            continue;
        }
        let lib_obj = LibraryObject {
            path: format!("libraries/{}", lib.clone().downloads.unwrap().artifact.path),
            os: vec![OsType::Windows, OsType::Linux, OsType::MacOs],
        };

        let maven = resolve_maven(&lib.name);

        let name = maven.split("/").last().unwrap();

        if !neoforge_manifest.arguments.jvm.join(" ").contains(name) {
            libraries.insert(lib_obj);
        }
    }

    for lib in mojang_parsed.libraries{
        let normal_path = resolve_maven(&lib.name);

        let path = format!("{}/{}", server_name, normal_path);

        let file_path: &Path = Path::new(&path).parent().unwrap();

        if !file_path.exists() {
            std::fs::create_dir_all(file_path).unwrap();
        }

        if lib.url.len() == 0 {
            println!(
                "[WHERE IS URL?] {}: {} -> {:?}",
                lib.name, lib.url, file_path
            );
            continue;
        }

        let res = reqwest::get(&lib.url)
            .await?
            .bytes()
            .await
            .expect("Failed to get file");

        let mut file = File::create(&path).await.expect("Failed to create file");

        file.write_all(&res).await?;

        file.flush().await?;

        println!("{}: {} -> {:?}", lib.name, lib.url, file_path);
    }

    run_loader_installer("neoforge", mc_version, loader_version, server_name.clone()).await;

    let mut manifest = File::create(format!("{}/manifest.json", server_name))
        .await
        .expect("Failed to create manifest");

    let mut jvm = default_jvm_args();

    jvm.extend(neoforge_manifest.arguments.jvm);

    let neko_manifest = NekoManifest {
        mainclass: neoforge_manifest.mainClass,
        assetIndex: mojang_parsed.asset_index,
        libraries,
        jvm,
        game: neoforge_manifest.arguments.game,
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
