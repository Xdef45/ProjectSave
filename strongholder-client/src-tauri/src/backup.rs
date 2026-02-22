use crate::system;
use serde::Serialize;
use std::fs;
use std::process::Stdio;
use std::sync::Mutex;
use tauri::{AppHandle, Emitter, Runtime, State};
use tauri_plugin_dialog::DialogExt;
use tokio::io::{AsyncBufReadExt, BufReader};
use tokio::process::Command as AsyncCommand;

// --- Structures de données ---

#[derive(Serialize)]
pub struct FileEntry {
    pub name: String,
    pub path: String,
    #[serde(rename = "isDirectory")]
    pub is_directory: bool,
}

#[derive(Serialize)]
pub struct DirResult {
    pub path: String,
    pub files: Vec<FileEntry>,
}

// --- Gestion de l'état ---
pub struct BackupState {
    pub active_pid: Mutex<Option<u32>>,
}

#[derive(Clone, Serialize)]
struct BackupEvent {
    event_type: String,
    data: String,
}

// --- Commandes Tauri ---

#[tauri::command]
pub fn list_directory(path: String) -> Result<DirResult, String> {
    // Si le chemin fourni est vide, on pointe par défaut sur la racine du système
    let target_path = if path.is_empty() {
        if cfg!(windows) {
            "C:\\".to_string()
        } else {
            "/".to_string()
        }
    } else {
        path
    };

    let entries = match fs::read_dir(&target_path) {
        Ok(dir) => dir,
        Err(e) => {
            println!(
                "[Sauvegarde] Dossier ignoré (illisible) '{}' : {}",
                target_path, e
            );
            return Ok(DirResult {
                path: target_path,
                files: Vec::new(),
            });
        }
    };

    let mut files = Vec::new();
    for entry in entries.flatten() {
        let meta = entry.metadata().ok();
        files.push(FileEntry {
            name: entry.file_name().to_string_lossy().into_owned(),
            path: entry.path().to_string_lossy().into_owned(),
            is_directory: meta.map(|m| m.is_dir()).unwrap_or(false),
        });
    }

    Ok(DirResult {
        path: target_path,
        files,
    })
}

#[tauri::command]
pub fn cancel_backup(state: State<'_, BackupState>) -> Result<(), String> {
    let mut pid_lock = state
        .active_pid
        .lock()
        .map_err(|_| "Impossible de verrouiller le mutex du PID")?;

    if let Some(pid) = *pid_lock {
        println!("[Sauvegarde] Arrêt forcé du processus PID : {}", pid);

        #[cfg(target_os = "windows")]
        {
            use std::os::windows::process::CommandExt;
            let _ = std::process::Command::new("taskkill")
                .args(&["/F", "/T", "/PID", &pid.to_string()])
                .creation_flags(0x08000000)
                .output();
        }

        #[cfg(target_os = "linux")]
        {
            let _ = std::process::Command::new("kill")
                .arg(pid.to_string())
                .output();
        }

        *pid_lock = None;
        Ok(())
    } else {
        Err("Aucune sauvegarde en cours à annuler".into())
    }
}

