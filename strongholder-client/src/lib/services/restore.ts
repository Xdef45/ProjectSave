import { invoke } from '@tauri-apps/api/core';

// --- Types Partagés ---

export interface ArchiveItem {
    archive: string;
    time: string;
}

export interface ArchiveFileRaw {
    type: '-' | 'd';
    path: string;
    mtime: string;
    size: number;
}

// --- Récupération des données ---

// Récupère la liste des archives disponibles sur le serveur de sauvegarde
export async function fetchArchivesList(): Promise<ArchiveItem[]> {
    try {
        return await invoke<ArchiveItem[]>('fetch_archives_list_req');
    } catch (e) {
        throw new Error('ARCHIVE_FETCH_FAILED', { cause: e });
    }
}

// Récupère la liste détaillée des fichiers contenus dans une archive spécifique
export async function fetchArchiveFiles(archiveName: string): Promise<ArchiveFileRaw[]> {
    try {
        return await invoke<ArchiveFileRaw[]>('fetch_archive_files_req', { archiveName });
    } catch (e) {
        throw new Error('FILE_FETCH_FAILED', { cause: e });
    }
}

// --- Opérations de restauration ---

// Ouvre une boîte de dialogue native pour demander à l'utilisateur où enregistrer l'archive
export async function askSavePath(archiveName: string): Promise<string | null> {
    try {
        return await invoke<string | null>('ask_save_path', { archiveName });
    } catch (e) {
        throw new Error('DIALOG_FAILED', { cause: e });
    }
}

// Télécharge l'archive et l'enregistre à l'emplacement choisi sur le disque
export async function downloadAndSaveArchive(archiveName: string, targetPath: string): Promise<string> {

    return await invoke<string>('download_and_save_archive_req', {
        archiveName,
        targetPath
    });

}

// Restaure les fichiers de l'archive directement à leurs emplacements d'origine sur le système
export async function restoreArchiveInPlace(archiveName: string): Promise<string> {

    return await invoke<string>('restore_to_original_req', { archiveName });

}

// Envoie un signal d'arrêt au backend Rust pour interrompre une restauration ou un téléchargement en cours
export async function cancelActiveRestore(): Promise<void> {
    try {
        await invoke('cancel_restore_operation');
    } catch {
        // Erreur ignorée silencieusement si le signal ne peut être transmis
    }
}

// --- Utilitaires Système ---

// Récupère la liste des lecteurs (C:\, D:\, etc.) pour l'explorateur de fichiers
export async function getSystemDrives(): Promise<string[]> {
    try {
        return await invoke<string[]>('get_drives');
    } catch {
        // Solution de repli simple si l'appel natif échoue
        return navigator.userAgent.includes('Win') ? ['C:\\'] : ['/'];
    }
}