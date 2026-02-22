use crate::parsing::{self, DashboardLogEntry, LogEntry};
use flate2::read::GzDecoder;
use reqwest::cookie::Jar;
use reqwest::{Client, StatusCode};
use serde::{de::DeserializeOwned, Deserialize, Serialize};
use std::io::Write;
use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;
use tar::Archive;
use tauri::State;

pub const API_BASE: &str = "https://strongholder.fr/api";

// --- Gestionnaire Réseau ---

#[derive(Clone)]
pub struct NetworkManager {
    // Une instance unique du client HTTP est utilisée pour toute l'application
    // pour optimiser les connexions (pool)
    pub client: Client,
    // Permet de stocker et d'envoyer automatiquement les cookies de session
    pub jar: Arc<Jar>,
}

impl NetworkManager {
    pub fn new() -> Self {
        let jar = Arc::new(Jar::default());
        let client = Client::builder()
            .cookie_store(true)
            .cookie_provider(jar.clone())
            .user_agent("Strongholder-App/1.0")
            .build()
            .expect("Impossible de construire le client HTTP réseau");

        Self { client, jar }
    }

    // --- Utilitaires de requêtes génériques (DRY) ---

    // Exécute la requête, vérifie les erreurs du serveur et retourne le texte brut
    async fn fetch_text(&self, req: reqwest::RequestBuilder) -> Result<String, String> {
        let req = req.header("Accept", "application/json");
        let res = req.send().await.map_err(|e| e.to_string())?;

        let status = res.status();
        let text = res.text().await.map_err(|e| e.to_string())?;

        handle_response_error(status, &text)?;
        Ok(text)
    }

    pub async fn post_raw(&self, url: &str) -> Result<String, String> {
        self.fetch_text(self.client.post(url)).await
    }

    pub async fn post_with_payload_raw<P: Serialize>(
        &self,
        url: &str,
        payload: &P,
    ) -> Result<String, String> {
        self.fetch_text(self.client.post(url).json(payload)).await
    }

    // Envoie une requête POST, vérifie les erreurs et convertit directement le JSON en structure Rust
    pub async fn post_and_parse<R: DeserializeOwned>(&self, url: &str) -> Result<R, String> {
        let text = self.post_raw(url).await?;
        serde_json::from_str(&text)
            .map_err(|e| format!("Impossible d'analyser le JSON renvoyé : {}", e))
    }

    pub async fn post_and_parse_with_payload<P: Serialize, R: DeserializeOwned>(
        &self,
        url: &str,
        payload: &P,
    ) -> Result<R, String> {
        let text = self.post_with_payload_raw(url, payload).await?;
        serde_json::from_str(&text)
            .map_err(|e| format!("Impossible d'analyser le JSON renvoyé : {}", e))
    }
}

// --- Structures de données (DTOs) ---

#[derive(Serialize)]
struct AuthPayload<'a> {
    username: &'a str,
    password: &'a str,
}

#[derive(Deserialize)]
struct ClientIdResponse {
    id: String,
}

#[derive(Deserialize)]
struct ServerKeyResponse {
    ssh_pub: String,
}

#[derive(Serialize)]
struct SshKeyPayload {
    ssh: String,
}

// Intercepte les erreurs HTTP classiques ainsi que les messages d'erreur textuels renvoyés par l'API
fn handle_response_error(status: StatusCode, text: &str) -> Result<(), String> {
    if !status.is_success() {
        return Err(format!("Erreur réseau (Code {}) : {}", status, text));
    }

    // Gestion des erreurs textuelles renvoyées par le backend (avec un code HTTP 200)
    let lower_text = text.to_lowercase();
    if lower_text.contains("incorrect")
        || lower_text.contains("existe déjà")
        || lower_text.contains("token expiré")
        || lower_text.contains("pas de cookie")
    {
        return Err(text.to_string());
    }

    Ok(())
}

// --- Commandes d'Authentification et SSH ---

#[tauri::command]
pub async fn login_user(
    state: State<'_, NetworkManager>,
    username: String,
    password: String,
    is_signup: bool,
) -> Result<String, String> {
    let endpoint = if is_signup { "signup" } else { "signin" };
    let url = format!("{}/{}", API_BASE, endpoint);

    let payload = AuthPayload {
        username: &username,
        password: &password,
    };

    state.post_with_payload_raw(&url, &payload).await?;

    Ok("Connexion réussie".to_string())
}

