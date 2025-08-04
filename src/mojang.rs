use std::collections::HashSet;

use crate::{
    resolve_maven,
    types::{
        Libraries, LibraryObject, MojangClientManifest, MojangResult, MojangVersionManifest, OsType,
    },
};

pub async fn parse_mojang(game_version: String) -> MojangResult {
    let get_version_manifest =
        reqwest::get("https://piston-meta.mojang.com/mc/game/version_manifest_v2.json")
            .await
            .unwrap()
            .json::<MojangVersionManifest>()
            .await
            .unwrap();

    let version_manifest = get_version_manifest
        .versions
        .into_iter()
        .find(|v| v.id == game_version)
        .expect("Version not found");

    let mojang_manifest = reqwest::get(&version_manifest.url)
        .await
        .unwrap()
        .json::<MojangClientManifest>()
        .await
        .unwrap();

    let mut hash_libs = HashSet::new();

    hash_libs.insert(LibraryObject {
        path: "minecraft.jar".to_owned(),
        os: [OsType::Windows, OsType::Linux, OsType::MacOs].to_vec(),
    });
    
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

        let mut os_vec = Vec::new();

        if oss.contains(&OsType::Windows) {
            os_vec.push(OsType::Windows);
        }
        if oss.contains(&OsType::Linux) {
            os_vec.push(OsType::Linux);
        }
        if oss.contains(&OsType::MacOs) {
            os_vec.push(OsType::MacOs);
        }

        hash_libs.insert(LibraryObject {
            path: format!("{}", resolve_maven(&lib.name)),
            os: os_vec,
        });
    }

    let mut mojang_libs: Vec<Libraries> = mojang_manifest
        .libraries
        .into_iter()
        .map(|lib| Libraries {
            name: lib.name.clone(),
            url: lib.downloads.artifact.url.clone(),
            sha1: Some(lib.downloads.artifact.sha1.clone()),
        })
        .collect();

    mojang_libs.push(Libraries {
        name: "minecraft.jar".to_owned(),
        url: mojang_manifest.downloads.client.url,
        sha1: Some(mojang_manifest.downloads.client.sha1),
    });

    MojangResult {
        libraries: mojang_libs,
        hash_libs,
        asset_index: mojang_manifest.assetIndex.id,
    }
}
