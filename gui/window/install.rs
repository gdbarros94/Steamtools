use crate::window::WindowPopup;
use std::{
    io::{self, Cursor, Error, ErrorKind, Write as ioWrite},
    fs::{self, File},
    path::{Path, MAIN_SEPARATOR},
};
use eframe::egui::{self, Window};
use log::{debug, warn};

/// ManifestHub: returns raw .lua directly
const MANIFESTS_URL: &str = "https://raw.githubusercontent.com/SteamAutoCracks/ManifestHub/refs/heads";

/// LuaTools compatible APIs — return a ZIP containing {appid}.lua + optional .manifest files
static LUATOOLS_APIS: &[(&str, &str)] = &[
    ("Ryuu",           "http://167.235.229.108/{appid}"),
    ("TwentyTwo Cloud","https://api.twentytwocloud.com/download?appid={appid}"),
    ("Sushi",          "https://raw.githubusercontent.com/sushi-dev55-alt/sushitools-games-repo-alt/refs/heads/main/{appid}.zip"),
];

pub struct InstallPopup {
    pub active: bool,
    pub appid: String,
    pub status: InstallStatus,
}

#[derive(Default, Debug, PartialEq)]
pub enum InstallStatus {
    #[default]
    Idle,
    Downloading(String), // api name currently trying
    Done,
    AlreadyExists,
    NotFound,
    Error(String),
}

/// Detect the real Steam path, considering both ~/.steam and ~/.steam/debian-installation
fn detect_real_steam_path(user_provided: &str) -> io::Result<String> {
    let user_path = user_provided.trim();
    
    // Try debian-installation variant first (more specific, used by native Steam on Linux)
    let debian_variant = format!("{user_path}{MAIN_SEPARATOR}debian-installation");
    if Path::new(&debian_variant).exists() 
        && Path::new(&format!("{debian_variant}{MAIN_SEPARATOR}steamapps")).exists() {
        debug!("Detected Steam using debian-installation path: {debian_variant}");
        return Ok(debian_variant);
    }
    
    // Fallback to user-provided path
    if Path::new(user_path).exists() 
        && Path::new(&format!("{user_path}{MAIN_SEPARATOR}steamapps")).exists() {
        debug!("Using provided Steam path: {user_path}");
        return Ok(user_path.to_string());
    }
    
    Err(Error::new(ErrorKind::NotFound, format!(
        "No valid Steam installation found in {user_path} or {debian_variant}"
    )))
}

/// Try ManifestHub first (returns raw .lua). If not found, try LuaTools ZIP APIs.
fn install(steam_path: &str, appid: i32) -> io::Result<()> {
    // Detect the real Steam path
    let real_steam_path = detect_real_steam_path(steam_path)?;
    
    let lua_dest = format!("{real_steam_path}{MAIN_SEPARATOR}config{MAIN_SEPARATOR}stplug-in{MAIN_SEPARATOR}{appid}.lua");
    if Path::new(&lua_dest).exists() {
        return Err(Error::new(ErrorKind::AlreadyExists, "Game already exists!"));
    }

    // 1. Try ManifestHub (raw .lua)
    let manifest_url = format!("{MANIFESTS_URL}/{appid}/{appid}.lua");
    if let Ok(resp) = reqwest::blocking::get(&manifest_url) {
        if resp.status().is_success() {
            if let Ok(bytes) = resp.bytes() {
                fs::create_dir_all(Path::new(&lua_dest).parent().unwrap())?;
                let mut f = File::create(&lua_dest)?;
                f.write_all(&bytes)?;
                debug!("ManifestHub -> installed {lua_dest}");
                return Ok(());
            }
        }
    }

    // 2. Try LuaTools ZIP APIs
    for (api_name, url_template) in LUATOOLS_APIS {
        let url = url_template.replace("{appid}", &appid.to_string());
        debug!("Trying API '{api_name}': {url}");

        let resp = match reqwest::blocking::get(&url) {
            Ok(r) => r,
            Err(e) => {
                warn!("API '{api_name}' connection error: {e}");
                continue;
            }
        };

        if !resp.status().is_success() {
            debug!("API '{api_name}' returned {}", resp.status());
            continue;
        }

        let bytes = match resp.bytes() {
            Ok(b) => b,
            Err(e) => {
                warn!("API '{api_name}' failed to read bytes: {e}");
                continue;
            }
        };

        // Validate ZIP magic bytes
        if bytes.len() < 4 || &bytes[..4] != b"PK\x03\x04" {
            debug!("API '{api_name}' did not return a valid ZIP");
            continue;
        }

        match extract_zip_and_install(&bytes, appid, &real_steam_path) {
            Ok(_) => {
                debug!("API '{api_name}' -> installed {lua_dest}");
                return Ok(());
            }
            Err(e) => {
                warn!("API '{api_name}' extraction failed: {e}");
                continue;
            }
        }
    }

    Err(Error::new(ErrorKind::ConnectionRefused, "Not found in any source"))
}

