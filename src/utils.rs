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

pub async fn get_loader_install_profile(
    loader_type: &str,
    mc_version: &str,
    loader_version: &str,
) -> Result<ForgeClientManifest, Box<dyn std::error::Error>> {
    let url = match loader_type {
        "forge" => format!(
            "https://maven.minecraftforge.net/net/minecraftforge/forge/{mc}-{loader}/forge-{mc}-{loader}-installer.jar",
            mc = mc_version,
            loader = loader_version
        ),
        "neoforge" => format!(
            "https://maven.neoforged.net/releases/net/neoforged/neoforge/{loader}/neoforge-{loader}-installer.jar",
            loader = loader_version
        ),
        _ => return Err("Unknown loader type".into()),
    };

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

    let version_json: ForgeClientManifest = serde_json::from_str(&version)?;

    Ok(version_json)
}

pub async fn run_loader_installer(
    loader_type: &str,
    mc_version: String,
    loader_version: String,
    server_name: String,
) {
    let url = match loader_type {
        "forge" => format!(
            "https://maven.minecraftforge.net/net/minecraftforge/forge/{mc}-{loader}/forge-{mc}-{loader}-installer.jar",
            mc = mc_version,
            loader = loader_version
        ),
        "neoforge" => format!(
            "https://maven.neoforged.net/releases/net/neoforged/neoforge/{loader}/neoforge-{loader}-installer.jar",
            loader = loader_version
        ),
        _ => panic!("Unknown loader type"),
    };

    let client = Client::new();
    let resp = client
        .get(&url)
        .send()
        .await
        .expect("Failed to download installer")
        .bytes()
        .await
        .expect("Failed to read installer bytes");

    let installer_name = match loader_type {
        "forge" => "forge-installer.jar",
        "neoforge" => "neoforge-installer.jar",
        _ => "installer.jar",
    };

    let installer_path = format!("{server}/{installer}", server = server_name, installer = installer_name);
    let installer_path_ref = Path::new(&installer_path);

    create_dir_all(&server_name).expect("Failed to create target directory");

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

    let mut file = File::create(installer_path_ref).expect("Failed to create installer file");
    std::io::copy(&mut Cursor::new(resp), &mut file).expect("Failed to write installer file");

    let status = Command::new("java")
        .arg("-Dminecraft.home=.")
        .arg("-jar")
        .arg(installer_name)
        .arg("--installClient")
        .current_dir(server_name.clone())
        .status()
        .expect("Failed to run installer");

    if !status.success() {
        panic!("Installer failed with status: {}", status);
    }

    println!("Cleanup");

    fs::remove_file(profiles_path).unwrap();
    fs::remove_file(installer_path).unwrap();
    fs::remove_file(Path::new(&server_name).join("installer.log")).unwrap();
    if loader_type == "forge" {
        fs::remove_dir_all(Path::new(&server_name).join("versions")).unwrap();
    }
}