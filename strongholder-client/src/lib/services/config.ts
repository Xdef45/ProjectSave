import { invoke } from '@tauri-apps/api/core';

// --- Types ---
// Cette interface doit correspondre très exactement aux structures Rust (AppConfig)
// pour que la désérialisation fonctionne sans erreur côté backend.
export interface AppConfig {
    general: {
        startup: boolean;
        admin: boolean;
        start_tray: boolean;
        minimize_tray: boolean;
        prevent_sleep: boolean;
        battery_limit: boolean;
    };
    notifications: {
        on_start: boolean;
        on_success: boolean;
        on_warning: boolean;
        on_error: boolean;
        sound: boolean;
        on_client_issue: boolean;
    };
    network: {
        upload_rate: number;
    };
}

// --- Services de Configuration ---

// Demande au backend Rust de lire le fichier de configuration local et de nous le renvoyer
export async function loadAppConfig(): Promise<AppConfig> {
    try {
        return await invoke<AppConfig>('load_config');
    } catch (error) {
        throw new Error('CONFIG_LOAD_FAILED', { cause: error });
    }
}

// Transmet la nouvelle configuration au backend pour qu'elle soit sauvegardée de manière atomique
export async function saveAppConfig(config: AppConfig): Promise<void> {
    // SÉCURITÉ : Clonage profond de l'objet pour détruire les éventuels Proxys réactifs 
    // injectés par Svelte. Tauri/Rust a besoin d'un objet pur (Plain Old JavaScript Object).
    const cleanPayload = JSON.parse(JSON.stringify(config));

    // SÉCURITÉ : On s'assure que la limite d'envoi réseau est un entier positif ou nul.
    // Cela évite un plantage côté Rust, qui s'attend strictement à recevoir un entier non signé (u32).
    cleanPayload.network.upload_rate = Math.max(0, Math.floor(Number(cleanPayload.network.upload_rate) || 0));

    try {
        await invoke('save_config', { config: cleanPayload });
    } catch (error) {
        throw new Error('CONFIG_SAVE_FAILED', { cause: error });
    }
}

// Relance l'application en demandant les droits administrateur à l'utilisateur du système
export async function requestAdminRestart(): Promise<void> {
    try {
        await invoke('restart_as_admin');
    } catch (error) {
        throw new Error('ADMIN_RESTART_FAILED', { cause: error });
    }
}