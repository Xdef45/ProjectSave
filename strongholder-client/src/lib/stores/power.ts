import { writable } from 'svelte/store';
import { invoke } from '@tauri-apps/api/core';

// Valeur par défaut à 'false' pour éviter de bloquer l'application au démarrage.
export const isLowBattery = writable<boolean>(false);

/**
 * Interroge le backend Rust pour vérifier si l'appareil fonctionne sur batterie
 * ET si le niveau de charge est en dessous du seuil critique.
 */
export async function checkPowerStatus(): Promise<void> {
    try {
        const low = await invoke<boolean>('is_low_battery');
        isLowBattery.set(low);
    } catch {
        // En cas d'erreur (ex: PC fixe sans batterie), on assume que l'alimentation
        // est stable sur secteur pour ne pas empêcher les opérations.
        isLowBattery.set(false);
    }
}

/**
 * Initialise le service de surveillance de l'alimentation en arrière-plan.
 * Vérifie l'état du système toutes le 30 secondes.
 */
export function startPowerMonitoring(): void {
    // 1. Vérification initiale au lancement
    checkPowerStatus();

    // 2. Sondage périodique (intervalle de 30s)
    setInterval(checkPowerStatus, 30000);
}