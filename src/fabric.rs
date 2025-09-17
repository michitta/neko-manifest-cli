use std::{collections::HashSet, path::Path};

use tokio::{fs::File, io::AsyncWriteExt};

use crate::{
    mojang::parse_mojang,
    types::{FabricManifest, Libraries, LibraryObject, NekoManifest, OsType},
    utils::{default_jvm_args, resolve_maven},
};

pub async fn create_fabric_manifest(
    server_name: String,
    fabric_version: String,
    mc_version: String,
) -> Result<(), Box<dyn std::error::Error>> {
    let mojang_parsed = parse_mojang(mc_version.clone()).await;

    let fabric_manifest = reqwest::get(format!(
        "https://meta.fabricmc.net/v2/versions/loader/{}/{}/profile/json",
        mc_version, fabric_version
    ))
    .await?
    .json::<FabricManifest>()
    .await
    .unwrap();

    let mut libraries = HashSet::new();

    libraries.extend(mojang_parsed.hash_libs);

    for lib in &fabric_manifest.libraries {
        let lib_obj = LibraryObject {
            path: format!("libraries/{}", resolve_maven(&lib.name)),
            os: vec![OsType::Windows, OsType::Linux, OsType::MacOs],
        };

        libraries.insert(lib_obj);
    }

    let fabric_libs = fabric_manifest.libraries.into_iter().map(|lib| Libraries {
        name: lib.name.clone(),
        url: format!("{}{}", lib.url, resolve_maven(&lib.name)),
        sha1: Some("".to_string()),
    });

    let libs = mojang_parsed
        .libraries
        .into_iter()
        .chain(fabric_libs)
        .collect::<Vec<Libraries>>();

    for lib in &libs {
        let normal_path = resolve_maven(&lib.name);

        let path = format!("{}/{}", server_name, normal_path);

        let file_path: &Path = Path::new(&path).parent().unwrap();

        if !file_path.exists() {
            std::fs::create_dir_all(file_path).unwrap();
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

    let mut manifest = File::create(format!("{}/manifest.json", server_name))
        .await
        .expect("Failed to create manifest");

    let mut jvm = default_jvm_args();

    jvm.extend(fabric_manifest.arguments.jvm);

    let neko_manifest = NekoManifest {
        mainclass: fabric_manifest.mainClass,
        assetIndex: mojang_parsed.asset_index,
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
