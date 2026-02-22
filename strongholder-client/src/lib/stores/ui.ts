import { writable } from 'svelte/store';

// Valeur par défaut à 'true' (Ouvert) pour les environnements de bureau.
// On pourrait vérifier 'window.innerWidth' ici pour passer à 'false' sur mobile.
export const isSidebarOpen = writable<boolean>(true);

/**
 * Alterne l'état de visibilité de la barre latérale.
 */
export function toggleSidebar(): void {
    isSidebarOpen.update(v => !v);
}

/**
 * Ferme explicitement la barre latérale.
 * Utile pour les événements de clic à l'extérieur ou la navigation sur mobile.
 */
export function closeSidebar(): void {
    isSidebarOpen.set(false);
}

/**
 * Ouvre explicitement la barre latérale.
 */
export function openSidebar(): void {
    isSidebarOpen.set(true);
}