/// Extract .lua and .manifest files from a ZIP returned by LuaTools-compatible APIs.
fn extract_zip_and_install(zip_bytes: &[u8], appid: i32, steam_path: &str) -> io::Result<()> {
    let cursor = Cursor::new(zip_bytes);
    let mut archive = zip::ZipArchive::new(cursor)
        .map_err(|e| Error::new(ErrorKind::InvalidData, e.to_string()))?;

    let lua_dest_dir = format!("{steam_path}{MAIN_SEPARATOR}config{MAIN_SEPARATOR}stplug-in");
    let depot_dir = format!("{steam_path}{MAIN_SEPARATOR}depotcache");
    fs::create_dir_all(&lua_dest_dir)?;

    let mut lua_installed = false;

    // First pass: extract .manifest files into depotcache
    for i in 0..archive.len() {
        let mut entry = archive.by_index(i)
            .map_err(|e| Error::new(ErrorKind::InvalidData, e.to_string()))?;
        let name = entry.name().to_string();
        if name.to_lowercase().ends_with(".manifest") {
            let filename = Path::new(&name).file_name()
                .map(|n| n.to_string_lossy().to_string())
                .unwrap_or(name.clone());
            let dest = format!("{depot_dir}{MAIN_SEPARATOR}{filename}");
            if let Ok(()) = fs::create_dir_all(&depot_dir) {
                if let Ok(mut f) = File::create(&dest) {
                    let mut buf = Vec::new();
                    io::copy(&mut entry, &mut buf)?;
                    f.write_all(&buf)?;
                    debug!("Extracted manifest -> {dest}");
                }
            }
        }
    }

    // Second pass: find and install {appid}.lua
    let preferred = format!("{appid}.lua");
    let mut fallback: Option<String> = None;

    for i in 0..archive.len() {
        let entry = archive.by_index(i)
            .map_err(|e| Error::new(ErrorKind::InvalidData, e.to_string()))?;
        let name = entry.name().to_string();
        let basename = Path::new(&name)
            .file_name()
            .map(|n| n.to_string_lossy().to_string())
            .unwrap_or_default();

        // Only numeric .lua files (e.g. 12345.lua)
        if basename.ends_with(".lua") {
            let stem = basename.trim_end_matches(".lua");
            if stem.chars().all(|c| c.is_ascii_digit()) {
                if basename == preferred {
                    fallback = Some(name.clone());
                    break;
                } else if fallback.is_none() {
                    fallback = Some(name.clone());
                }
            }
        }
    }

    if let Some(entry_name) = fallback {
        let mut entry = archive.by_name(&entry_name)
            .map_err(|e| Error::new(ErrorKind::NotFound, e.to_string()))?;

        let mut content = Vec::new();
        io::copy(&mut entry, &mut content)?;

        // Comment out setManifestid lines (consistent with LuaTools behaviour)
        let text = String::from_utf8_lossy(&content);
        let processed: String = text.lines()
            .map(|line| {
                let trimmed = line.trim_start();
                if trimmed.starts_with("setManifestid(") && !trimmed.starts_with("--") {
                    format!("--{line}")
                } else {
                    line.to_string()
                }
            })
            .collect::<Vec<_>>()
            .join("\n");

        let dest = format!("{lua_dest_dir}{MAIN_SEPARATOR}{appid}.lua");
        let mut f = File::create(&dest)?;
        f.write_all(processed.as_bytes())?;
        lua_installed = true;
        debug!("Installed lua -> {dest}");
    }

    if lua_installed {
        Ok(())
    } else {
        Err(Error::new(ErrorKind::NotFound, "No numeric .lua found in ZIP"))
    }
}

