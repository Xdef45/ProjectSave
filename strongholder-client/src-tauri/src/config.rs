use serde::{Deserialize, Serialize};
use std::env;
use std::fs;
use std::path::PathBuf;
use std::process::Command;
use std::sync::RwLock;
use tauri::{AppHandle, Manager, Runtime};

const CONFIG_FILENAME: &str = "app_config.json";

// --- Gestion de l'état en mémoire ---
pub struct ConfigState(pub RwLock<AppConfig>);

// --- Structures de configuration ---
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct AppConfig {
    pub general: GeneralConfig,
    pub notifications: NotificationConfig,
    pub network: NetworkConfig,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct GeneralConfig {
    pub startup: bool,
    pub admin: bool,
    pub start_tray: bool,
    pub minimize_tray: bool,
    pub prevent_sleep: bool,
    pub battery_limit: bool,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct NotificationConfig {
    pub on_start: bool,
    pub on_success: bool,
    pub on_warning: bool,
    pub on_error: bool,
    pub sound: bool,
    pub on_client_issue: bool,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct NetworkConfig {
    pub upload_rate: u32,
}

impl Default for AppConfig {
    fn default() -> Self {
        Self {
            general: GeneralConfig {
                startup: false,
                admin: false,
                start_tray: false,
                minimize_tray: false,
                prevent_sleep: true,
                battery_limit: true,
            },
            notifications: NotificationConfig {
                on_start: true,
                on_success: true,
                on_warning: true,
                on_error: true,
                sound: false,
                on_client_issue: true,
            },
            network: NetworkConfig { upload_rate: 0 },
        }
    }
}

// --- Fonctions utilitaires ---

// Construit le chemin absolu vers le fichier de configuration de l'application
fn get_config_path<R: Runtime>(app: &AppHandle<R>) -> Result<PathBuf, String> {
    app.path()
        .app_config_dir()
        .map(|p| p.join(CONFIG_FILENAME))
        .map_err(|e| {
            format!(
                "Impossible de résoudre le dossier de configuration de l'application : {}",
                e
            )
        })
}

pub fn read_from_disk<R: Runtime>(app: &AppHandle<R>) -> AppConfig {
    let config_path = match get_config_path(app) {
        Ok(path) => path,
        Err(_) => {
            return AppConfig::default();
        }
    };

    if !config_path.exists() {
        return AppConfig::default();
    }

    match fs::read_to_string(&config_path) {
        Ok(content) => match serde_json::from_str::<AppConfig>(&content) {
            Ok(config) => config,
            Err(_) => AppConfig::default(),
        },
        Err(_) => AppConfig::default(),
    }
}

// --- Commandes Tauri (appelables depuis le frontend JS/Svelte) ---

#[tauri::command]
pub fn load_config<R: Runtime>(
    app: AppHandle<R>,
    state: tauri::State<'_, ConfigState>,
) -> AppConfig {
    // On lit le fichier depuis le disque local
    let loaded_config = read_from_disk(&app);

    // On met à jour l'état global de l'application pour que tout le monde ait la dernière version
    if let Ok(mut lock) = state.0.write() {
        *lock = loaded_config.clone();
    }

    loaded_config
}

#[tauri::command]
pub fn save_config<R: Runtime>(
    app: AppHandle<R>,
    state: tauri::State<'_, ConfigState>,
    config: AppConfig,
) -> Result<(), String> {
    let config_path = get_config_path(&app)?;

    // Création du dossier parent s'il n'existe pas encore
    let config_dir = config_path
        .parent()
        .ok_or_else(|| "Chemin de configuration invalide".to_string())?;

    if !config_dir.exists() {
        fs::create_dir_all(config_dir)
            .map_err(|e| format!("Impossible de créer le dossier de configuration : {}", e))?;
    }

    let content = serde_json::to_string_pretty(&config)
        .map_err(|e| format!("Impossible de sérialiser la configuration : {}", e))?;

    // Écriture atomique : on écrit dans un fichier temporaire puis on le renomme.
    // Cela évite de corrompre le fichier de configuration si l'application ou le PC crash pendant l'écriture.
    let tmp_path = config_path.with_extension("tmp");
    fs::write(&tmp_path, content).map_err(|e| {
        format!(
            "Impossible d'écrire le fichier de configuration temporaire : {}",
            e
        )
    })?;
    fs::rename(&tmp_path, config_path)
        .map_err(|e| format!("Impossible de valider le fichier de configuration : {}", e))?;

    // Mise à jour de la configuration en mémoire vive
    if let Ok(mut lock) = state.0.write() {
        *lock = config;
    }

    Ok(())
}

#[tauri::command]
pub fn restart_as_admin(app: AppHandle) -> Result<(), String> {
    let current_exe = env::current_exe().map_err(|e| {
        format!(
            "Impossible de récupérer le chemin de l'exécutable courant : {}",
            e
        )
    })?;

    // Élévation des privilèges spécifique à Windows
    #[cfg(target_os = "windows")]
    {
        let status = Command::new("powershell")
            .args([
                "-NoProfile",
                "-WindowStyle",
                "Hidden",
                "-Command",
                "Start-Process",
            ])
            .arg("-FilePath")
            .arg(&current_exe)
            .args(["-Verb", "RunAs"])
            .spawn();

        if status.is_err() {
            return Err("Impossible de lancer le processus administrateur. Avez-vous cliqué sur 'Non' lors de la demande d'autorisation (UAC) ?".to_string());
        }
    }

    // Élévation des privilèges spécifique à Linux
    #[cfg(target_os = "linux")]
    {
        let status = Command::new("pkexec").arg(current_exe).spawn();

        if status.is_err() {
            return Err("Impossible de lancer le processus administrateur.".to_string());
        }
    }

    // On ferme l'instance classique actuelle pour laisser la place à l'instance administrateur
    app.exit(0);

    Ok(())
}

#[tauri::command]
pub fn is_low_battery() -> bool {
    // Tente d'accéder au gestionnaire de batterie. Si on échoue (ex: PC fixe), on part du principe qu'il n'y a pas de problème.
    let manager = match battery::Manager::new() {
        Ok(m) => m,
        Err(_) => return false,
    };

    if let Ok(batteries) = manager.batteries() {
        for maybe_battery in batteries {
            if let Ok(battery) = maybe_battery {
                let state = battery.state();
                let percentage = battery.state_of_charge().value;

                // On vérifie si l'appareil se décharge ET qu'il reste moins de 25% de batterie
                if state == battery::State::Discharging && percentage < 0.25 {
                    return true;
                }
            }
        }
    }

    false
}
