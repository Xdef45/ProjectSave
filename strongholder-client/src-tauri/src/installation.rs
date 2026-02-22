use crate::system;
use std::fs;
use tauri::{AppHandle, Emitter, Manager, Runtime};
use tokio::process::Command as AsyncCommand;

#[cfg(target_os = "windows")]
use std::os::windows::process::CommandExt;
const CREATE_NO_WINDOW: u32 = 0x08000000;
const EXPECTED_UBUNTU_HASH: &str =
    "8251e27ffff381a4af5f41dcb94d867de3e0d9774a9241908ab34555d99315ea";

// --- Utilitaires ---

// Envoie les messages de suivi directement au frontend (via l'événement "setup_log")
// tout en les affichant dans la console pour le développeur.
fn log<R: Runtime>(app: &AppHandle<R>, message: &str) {
    println!("[INSTALLATION] {}", message);
    let _ = app.emit("setup_log", message);
}

// Exécute une commande avec les privilèges administrateur/root dans le système cible.
async fn run_privileged(cmd: &str) -> Result<(), String> {
    #[cfg(target_os = "windows")]
    {
        let distro_name = system::get_ubuntu_distro_name().await;

        let status = AsyncCommand::new("wsl")
            .args(["-d", &distro_name, "-u", "root", "sh", "-c", cmd])
            .creation_flags(CREATE_NO_WINDOW)
            .status()
            .await
            .map_err(|e| e.to_string())?;

        if status.success() {
            Ok(())
        } else {
            Err("La commande WSL a échoué".into())
        }
    }
    #[cfg(target_os = "linux")]
    {
        let status = AsyncCommand::new("pkexec")
            .args(["sh", "-c", cmd])
            .status()
            .await
            .map_err(|e| e.to_string())?;

        if status.success() {
            Ok(())
        } else {
            Err("La commande Linux avec pkexec a échoué".into())
        }
    }
}

// Raccourci pour exécuter silencieusement une commande sous WSL (Windows uniquement).
#[cfg(target_os = "windows")]
async fn run_wsl_command(cmd: &str) -> Result<std::process::Output, std::io::Error> {
    let distro_name = system::get_ubuntu_distro_name().await;
    AsyncCommand::new("wsl")
        .args(["-d", &distro_name, "-u", "root", "sh", "-c", cmd])
        .creation_flags(CREATE_NO_WINDOW)
        .output()
        .await
}

// --- Étape 1 : Moteur WSL ---

#[tauri::command]
pub async fn check_wsl_installed() -> bool {
    #[cfg(target_os = "windows")]
    {
        match AsyncCommand::new("wsl")
            .arg("--status")
            .creation_flags(CREATE_NO_WINDOW)
            .output()
            .await
        {
            Ok(out) => out.status.success(),
            Err(_) => false,
        }
    }
    #[cfg(target_os = "linux")]
    {
        true
    }
}

#[tauri::command]
pub async fn install_wsl_engine<R: Runtime>(app: AppHandle<R>) -> Result<(), String> {
    #[cfg(target_os = "windows")]
    {
        log(&app, "Installation du moteur principal WSL...");

        let status = AsyncCommand::new("powershell")
            .args([
                "-NoProfile",
                "-ExecutionPolicy", "Bypass",
                "-Command", 
                "Start-Process wsl -ArgumentList '--install', '--no-distribution' -Verb RunAs -WindowStyle Hidden -Wait"
            ])
            .status()
            .await
            .map_err(|e| e.to_string())?;

        if status.success() {
            Ok(())
        } else {
            Err("L'installation du moteur WSL a échoué ou a été annulée par l'utilisateur.".into())
        }
    }
    #[cfg(target_os = "linux")]
    {
        Ok(())
    }
}

// --- Étape 2 : Distribution Ubuntu ---

#[tauri::command]
pub async fn check_ubuntu_installed() -> bool {
    #[cfg(target_os = "windows")]
    {
        if let Ok(out) = AsyncCommand::new("wsl")
            .args(["--list", "--quiet"])
            .creation_flags(CREATE_NO_WINDOW)
            .output()
            .await
        {
            let raw = String::from_utf8_lossy(&out.stdout).to_lowercase();
            // On filtre les octets nuls pour éviter les problèmes d'encodage UTF-16 retournés par WSL
            let clean_text: String = raw.chars().filter(|c| *c != '\0').collect();
            clean_text.contains("ubuntu")
        } else {
            false
        }
    }
    #[cfg(target_os = "linux")]
    {
        true
    }
}