impl Default for InstallPopup {
    fn default() -> Self {
        Self {
            active: false,
            appid: String::new(),
            status: InstallStatus::Idle,
        }
    }
}

impl WindowPopup for InstallPopup {
    fn view(app: &mut crate::App, ctx: &eframe::egui::Context) {
        Window::new("Install").default_size([0.0, 0.0]).open(&mut app.install.active).show(ctx, |ui| {
            ui.vertical_centered(|ui| {
                ui.horizontal(|ui| {
                    ui.label("Appid: ");
                    let response = ui.text_edit_singleline(&mut app.install.appid);
                    if response.changed() && !app.install.appid.chars().all(|c| c.is_ascii_digit()) {
                        app.install.appid.retain(|c| c.is_ascii_digit());
                    }
                });

                // Status label
                match &app.install.status {
                    InstallStatus::Downloading(api) => {
                        ui.spinner();
                        ui.label(format!("Trying: {api}..."));
                    }
                    InstallStatus::Done => { ui.label("✅ Installed!"); }
                    InstallStatus::AlreadyExists => { ui.label("⚠ Already installed."); }
                    InstallStatus::NotFound => { ui.label("❌ Not found in any source."); }
                    InstallStatus::Error(e) => { ui.label(format!("❌ Error: {e}")); }
                    InstallStatus::Idle => {}
                }

                let downloading = matches!(app.install.status, InstallStatus::Downloading(_));

                if !downloading && ui.button("Download").clicked() && !app.install.appid.is_empty() {
                    let appid: i32 = app.install.appid.parse().unwrap();
                    let steam_path = app.st.path.clone();

                    app.install.status = InstallStatus::Downloading("ManifestHub".to_string());

                    match install(&steam_path, appid) {
                        Ok(_) => {
                            app.install.status = InstallStatus::Done;
                            app.loaded = false;
                            rfd::MessageDialog::new()
                                .set_title("Success")
                                .set_description("Lua file downloaded and installed!")
                                .show();
                        }
                        Err(e) => {
                            app.install.status = match e.kind() {
                                ErrorKind::AlreadyExists => InstallStatus::AlreadyExists,
                                ErrorKind::ConnectionRefused => InstallStatus::NotFound,
                                _ => InstallStatus::Error(e.to_string()),
                            };
                            let msg = match &app.install.status {
                                InstallStatus::AlreadyExists => "You already have this game/app!".to_string(),
                                InstallStatus::NotFound => "Not found in any source\n(ManifestHub, Ryuu, TwentyTwo Cloud, Sushi)".to_string(),
                                InstallStatus::Error(e) => e.clone(),
                                _ => String::new(),
                            };
                            rfd::MessageDialog::new()
                                .set_title("Error")
                                .set_level(rfd::MessageLevel::Error)
                                .set_description(&msg)
                                .set_buttons(rfd::MessageButtons::Ok)
                                .show();
                        }
                    }
                }

                // Reset status when user changes appid
                if !matches!(app.install.status, InstallStatus::Idle | InstallStatus::Downloading(_)) {
                    if ui.small_button("Clear").clicked() {
                        app.install.status = InstallStatus::Idle;
                    }
                }
            });
        });
    }
}