#[tauri::command]
pub async fn get_client_id_req(state: State<'_, NetworkManager>) -> Result<String, String> {
    let url = format!("{}/imaconnected", API_BASE);
    let json: ClientIdResponse = state.post_and_parse(&url).await?;
    Ok(json.id)
}

#[tauri::command]
pub async fn get_repo_key_req(state: State<'_, NetworkManager>) -> Result<Vec<u8>, String> {
    let url = format!("{}/get_repot_key", API_BASE);

    let res = state
        .client
        .post(&url)
        .send()
        .await
        .map_err(|e| e.to_string())?;

    if !res.status().is_success() {
        return Err(format!(
            "Impossible de récupérer la clé du dépôt : {}",
            res.status()
        ));
    }

    let bytes = res.bytes().await.map_err(|e| e.to_string())?;
    Ok(bytes.to_vec())
}

#[tauri::command]
pub async fn send_ssh_key_req(
    state: State<'_, NetworkManager>,
    key_content: String,
    is_tunnel: bool,
) -> Result<(), String> {
    let endpoint = if is_tunnel {
        "send_ssh_key_tunnel"
    } else {
        "send_ssh_key"
    };
    let url = format!("{}/{}", API_BASE, endpoint);
    let payload = SshKeyPayload { ssh: key_content };

    state.post_with_payload_raw(&url, &payload).await?;
    Ok(())
}

#[tauri::command]
pub async fn get_server_ssh_key_req(state: State<'_, NetworkManager>) -> Result<String, String> {
    let url = format!("{}/get_ssh_pub_key_server", API_BASE);
    let json: ServerKeyResponse = state.post_and_parse(&url).await?;
    Ok(json.ssh_pub)
}

// --- Historique et Journaux ---

#[tauri::command]
pub async fn get_logs_req(state: State<'_, NetworkManager>) -> Result<Vec<LogEntry>, String> {
    let url = format!("{}/get_log", crate::network::API_BASE);
    println!("[Réseau] Requête de récupération des journaux d'activité...");
    let text = state.post_raw(&url).await?;
    let logs = parsing::parse_server_response(&text);
    Ok(logs)
}

// --- Structures de données pour les Sauvegardes ---

#[derive(Serialize, Deserialize, Debug)]
pub struct ArchiveItem {
    pub archive: String,
    pub time: String,
}