#[tauri::command]
pub async fn run_backup_script<R: Runtime>(
    app: AppHandle<R>,
    state: State<'_, BackupState>,
    client_id: String,
    preset_path: String,
    username: String,
) -> Result<(), String> {
    println!("\n--- [Lancement de la Sauvegarde] ---");

    let is_ssh_running = crate::installation::check_ssh_running().await;
    let mut we_started_ssh = false;

    if !is_ssh_running {
        println!("Le service SSH est arrêté. Démarrage temporaire pour la sauvegarde...");
        crate::installation::start_ssh_service(app.clone()).await?;
        we_started_ssh = true;
    } else {
        println!("Le service SSH est déjà en cours d'exécution. On ne touche à rien.");
    }

    let backup_result: Result<(), String> = async {
        let mut cmd;

        #[cfg(target_os = "windows")]
        {
            const CREATE_NO_WINDOW: u32 = 0x08000000;
            let distro_name = system::get_ubuntu_distro_name().await;

            let wsl_path_output = AsyncCommand::new("wsl")
                .args(&["-d", &distro_name, "wslpath", "-a", &preset_path])
                .creation_flags(CREATE_NO_WINDOW)
                .output()
                .await;

            let mut final_wsl_path = String::new();

            if let Ok(output) = wsl_path_output {
                let stdout_bytes = output.stdout;
                if stdout_bytes.contains(&0) && stdout_bytes.len() % 2 == 0 {
                    let u16_chars: Vec<u16> = stdout_bytes
                        .chunks_exact(2)
                        .map(|c| u16::from_le_bytes([c[0], c[1]]))
                        .collect();
                    final_wsl_path = String::from_utf16_lossy(&u16_chars).trim().to_string();
                } else {
                    final_wsl_path = String::from_utf8_lossy(&stdout_bytes).trim().to_string();
                }
            }

            if final_wsl_path.is_empty() {
                println!("[Sauvegarde] Échec de la commande 'wslpath'. Utilisation de la méthode de secours manuelle.");
                let forward_slashes = preset_path.replace("\\", "/");
                if let Some(colon_pos) = forward_slashes.find(':') {
                    let drive = &forward_slashes[0..colon_pos].to_lowercase();
                    let rest = &forward_slashes[colon_pos + 1..];
                    final_wsl_path = format!("/mnt/{}{}", drive, rest);
                } else {
                    final_wsl_path = forward_slashes;
                }
            }

            println!("[Windows] Chemin converti pour WSL : {}", final_wsl_path);

            cmd = AsyncCommand::new("wsl");
            cmd.args(&[
                "-d",
                &distro_name,
                "-u",
                &username,
                "bash",
                "/usr/local/sbin/scripts/client_backup.sh",
                &client_id,
                &final_wsl_path,
            ]);
            cmd.creation_flags(CREATE_NO_WINDOW);
        }

        #[cfg(target_os = "linux")]
        {
            println!("[Linux] Utilisation du chemin natif : {}", preset_path);
            let script_path = "/usr/local/sbin/scripts/client_backup.sh";

            if !std::path::Path::new(script_path).exists() {
                return Err(format!(
                    "Le script de sauvegarde est introuvable à l'emplacement {}.",
                    script_path
                ));
            }

            cmd = AsyncCommand::new("bash");
            cmd.args(&[script_path, &client_id, &preset_path]);
        }

        cmd.stdout(Stdio::piped());
        cmd.stderr(Stdio::piped());

        let mut child = cmd
            .spawn()
            .map_err(|e| format!("Échec du lancement du script de sauvegarde : {}", e))?;

        if let Some(pid) = child.id() {
            let mut pid_lock = state.active_pid.lock().unwrap();
            *pid_lock = Some(pid);
            println!("[Sauvegarde] Processus démarré avec le PID : {}", pid);
        }

        let stdout = child.stdout.take().expect("Impossible de capturer la sortie standard (stdout)");
        let stderr = child.stderr.take().expect("Impossible de capturer la sortie d'erreur (stderr)");

        let app_handle_1 = app.clone();
        tokio::spawn(async move {
            let mut reader = BufReader::new(stdout).lines();
            while let Ok(Some(line)) = reader.next_line().await {
                let clean_line = line.trim();
                if clean_line.starts_with("PROGRESS:") {
                    let percent = clean_line.replace("PROGRESS:", "").trim().to_string();
                    let _ = app_handle_1.emit(
                        "backup-event",
                        BackupEvent {
                            event_type: "progress".into(),
                            data: percent,
                        },
                    );
                } else if clean_line.starts_with("FILE:") {
                    let file = clean_line.replace("FILE:", "").trim().to_string();
                    let _ = app_handle_1.emit(
                        "backup-event",
                        BackupEvent {
                            event_type: "file".into(),
                            data: file,
                        },
                    );
                } else {
                    println!("[Script Info] {}", line);
                }
            }
        });

        let app_handle_2 = app.clone();
        tokio::spawn(async move {
            let mut reader = BufReader::new(stderr).lines();
            while let Ok(Some(line)) = reader.next_line().await {
                println!("[Script Erreur] {}", line);
                let _ = app_handle_2.emit(
                    "backup-event",
                    BackupEvent {
                        event_type: "log".into(),
                        data: line,
                    },
                );
            }
        });

        // Attente de la fin du processus de sauvegarde
        let status_result = child.wait().await;

        // Libération du PID
        {
            let mut pid_lock = state.active_pid.lock().unwrap();
            *pid_lock = None;
        }

        // Traitement du code de retour de la commande
        match status_result {
            Ok(status) if status.success() => {
                let _ = app.emit(
                    "backup-event",
                    BackupEvent {
                        event_type: "success".into(),
                        data: "100".into(),
                    },
                );
                Ok(())
            }
            Ok(status) => {
                let _ = app.emit(
                    "backup-event",
                    BackupEvent {
                        event_type: "error".into(),
                        data: format!("Le processus s'est terminé avec le code {:?}", status.code()),
                    },
                );
                Err("Le script de sauvegarde a renvoyé un code d'erreur non nul".into())
            }
            Err(e) => {
                let _ = app.emit(
                    "backup-event",
                    BackupEvent {
                        event_type: "error".into(),
                        data: e.to_string(),
                    },
                );
                Err(format!("Impossible d'attendre la fin du processus enfant : {}", e))
            }
        }
    }
    .await;

    // --- BLOC DE SÉCURITÉ : NETTOYAGE ---
    // Ce bloc s'exécute toujours, que la sauvegarde ait réussi ou qu'elle ait planté.
    if we_started_ssh {
        println!("[Sécurité] Sauvegarde terminée. Arrêt du service SSH...");
        if let Err(e) = crate::installation::stop_ssh_service(app.clone()).await {
            println!(
                "[Sécurité] CRITIQUE : Impossible d'arrêter le service SSH : {}",
                e
            );
        }
    }

    backup_result
}

