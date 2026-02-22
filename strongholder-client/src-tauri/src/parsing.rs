use rayon::prelude::*;
use serde::{Deserialize, Serialize};

// --- Structures de sortie (Envoyées au Frontend) ---

#[derive(Serialize, Clone, Debug)]
pub struct FileStatus {
    pub path: String,
    // Optimisation : utilisation d'un caractère simple (4 octets) plutôt qu'une allocation de String
    pub status: char,
}

#[derive(Serialize, Clone, Debug)]
pub struct LogEntry {
    pub id: String,
    pub date: String,
    pub duration: f64,
    pub total_size: u64,
    pub total_files: u64,
    pub status: String,
    pub count_added: usize,
    pub count_modified: usize,
    pub count_deleted: usize,
    pub count_error: usize,
    pub files: Vec<FileStatus>,
}

#[derive(Serialize, Clone, Debug)]
pub struct DashboardLogEntry {
    pub id: String,
    pub date: String,
    pub duration: f64,
    pub total_size: u64,
    pub total_files: u64,
    pub status: String,
    pub count_added: usize,
    pub count_modified: usize,
    pub count_deleted: usize,
    pub count_error: usize,
    // Le tableau 'files' est volontairement omis ici !
    // Cela permet un gain massif de vitesse et de mémoire lors du chargement de la vue globale du tableau de bord.
}

// Conversion automatique d'un journal complet vers un journal allégé pour le tableau de bord
impl From<LogEntry> for DashboardLogEntry {
    fn from(log: LogEntry) -> Self {
        Self {
            id: log.id,
            date: log.date,
            duration: log.duration,
            total_size: log.total_size,
            total_files: log.total_files,
            status: log.status,
            count_added: log.count_added,
            count_modified: log.count_modified,
            count_deleted: log.count_deleted,
            count_error: log.count_error,
        }
    }
}

// --- Structures d'entrée (Reçues du Serveur) ---

#[derive(Deserialize, Debug)]
struct ServerLogResponse {
    logs: Vec<String>,
}

#[derive(Deserialize, Debug)]
struct BorgArchiveWrapper {
    archive: BorgArchive,
}

#[derive(Deserialize, Debug)]
struct BorgArchive {
    id: String,
    end: String,
    duration: f64,
    stats: BorgStats,
}

#[derive(Deserialize, Debug)]
struct BorgStats {
    original_size: u64,
    nfiles: u64,
}

// --- Logique d'analyse (Parsing) ---

pub fn parse_server_response(json_text: &str) -> Vec<LogEntry> {
    // 1. Tente d'analyser l'enveloppe principale JSON
    let response: ServerLogResponse = match serde_json::from_str(json_text) {
        Ok(r) => r,
        Err(_) => return vec![], // Échec silencieux si le flux du serveur est invalide (renvoie un tableau vide)
    };

    // 2. Traitement de chaque bloc d'historique EN PARALLÈLE
    // L'utilisation de 'into_par_iter' (fourni par rayon) répartit automatiquement la charge sur tous les cœurs du processeur.
    let mut entries: Vec<LogEntry> = response
        .logs
        .into_par_iter()
        .filter_map(|block| parse_mixed_block(&block))
        .collect();

    // 3. Tri des résultats par date décroissante (du plus récent au plus ancien)
    entries.sort_by(|a, b| b.date.cmp(&a.date));
    entries
}

fn parse_mixed_block(raw_text: &str) -> Option<LogEntry> {
    let mut files: Vec<FileStatus> = Vec::new();
    let mut wrapper: Option<BorgArchiveWrapper> = None;

    // Chemin rapide : on isole uniquement la portion JSON dans le texte brut.
    // Cela permet à serde_json de travailler directement sur la référence sans allouer de nouvelle mémoire (zéro-copie).
    if let Some(start_idx) = raw_text.find('{') {
        if let Some(end_idx) = raw_text.rfind('}') {
            let json_slice = &raw_text[start_idx..=end_idx];
            wrapper = serde_json::from_str::<BorgArchiveWrapper>(json_slice).ok();
        }
    }

    // Analyse ligne par ligne pour extraire les fichiers modifiés
    for line in raw_text.lines() {
        let trimmed = line.trim();
        let bytes = trimmed.as_bytes();

        if bytes.is_empty() {
            continue;
        }

        // Rejet extrêmement rapide des lignes qui correspondent à du JSON (accolades ou guillemets)
        if bytes[0] == b'{' || bytes[0] == b'}' || bytes[0] == b'"' {
            continue;
        }

        // Analyse des fichiers : on s'attend au format "M /chemin/vers/fichier"
        // Vérifier l'espace via bytes[1] == b' ' est une opération O(1) ultra-rapide qui contourne le décodage UTF-8.
        if bytes.len() > 2 && bytes[1] == b' ' {
            let status_char = bytes[0] as char;
            let path = &trimmed[2..];

            if is_status_char(status_char) {
                files.push(FileStatus {
                    status: status_char,
                    path: normalize_path(path),
                });
            }
        }
    }

    // Si les données de l'archive ont été extraites avec succès, on assemble et on renvoie l'entrée formatée
    wrapper.map(|w| format_log_entry(w.archive, files))
}

fn format_log_entry(archive: BorgArchive, files: Vec<FileStatus>) -> LogEntry {
    let mut count_added = 0;
    let mut count_modified = 0;
    let mut count_deleted = 0;
    let mut count_error = 0;

    // Optimisation : Comptage des statuts en un seul passage
    for f in &files {
        match f.status {
            'A' | 'a' => count_added += 1,
            'M' | 'm' => count_modified += 1,
            'D' | 'd' => count_deleted += 1,
            'E' | 'e' => count_error += 1,
            _ => {}
        }
    }

    LogEntry {
        id: archive.id,
        date: archive.end,
        duration: archive.duration,
        total_size: archive.stats.original_size,
        total_files: archive.stats.nfiles,
        // Ces drapeaux sont conservés en anglais car ils sont probablement utilisés par la logique du frontend
        status: if count_error > 0 {
            "Error".to_string()
        } else {
            "Success".to_string()
        },
        count_added,
        count_modified,
        count_deleted,
        count_error,
        files,
    }
}

fn is_status_char(c: char) -> bool {
    matches!(c, 'A' | 'M' | 'D' | 'd' | 'U' | 'E')
}

// Convertit les chemins Linux/WSL de type /mnt/c/ en chemins Windows de type C:/
fn normalize_path(path: &str) -> String {
    let bytes = path.as_bytes();

    // L'évaluation directe par octets ne peut pas planter et s'exécute en O(1), ignorant le décodage complet de la chaîne
    if path.starts_with("/mnt/") && bytes.len() >= 7 && bytes[6] == b'/' {
        let drive = bytes[5].to_ascii_uppercase() as char;
        let rest = &path[7..];
        return format!("{}:/{}", drive, rest);
    }

    path.to_string()
}