#[derive(Deserialize)]
struct ArchiveListResponse {
    archives: Vec<ArchiveItem>,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct ArchiveFileRaw {
    #[serde(rename = "type")]
    pub file_type: String,
    pub path: String,
    pub mtime: String,
    pub size: u64,
}

#[derive(Deserialize)]
struct ArchiveContentResponse {
    archive_content: Vec<ArchiveFileRaw>,
}

#[derive(Serialize)]
struct ArchiveRequest {
    archive_name: String,
}

// --- Commandes de Restauration ---

#[tauri::command]
pub async fn fetch_archives_list_req(
    state: State<'_, NetworkManager>,
) -> Result<Vec<ArchiveItem>, String> {
    let url = format!("{}/get_list", API_BASE);
    println!(
        "[Réseau] Récupération de la liste des archives depuis {}",
        url
    );

    let data: ArchiveListResponse = state.post_and_parse(&url).await?;

    // On masque les archives de type "logs" à l'utilisateur
    let filtered_archives: Vec<ArchiveItem> = data
        .archives
        .into_iter()
        .filter(|item| !item.archive.to_lowercase().contains("logs"))
        .collect();

    Ok(filtered_archives)
}

#[tauri::command]
pub async fn fetch_archive_files_req(
    state: State<'_, NetworkManager>,
    archive_name: String,
) -> Result<Vec<ArchiveFileRaw>, String> {
    let url = format!("{}/get_list", API_BASE);
    println!(
        "[Réseau] Récupération du contenu de l'archive : {}",
        archive_name
    );

    let payload = ArchiveRequest { archive_name };
    let data: ArchiveContentResponse = state.post_and_parse_with_payload(&url, &payload).await?;

    Ok(data.archive_content)
}

// État global pour suivre et annuler une opération de restauration en cours
pub struct RestoreState {
    pub is_cancelled: Arc<AtomicBool>,
}

impl Default for RestoreState {
    fn default() -> Self {
        Self {
            is_cancelled: Arc::new(AtomicBool::new(false)),
        }
    }
}

#[tauri::command]
pub fn cancel_restore_operation(state: State<'_, RestoreState>) {
    println!("[Système] Annulation de l'opération demandée par l'utilisateur !");
    state.is_cancelled.store(true, Ordering::SeqCst);
}

#[tauri::command]
pub async fn download_and_save_archive_req(
    network_state: State<'_, NetworkManager>,
    restore_state: State<'_, RestoreState>,
    archive_name: String,
    target_path: String,
) -> Result<String, String> {
    // Réinitialise le drapeau d'annulation avant de commencer
    restore_state.is_cancelled.store(false, Ordering::SeqCst);

    let url = format!("{}/get_restore", API_BASE);
    println!(
        "[Téléchargement] Enregistrement direct sur le disque : {}",
        target_path
    );

    let payload = ArchiveRequest { archive_name };

    let mut res = network_state
        .client
        .post(&url)
        .json(&payload)
        .header("Accept", "application/octet-stream")
        .send()
        .await
        .map_err(|e| format!("Erreur de connexion : {}", e))?;

    if !res.status().is_success() {
        let err = res.text().await.unwrap_or_default();
        return Err(format!("Échec du téléchargement : {}", err));
    }

    let mut file = std::fs::File::create(&target_path)
        .map_err(|e| format!("Impossible de créer le fichier cible : {}", e))?;

    // Téléchargement en streaming par morceaux (chunks).
    // Cela permet de télécharger des fichiers de plusieurs Go sans exploser la RAM du système.
    while let Some(chunk) = res
        .chunk()
        .await
        .map_err(|e| format!("Erreur lors de la lecture du flux réseau : {}", e))?
    {
        // On vérifie à chaque morceau si l'utilisateur a cliqué sur "Annuler"
        if restore_state.is_cancelled.load(Ordering::SeqCst) {
            println!("[Téléchargement] Opération interrompue. Nettoyage du fichier partiel.");
            let _ = std::fs::remove_file(&target_path);
            return Err("Téléchargement annulé par l'utilisateur".to_string());
        }

        file.write_all(&chunk)
            .map_err(|e| format!("Impossible d'écrire les données sur le disque : {}", e))?;
    }

    println!("[Téléchargement] Fichier téléchargé et sauvegardé avec succès !");
    Ok(target_path)
}

#[tauri::command]
pub async fn restore_to_original_req(
    network_state: State<'_, NetworkManager>,
    restore_state: State<'_, RestoreState>,
    archive_name: String,
) -> Result<String, String> {
    restore_state.is_cancelled.store(false, Ordering::SeqCst);

    println!(
        "[Restauration] Téléchargement de l'archive {} pour une restauration à l'emplacement d'origine...",
        archive_name
    );

    let url = format!("{}/get_restore", API_BASE);
    let payload = ArchiveRequest {
        archive_name: archive_name.clone(),
    };

    let mut res = network_state
        .client
        .post(&url)
        .json(&payload)
        .header("Accept", "application/octet-stream")
        .send()
        .await
        .map_err(|e| format!("Erreur de connexion : {}", e))?;

    if !res.status().is_success() {
        let err_text = res.text().await.unwrap_or_default();
        return Err(format!("Échec du téléchargement : {}", err_text));
    }

    // Création d'un fichier temporaire caché pour stocker l'archive avant décompression
    let temp_dir = std::env::temp_dir();
    let temp_file_path = temp_dir.join(format!("strongholder_restore_{}.tar.gz", archive_name));

    let mut temp_file = std::fs::File::create(&temp_file_path)
        .map_err(|e| format!("Impossible de créer le fichier temporaire : {}", e))?;

    // Réception du fichier en streaming vers le dossier temporaire
    while let Some(chunk) = res
        .chunk()
        .await
        .map_err(|e| format!("Erreur lors de la lecture du flux : {}", e))?
    {
        if restore_state.is_cancelled.load(Ordering::SeqCst) {
            println!("[Restauration] Téléchargement interrompu en cours de route !");
            let _ = std::fs::remove_file(&temp_file_path);
            return Err("Restauration annulée par l'utilisateur".to_string());
        }

        temp_file
            .write_all(&chunk)
            .map_err(|e| format!("Impossible d'écrire dans le fichier temporaire : {}", e))?;
    }

    println!("[Restauration] Téléchargement terminé. Début de l'extraction...");

    let cancel_flag = restore_state.is_cancelled.clone();

    // La décompression est déléguée à un thread bloquant séparé pour ne pas geler l'interface Tauri
    let result_message = tokio::task::spawn_blocking(move || {
        let file_for_reading = std::fs::File::open(&temp_file_path)
            .map_err(|e| format!("Impossible d'ouvrir l'archive temporaire : {}", e))?;

        let tar = GzDecoder::new(file_for_reading);
        let mut archive = Archive::new(tar);

        let mut restored_count = 0;
        let mut skipped_count = 0;

        for entry in archive.entries().map_err(|e| e.to_string())? {
            if cancel_flag.load(Ordering::SeqCst) {
                println!("[Restauration] Extraction stoppée par l'utilisateur !");
                let _ = std::fs::remove_file(&temp_file_path);
                return Err("Restauration annulée pendant l'extraction".to_string());
            }

            let mut file = entry.map_err(|e| e.to_string())?;
            let path = file.path().map_err(|e| e.to_string())?.into_owned();

            let target_path = translate_wsl_to_win(&path);

            if let Some(dest) = target_path {
                // BLOC DE SÉCURITÉ : Empêche l'archive d'écraser des dossiers critiques du système d'exploitation.
                let dest_lower = dest.to_string_lossy().to_lowercase();

                #[cfg(target_os = "windows")]
                let is_protected = dest_lower.starts_with("c:\\windows")
                    || dest_lower.starts_with("c:\\program files");

                #[cfg(target_os = "linux")]
                let is_protected = dest_lower.starts_with("/bin")
                    || dest_lower.starts_with("/sbin")
                    || dest_lower.starts_with("/boot")
                    || dest_lower.starts_with("/usr")
                    || dest_lower.starts_with("/lib")
                    || dest_lower.starts_with("/sys")
                    || dest_lower.starts_with("/proc")
                    || dest_lower.starts_with("/dev")
                    || dest_lower.starts_with("/etc");

                if is_protected {
                    println!(
                        "[Sécurité] Tentative bloquée d'écraser un chemin protégé : {:?}",
                        dest
                    );
                    skipped_count += 1;
                    continue;
                }

                if let Some(parent) = dest.parent() {
                    std::fs::create_dir_all(parent).map_err(|e| e.to_string())?;
                }

                file.unpack(&dest).map_err(|e| {
                    format!("Impossible de restaurer le fichier {:?} : {}", dest, e)
                })?;
                restored_count += 1;
            } else {
                println!(
                    "[Restauration] Chemin ignoré (format non reconnu) : {:?}",
                    path
                );
                skipped_count += 1;
            }
        }

        // Nettoyage final du fichier lourd temporaire
        let _ = std::fs::remove_file(&temp_file_path);

        Ok::<String, String>(format!(
            "Restauration terminée : {} fichiers remis à leur emplacement d'origine ({} ignorés).",
            restored_count, skipped_count
        ))
    })
    .await
    .map_err(|e| format!("La tâche de décompression a planté : {}", e))??;

    Ok(result_message)
}

// Convertit les chemins Linux/WSL (ex: /mnt/c/Dossier) vers des chemins Windows exploitables (C:\Dossier)
fn translate_wsl_to_win(path: &Path) -> Option<PathBuf> {
    let path_str = path.to_string_lossy();
    let unified = path_str.replace("\\", "/");
    let parts: Vec<&str> = unified.split('/').filter(|s| !s.is_empty()).collect();

    // On vérifie si le chemin commence par "mnt" (typique de WSL et de l'outil Borg Backup)
    if parts.len() > 2 && parts[0] == "mnt" {
        let drive_letter = parts[1];
        if drive_letter.len() == 1 {
            let rest = parts[2..].join("\\");
            return Some(PathBuf::from(format!("{}:\\{}", drive_letter, rest)));
        }
    }

    None
}

// --- Vérifications de l'état système ---

#[tauri::command]
pub async fn check_internet_connection(state: State<'_, NetworkManager>) -> Result<bool, String> {
    let res = state
        .client
        .get("https://clients3.google.com/generate_204")
        .timeout(std::time::Duration::from_secs(2))
        .send()
        .await;

    match res {
        Ok(response) => Ok(response.status().is_success()),
        Err(_) => Ok(false),
    }
}

#[tauri::command]
pub async fn get_backup_logs(
    state: State<'_, NetworkManager>,
) -> Result<Vec<DashboardLogEntry>, String> {
    let url = format!("{}/get_log", crate::network::API_BASE);

    print!("[Système] Récupération des logs de sauvegarde...\n");

    let text = state.post_raw(&url).await?;
    let full_logs = parsing::parse_server_response(&text);

    let dashboard_logs: Vec<DashboardLogEntry> =
        full_logs.into_iter().map(DashboardLogEntry::from).collect();

    Ok(dashboard_logs)
}