#[tauri::command]
pub fn get_drives() -> Result<Vec<String>, String> {
    #[cfg(target_os = "windows")]
    {
        let mut drives = Vec::new();
        // Parcours de l'alphabet pour tester les lettres de lecteurs Windows (de A à Z)
        for letter in b'A'..=b'Z' {
            let drive_str = format!("{}:\\", letter as char);
            let path = std::path::Path::new(&drive_str);

            // Si le chemin existe selon le système, c'est que le lecteur est branché et valide !
            if path.exists() {
                drives.push(drive_str);
            }
        }
        Ok(drives)
    }

    #[cfg(target_os = "linux")]
    {
        // Sous Linux, il n'y a pas de lettres de lecteur, tout part de la racine
        Ok(vec!["/".to_string()])
    }
}

#[tauri::command]
pub async fn ask_save_path<R: Runtime>(
    app: tauri::AppHandle<R>,
    archive_name: String,
) -> Result<Option<String>, String> {
    // Création d'un canal de communication asynchrone pour ne pas geler l'interface utilisateur
    let (tx, rx) = tokio::sync::oneshot::channel();

    app.dialog()
        .file()
        .set_title("Enregistrer l'archive de sauvegarde")
        .set_file_name(&format!("{}.tar.gz", archive_name))
        .add_filter("Archive", &["tar.gz", "zip", "tar"])
        .save_file(move |file_path| {
            let _ = tx.send(file_path);
        });

    let file_path = rx.await.map_err(|_| {
        "Le canal de la boîte de dialogue s'est fermé de manière inattendue".to_string()
    })?;

    // On renvoie le chemin sous forme de chaîne au frontend (JS/Svelte), ou None si l'utilisateur a cliqué sur "Annuler"
    match file_path {
        Some(path) => Ok(Some(
            path.into_path().unwrap().to_string_lossy().into_owned(),
        )),
        None => Ok(None),
    }
}