#[tauri::command]
pub async fn install_ubuntu_silent<R: Runtime>(app: AppHandle<R>) -> Result<(), String> {
    #[cfg(target_os = "windows")]
    {
        log(
            &app,
            "Démarrage de l'installation manuelle d'Ubuntu (Mode silencieux)...",
        );

        let app_data = app.path().app_data_dir().map_err(|e| e.to_string())?;
        if !app_data.exists() {
            fs::create_dir_all(&app_data).map_err(|e| e.to_string())?;
        }

        let install_dir = app_data.join("Distro");
        let tarball_path = app_data.join("ubuntu-noble-wsl-amd64-wsl.rootfs.tar.gz");

        log(&app, "Nettoyage des fichiers d'installations précédents...");
        let _ = AsyncCommand::new("wsl")
            .args(["--unregister", "Ubuntu"])
            .creation_flags(CREATE_NO_WINDOW)
            .status()
            .await;

        if install_dir.exists() {
            let _ = fs::remove_dir_all(&install_dir);
        }
        fs::create_dir_all(&install_dir).map_err(|e| e.to_string())?;

        if tarball_path.exists() {
            let _ = fs::remove_file(&tarball_path);
        }

        // Téléchargement de l'image de la distribution
        let url = "https://cloud-images.ubuntu.com/wsl/releases/24.04/current/ubuntu-noble-wsl-amd64-wsl.rootfs.tar.gz";
        log(
            &app,
            "Téléchargement de l'image système Ubuntu (environ 350 Mo)...",
        );

        let dl_status = AsyncCommand::new("curl")
            .args(["-L", "-o", &tarball_path.to_string_lossy(), url])
            .creation_flags(CREATE_NO_WINDOW)
            .status()
            .await
            .map_err(|e| format!("L'exécution de la requête réseau a échoué : {}", e))?;

        if !dl_status.success() {
            return Err("Échec du téléchargement de l'archive Ubuntu.".into());
        }

        // Sécurité : Vérification de la somme de contrôle (Hash) pour s'assurer que le fichier n'est pas corrompu
        log(&app, "Vérification de l'intégrité de l'image...");
        let hash_output = AsyncCommand::new("powershell")
            .args([
                "-NoProfile",
                "-Command",
                &format!(
                    "(Get-FileHash -Path '{}' -Algorithm SHA256).Hash",
                    tarball_path.to_string_lossy()
                ),
            ])
            .creation_flags(CREATE_NO_WINDOW)
            .output()
            .await
            .map_err(|e| e.to_string())?;

        let actual_hash = String::from_utf8_lossy(&hash_output.stdout)
            .trim()
            .to_string();

        if !actual_hash.eq_ignore_ascii_case(EXPECTED_UBUNTU_HASH) {
            let _ = fs::remove_file(&tarball_path); // Destruction immédiate du fichier suspect
            return Err(format!(
                "Erreur de sécurité : L'empreinte de l'image Ubuntu ne correspond pas !\nAttendu : {}\nObtenu : {}", 
                EXPECTED_UBUNTU_HASH, actual_hash
            ));
        }

        log(&app, "Importation de la distribution dans WSL...");
        let import_result = AsyncCommand::new("wsl")
            .args([
                "--import",
                "Ubuntu",
                &install_dir.to_string_lossy(),
                &tarball_path.to_string_lossy(),
            ])
            .creation_flags(CREATE_NO_WINDOW)
            .status()
            .await;

        // Quoi qu'il arrive, on libère l'espace disque en supprimant l'archive
        let _ = fs::remove_file(&tarball_path);

        match import_result {
            Ok(status) if status.success() => {
                log(&app, "Ubuntu a été installé avec succès.");
                Ok(())
            }
            Ok(_) => Err("Échec de l'importation de la distribution WSL. Le processus a retourné une erreur.".into()),
            Err(e) => Err(format!("Échec de l'exécution de la commande d'importation : {}", e)),
        }
    }
    #[cfg(target_os = "linux")]
    {
        Ok(())
    }
}

// --- Étapes 3 & 4 : Gestion du réseau et de la sécurité SSH ---

