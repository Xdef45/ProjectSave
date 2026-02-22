import { writable } from 'svelte/store';
import { invoke } from '@tauri-apps/api/core';

// Valeur par défaut à 'true' pour éviter un flash visuel "Hors ligne" au démarrage
export const isOnline = writable<boolean>(true);

/**
 * Vérifie la connectivité internet réelle via le backend Rust.
 * Cette méthode contourne les restrictions CORS du navigateur et garantit 
 * que l'accès au réseau est fonctionnel (pas seulement connecté au WiFi).
 */
export async function checkConnection(): Promise<boolean> {

    // Le backend Rust effectue un ping léger vers un serveur de référence
    const result = await invoke<boolean>('check_internet_connection');

    // Mise à jour du store réactif
    isOnline.set(result);
    return result;

}

/**
 * Initialise le service de surveillance de la connexion.
 * Stratégie :
 * 1. Vérification immédiate au lancement.
 * 2. Écouteurs d'événements pour les changements d'état système.
 * 3. Vérification périodique (60s) pour détecter les coupures silencieuses.
 */
export function startConnectionMonitoring(): void {
    // 1. Vérification initiale
    checkConnection();

    // 2. Écouteurs d'événements navigateur (Réaction rapide)
    // Se déclenchent instantanément lors du débranchement d'un câble ou de la coupure du WiFi
    window.addEventListener('offline', () => {
        isOnline.set(false);
    });

    // Lorsque le système signale un retour en ligne, on vérifie avec Rust
    // pour s'assurer que l'accès à Internet est bien réel.
    window.addEventListener('online', () => {
        checkConnection();
    });

    // 3. Sondage périodique (Filet de sécurité en arrière-plan)
    setInterval(() => {
        checkConnection();
    }, 60_000);
}