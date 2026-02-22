import { invoke } from '@tauri-apps/api/core';

// --- Types Partagés ---

export interface FileStatus {
    readonly path: string;
    readonly status: string;
}

export interface LogEntry {
    readonly id: string;
    readonly date: string;
    readonly duration: number;
    readonly total_size: number;
    readonly total_files: number;
    readonly status: string;
    readonly count_added: number;
    readonly count_modified: number;
    readonly count_deleted: number;
    readonly count_error: number;
    readonly files: FileStatus[];

    // --- État local pour l'interface utilisateur uniquement ---
    // Ces propriétés ne viennent pas de Rust, elles servent à la gestion Svelte
    isOpen: boolean;
    filterTerm: string;
    filterState: string;
}

export interface DashboardLogEntry {
    readonly id: string;
    readonly date: string;
    readonly total_size: number;
    readonly total_files: number;
    readonly status: string;
    readonly count_added: number;
    readonly count_modified: number;
    readonly count_deleted: number;
    readonly count_error: number;
}

// --- Méthodes de Service ---

/**
 * Récupère l'historique complet des sauvegardes depuis le backend Rust.
 * Note de performance : Le parsing lourd est géré par Rust ; 
 * le JS s'occupe uniquement du mapping final pour l'interface.
 */
export async function getBackupLogs(): Promise<LogEntry[]> {
    try {
        const rawLogs = await invoke<LogEntry[]>('get_logs_req') ?? [];

        if (!Array.isArray(rawLogs)) {
            return [];
        }

        // Initialisation de l'état local pour chaque entrée de journal
        return rawLogs.map(log => ({
            ...log,
            isOpen: false,
            filterTerm: '',
            filterState: 'all'
        }));

    } catch (error) {
        throw new Error('LOG_FETCH_FAILED', { cause: error });
    }
}

/**
 * Récupère les statistiques de la dernière sauvegarde pour le tableau de bord.
 * Retourne l'entrée la plus récente ou null si aucune sauvegarde n'existe.
 */
export async function getDashboardStats(): Promise<DashboardLogEntry | null> {
    try {
        const logs = await invoke<DashboardLogEntry[]>('get_backup_logs');
        if (logs && logs.length > 0) {
            return logs[0];
        }
        return null;
    } catch (error) {
        throw new Error('DASHBOARD_STATS_FAILED', { cause: error });
    }
}