#[tauri::command]
pub async fn check_ssh_installed() -> bool {
    let check_cmd = "dpkg -l | grep openssh-server";

    #[cfg(target_os = "windows")]
    {
        match run_wsl_command(check_cmd).await {
            Ok(out) => !out.stdout.is_empty(),
            Err(_) => false,
        }
    }
    #[cfg(target_os = "linux")]
    {
        match AsyncCommand::new("sh")
            .arg("-c")
            .arg(check_cmd)
            .output()
            .await
        {
            Ok(out) => !out.stdout.is_empty(),
            Err(_) => false,
        }
    }
}

#[tauri::command]
pub async fn check_ssh_running() -> bool {
    let check_cmd = "systemctl status ssh | grep 'active (running)'";

    #[cfg(target_os = "windows")]
    {
        match run_wsl_command(check_cmd).await {
            Ok(out) => out.status.success(),
            Err(_) => false,
        }
    }
    #[cfg(target_os = "linux")]
    {
        match AsyncCommand::new("sh")
            .arg("-c")
            .arg(check_cmd)
            .output()
            .await
        {
            Ok(out) => out.status.success(),
            Err(_) => false,
        }
    }
}

// Installe SSH et le configure pour une utilisation hautement sécurisée (On-Demand)
#[tauri::command]
pub async fn install_ssh_silent<R: Runtime>(app: AppHandle<R>) -> Result<(), String> {
    log(&app, "Installation du serveur OpenSSH...");

    // 1. Installation du paquet openssh-server.
    // 2. Restriction stricte des écoutes sur l'interface locale (127.0.0.1) pour bloquer les requêtes externes.
    // 3. Désactivation du démarrage automatique au boot, et arrêt immédiat du service.
    let cmd = "\
        apt-get update && \
        apt-get install -y openssh-server && \
        mkdir -p /etc/ssh/sshd_config.d && \
        echo 'ListenAddress 127.0.0.1' > /etc/ssh/sshd_config.d/99-strongholder-secure.conf && \
        systemctl disable ssh && \
        systemctl stop ssh\
    ";

    run_privileged(cmd).await
}

#[tauri::command]
pub async fn start_ssh_service<R: Runtime>(app: AppHandle<R>) -> Result<(), String> {
    log(
        &app,
        "Démarrage du service SSH (Mode temporaire à la demande)...",
    );
    run_privileged("systemctl start ssh").await
}

#[tauri::command]
pub async fn stop_ssh_service<R: Runtime>(app: AppHandle<R>) -> Result<(), String> {
    log(&app, "Arrêt du service SSH (Clôture de sécurité)...");
    run_privileged("systemctl stop ssh").await
}

// --- Étape 5 : Vérification de l'espace disque ---

#[tauri::command]
pub async fn check_disk_space<R: Runtime>(app: AppHandle<R>) -> Result<bool, String> {
    const MIN_BYTES: u64 = 3 * 1024 * 1024 * 1024; // 3 Go minimum requis pour l'image Ubuntu et l'extraction

    #[cfg(target_os = "windows")]
    {
        let ps_cmd = "[System.Console]::Write([System.IO.DriveInfo]::new((Get-Item $env:APPDATA).Root.Name).AvailableFreeSpace)";

        let output = AsyncCommand::new("powershell")
            .args(["-NoProfile", "-Command", ps_cmd])
            .creation_flags(CREATE_NO_WINDOW)
            .output()
            .await
            .map_err(|e| e.to_string())?;

        if !output.status.success() {
            println!("[ATTENTION] Impossible de vérifier l'espace disque disponible.");
            return Ok(true); // On laisse passer au bénéfice du doute
        }

        let output_str = String::from_utf8_lossy(&output.stdout);
        let free_bytes: u64 = output_str.trim().parse().unwrap_or(u64::MAX);

        if free_bytes < MIN_BYTES {
            log(
                &app,
                &format!(
                    "Espace disque insuffisant : {} Go disponibles, 3 Go requis pour l'installation.",
                    free_bytes / 1024 / 1024 / 1024
                ),
            );
            return Ok(false);
        }
        Ok(true)
    }
    #[cfg(target_os = "linux")]
    {
        // Pas de vérification stricte nécessaire sous Linux (pas de tarball WSL lourd à télécharger)
        Ok(true)
    }
}
