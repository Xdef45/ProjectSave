import { invoke } from '@tauri-apps/api/core';

// --- Types de base ---

interface AuthPayload {
    username: string;
    password: string;
}

// --- Services d'API (Wrappers simples de communication avec Rust) ---

export async function authRequest(endpoint: 'signin' | 'signup', payload: AuthPayload): Promise<boolean> {
    const isSignup = endpoint === 'signup';
    await invoke('login_user', {
        username: payload.username,
        password: payload.password,
        isSignup: isSignup
    });
    return true;
}

export async function getRepoKey(): Promise<number[]> {
    return await invoke<number[]>('get_repo_key_req');
}

export async function sendSshKey(keyContent: string): Promise<void> {
    await invoke('send_ssh_key_req', { keyContent, isTunnel: true });
}

export async function sendBorgKey(keyContent: string): Promise<void> {
    await invoke('send_ssh_key_req', { keyContent, isTunnel: false });
}

export async function getClientId(): Promise<string> {
    return await invoke<string>('get_client_id_req');
}

export async function getServerSshKey(): Promise<string> {
    return await invoke<string>('get_server_ssh_key_req');
}

// --- Logique d'Orchestration Globale ---

// Gère le flux complet d'authentification et de provisionnement de l'environnement sécurisé
export async function orchestrateLoginFlow(
    payload: AuthPayload,
    isRegistering: boolean,
    onProgress: (translationKey: string) => void
): Promise<void> {

    // 1. Authentification de l'utilisateur auprès du serveur distant
    onProgress('login.process.authenticating');
    const endpoint = isRegistering ? 'signup' : 'signin';
    await authRequest(endpoint, payload);
    localStorage.setItem('username', payload.username);

    // 2. Récupération de l'identifiant unique lié à cet ordinateur/client
    onProgress('login.process.retrieving_id');
    const clientId = await getClientId();
    localStorage.setItem('client_id', clientId);

    // --- BLOC DE SÉCURITÉ : GESTION DYNAMIQUE DE SSH ---
    // Le service SSH interne est allumé uniquement le temps de la configuration
    // pour réduire drastiquement la surface d'attaque du système de l'utilisateur.
    onProgress('login.process.securing_connection');
    let weStartedSsh = false;

    try {
        const isSshRunning = await invoke<boolean>('check_ssh_running');
        if (!isSshRunning) {
            await invoke('start_ssh_service');
            weStartedSsh = true;
        }

        // 3. Préparation du compte utilisateur dans l'environnement Linux (WSL)
        onProgress('login.process.init_env');
        try {
            await invoke('wsl_setup_user', { username: payload.username });
        } catch (wslErr) {
            throw new Error('WSL_USER_CREATION_FAILED', { cause: wslErr });
        }

        // 4. Copie et exécution des scripts de configuration matérielle et de sauvegarde
        onProgress('login.process.config_protocols');
        try {
            await invoke('wsl_provision_scripts', {
                username: payload.username,
                clientId: clientId
            });
        } catch (scriptErr) {
            throw new Error('WSL_SCRIPT_ERROR', { cause: scriptErr });
        }

        // 5. Récupération et synchronisation de la clé SSH dédiée au tunnel de connexion
        onProgress('login.process.establish_tunnel');
        const sshKeyContent = await fetchKeyWithRetry('get_tunnel_ssh_key', payload.username, clientId, onProgress);
        await sendSshKey(sshKeyContent);

        // 6. Récupération et synchronisation de la clé SSH dédiée à l'outil Borg Backup
        onProgress('login.process.establish_borg');
        const sshBorgKeyContent = await fetchKeyWithRetry('get_borg_ssh_key', payload.username, clientId, onProgress);
        await sendBorgKey(sshBorgKeyContent);

        // 7. Enregistrement de la clé publique du serveur cible 
        // (Prévient les attaques de type "Man-in-the-Middle")
        onProgress('login.process.verify_server');
        const serverSshKey = await getServerSshKey();
        try {
            await invoke('save_server_ssh_key', { username: payload.username, sshKey: serverSshKey });
        } catch (saveErr) {
            throw new Error('SERVER_KEY_SAVE_FAILED', { cause: saveErr });
        }

        // 8. Configuration finale des paramètres du client Borg
        onProgress('login.process.setup_backup');
        try {
            await invoke('wsl_configure_borg_client', { username: payload.username });
        } catch (borgErr) {
            throw new Error('BORG_KEY_INSTALL_FAILED', { cause: borgErr });
        }

        // 9. Chiffrement : récupération et stockage sécurisé de la clé maître du dépôt
        onProgress('login.process.finalize_encryption');
        const repoKey = await getRepoKey();
        if (!repoKey || repoKey.length === 0) throw new Error('KEY_MISSING');

        try {
            await invoke('save_master_key', {
                username: payload.username,
                clientId: clientId,
                key: repoKey
            });
        } catch (invokeErr) {
            throw new Error('KEY_SAVE_FAILED', { cause: invokeErr });
        }

    } finally {
        // Quoi qu'il arrive (succès ou échec critique), on referme le service SSH 
        // si c'est nous qui l'avions démarré à l'étape 2.
        if (weStartedSsh) {
            try {
                await invoke('stop_ssh_service');
            } catch {
                // Les erreurs de fermeture SSH sont ignorées silencieusement
            }
        }
    }

    onProgress('login.process.welcome');
}

/**
 * Fonction utilitaire pour récupérer une clé générée dans WSL.
 * Comme l'écriture de fichiers depuis WSL vers Windows peut accuser un très léger 
 * délai de synchronisation (I/O delay), cette fonction intègre un mécanisme de 
 * réessai automatique avec une pause d'une seconde.
 */
async function fetchKeyWithRetry(
    command: string,
    username: string,
    clientId: string,
    onProgress: (k: string) => void
): Promise<string> {
    try {
        return await invoke<string>(command, { username, clientId });
    } catch {
        onProgress('login.process.retrying');
        try {
            // Pause bloquante de 1000ms avant de relancer la tentative
            await new Promise(r => setTimeout(r, 1000));
            return await invoke<string>(command, { username, clientId });
        } catch (retryErr) {
            throw new Error('SSH_KEY_MISSING_AFTER_RETRY', { cause: retryErr });
        }
    }
}