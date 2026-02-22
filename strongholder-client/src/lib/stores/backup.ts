import { writable } from 'svelte/store';
import { listen } from '@tauri-apps/api/event';
import { invoke } from '@tauri-apps/api/core';

// --- Types ---
export type BackupState = 'idle' | 'preparing' | 'running' | 'error' | 'success';

export interface BackupStatus {
    presetId: string | null;
    state: BackupState;
    progress: number;
    currentFile: string;
    logs: string[];
}

const INITIAL_STATUS: BackupStatus = {
    presetId: null,
    state: 'idle',
    progress: 0,
    currentFile: '',
    logs: []
};

// Store réactif Svelte pour l'état de la sauvegarde
export const backupStatus = writable<BackupStatus>(INITIAL_STATUS);

// --- État interne ---
let isListening = false;
let resetTimer: ReturnType<typeof setTimeout> | null = null;
let isBackupCancelled = false;

// --- Logique de l'Écouteur d'Événements ---

/**
 * Initialise l'écouteur global qui réceptionne les événements envoyés par le script de sauvegarde Rust.
 * Permet de mettre à jour l'interface en temps réel (progression, fichiers, erreurs).
 */
export async function initBackupListener(): Promise<() => void> {
    if (isListening) return () => { };

    isListening = true;

    const unlisten = await listen<{ event_type: string; data: string }>('backup-event', (event) => {
        const { event_type, data } = event.payload;

        backupStatus.update((s) => {
            switch (event_type) {
                case 'progress': {
                    const activeState = s.state === 'idle' ? 'running' : s.state;
                    const parsedProgress = parseInt(data, 10);
                    return {
                        ...s,
                        state: activeState,
                        progress: isNaN(parsedProgress) ? s.progress : parsedProgress
                    };
                }
                case 'file':
                    return { ...s, currentFile: data };

                case 'success':
                    // Réinitialisation automatique de l'interface après 5 secondes en cas de succès
                    if (resetTimer) clearTimeout(resetTimer);
                    resetTimer = setTimeout(() => {
                        backupStatus.set(INITIAL_STATUS);
                        resetTimer = null;
                    }, 5000);

                    return {
                        ...s,
                        state: 'success',
                        progress: 100,
                        currentFile: 'Sauvegarde terminée avec succès'
                    };

                case 'error':
                    return {
                        ...s,
                        state: 'error',
                        logs: [...s.logs, `Erreur : ${data}`]
                    };

                case 'log':
                    // On conserve uniquement les 50 derniers messages de log pour les performances
                    return {
                        ...s,
                        logs: [...s.logs.slice(-49), data]
                    };

                default:
                    return s;
            }
        });
    });

    return () => {
        isListening = false;
        unlisten();
    };
}

// --- Actions de Service (Couche de commande) ---

/**
 * Lance l'exécution d'un préréglage de sauvegarde via le backend Rust.
 */
export async function executeBackup(
    presetId: string,
    presetName: string,
    presetPath: string,
    clientId: string,
    username: string,
    cancelMessage: string,
    errorMessagePrefix: string,
    unknownErrorMessage: string
): Promise<void> {

    // Annulation de tout minuteur de réinitialisation en cours
    if (resetTimer) {
        clearTimeout(resetTimer);
        resetTimer = null;
    }

    isBackupCancelled = false;
    backupStatus.set({
        ...INITIAL_STATUS,
        state: 'preparing',
        presetId: presetId,
        currentFile: 'Initialisation...'
    });

    // S'assure que l'écouteur est bien actif avant de lancer le script
    await initBackupListener();

    try {
        await invoke('run_backup_script', {
            clientId,
            presetPath,
            username
        });
    } catch (e: unknown) {
        if (isBackupCancelled) {
            backupStatus.update((s) => ({ ...s, state: 'error', currentFile: cancelMessage }));
        } else {
            let safeErrorMsg = unknownErrorMessage;
            if (typeof e === 'string') safeErrorMsg = e;
            else if (e instanceof Error) safeErrorMsg = e.message;
            else if (e && typeof e === 'object' && 'message' in e) safeErrorMsg = String(e.message);

            backupStatus.update((s) => ({
                ...s,
                state: 'error',
                currentFile: `${errorMessagePrefix}: ${safeErrorMsg}`
            }));

            throw new Error(safeErrorMsg);
        }
    }
}

/**
 * Interrompt violemment le processus de sauvegarde en cours au niveau de l'OS.
 */
export async function cancelActiveBackup(cancelMessage: string): Promise<void> {
    isBackupCancelled = true;


    await invoke('cancel_backup');
    backupStatus.update((s) => ({
        ...s,
        state: 'error',
        currentFile: cancelMessage
    }));

}