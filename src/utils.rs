use std::{
    fs::{self, create_dir_all, File},
    io::{Cursor, Write},
    path::Path,
    process::Command,
};

use reqwest::Client;
use zip::ZipArchive;

use crate::forgeinstaller::ForgeClientManifest;

pub fn resolve_maven(maven: &str) -> String {
    let parts: Vec<&str> = maven.split(':').collect();

    if parts.len() < 3 {
        panic!("Invalid maven coordinate: {}", maven);
    }

    let group = parts[0].replace('.', "/");
    let artifact = parts[1];
    let version = parts[2];

    let classifier = if parts.len() == 4 {
        format!("-{}", parts[3])
    } else {
        String::new()
    };

    format!(
        "{}/{}/{}/{}-{}{}.jar",
        group, artifact, version, artifact, version, classifier
    )
}

pub fn default_jvm_args() -> Vec<String> {
    vec![
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
    ]
}

pub async fn get_forge_install_profile(
    mc_version: &str,
    forge_version: &str,
) -> Result<ForgeClientManifest, Box<dyn std::error::Error>> {
    let url = format!(
        "https://maven.minecraftforge.net/net/minecraftforge/forge/{mc}-{forge}/forge-{mc}-{forge}-installer.jar",
        mc = mc_version,
        forge = forge_version
    );

    let client = Client::new();
    let resp = client.get(&url).send().await?.bytes().await?;
    let reader = Cursor::new(resp);
    let mut archive = ZipArchive::new(reader)?;

    let mut version = String::new();
    {
        let mut version_file = archive.by_name("version.json")?;
        use std::io::Read;
        version_file.read_to_string(&mut version)?;
    }

    // let mut profile = String::new();
    // {
    //     let mut profile_file = archive.by_name("install_profile.json")?;
    //     use std::io::Read;
    //     profile_file.read_to_string(&mut profile)?;
    // }

    let version_json: ForgeClientManifest = serde_json::from_str(&version)?;
    // let profile_json: ForgeInstallProfile = serde_json::from_str(&profile)?;

    let result: ForgeClientManifest = ForgeClientManifest {
        id: version_json.id,
        inheritsFrom: version_json.inheritsFrom,
        mainClass: version_json.mainClass,
        libraries: version_json.libraries,
        arguments: version_json.arguments,
    };

    Ok(result)
}

pub async fn run_forge_installer(mc_version: String, loader_version: String, server_name: String) {
    let url = format!(
        "https://maven.minecraftforge.net/net/minecraftforge/forge/{mc}-{forge}/forge-{mc}-{forge}-installer.jar",
        mc = mc_version,
        forge = loader_version
    );

    let client = Client::new();
    let resp = client
        .get(&url)
        .send()
        .await
        .expect("Failed to download installer")
        .bytes()
        .await
        .expect("Failed to read installer bytes");

    let installer_path = format!("{server}/forge-installer.jar", server = server_name.clone());
    let installer_path_ref = Path::new(&installer_path);

    // Ensure the server directory exists
    create_dir_all(server_name.clone()).expect("Failed to create target directory");

    let profiles_path = Path::new(&server_name).join("launcher_profiles.json");

    let json = r#"{
      "profiles": {},
      "settings": {
        "crashAssistance": false,
        "enableAdvanced": false,
        "enableAnalytics": true,
        "enableHistorical": false,
        "enableReleases": true,
        "enableSnapshots": false,
        "keepLauncherOpen": false,
        "profileSorting": "ByLastPlayed",
        "showGameLog": true,
        "showMenu": false,
        "soundOn": false
      },
      "version": 4
    }"#;

    let mut profiles_file = File::create(profiles_path.clone()).unwrap();
    profiles_file
        .write_all(json.as_bytes())
        .expect("Failed to write profiles json");

    // Write the installer to disk
    let mut file = File::create(installer_path_ref).expect("Failed to create installer file");
    std::io::copy(&mut Cursor::new(resp), &mut file).expect("Failed to write installer file");

    // Run the installer
    let status = Command::new("java")
        .arg("-Dminecraft.home=.")
        .arg("-jar")
        .arg("forge-installer.jar")
        .arg("--installClient")
        .current_dir(server_name.clone())
        .status()
        .expect("Failed to run installer");

    if !status.success() {
        panic!("Forge installer failed with status: {}", status);
    }

    println!("Cleanup");

    fs::remove_file(profiles_path).unwrap();
    fs::remove_file(installer_path).unwrap();
    fs::remove_file(Path::new(&server_name).join("installer.log")).unwrap();
    fs::remove_dir_all(Path::new(&server_name).join("versions")).unwrap();
}
