import { BaseDirectory, writeTextFile, readTextFile, exists, mkdir } from '@tauri-apps/plugin-fs';
import { invoke } from '@tauri-apps/api/core';
import { appConfigDir, join } from '@tauri-apps/api/path';

// --- Structures de données ---

export interface ExtensionRule {
    pattern: string;
    mode: 'include' | 'exclude';
}

export interface BackupPreset {
    id: string;
    name: string;
    frequency: 'manual' | 'hourly' | 'daily' | 'weekly' | 'monthly';
    scheduleTime: string; // Format "HH:MM"
    scheduleDay: string;
    sources: string[];
    exclusions: string[];
    extensionRules: ExtensionRule[];
    includeOnlyMode: boolean;
    paused?: boolean;
}

// --- Usine (Factory) ---

export const createEmptyPreset = (): BackupPreset => ({
    id: crypto.randomUUID(),
    name: 'New Preset',
    frequency: 'manual',
    scheduleTime: '12:00',
    scheduleDay: 'Monday',
    sources: [],
    exclusions: [],
    extensionRules: [],
    includeOnlyMode: false,
    paused: false
});

// --- Utilitaires de normalisation des chemins ---

// Convertit les chemins Windows en chemins POSIX compatibles WSL (/mnt/c/...)
const formatPathForPatternFile = (rawPath: string): string => {
    const isWindows = navigator.userAgent.includes('Windows');
    let cleanPath = rawPath.trim();

    if (isWindows) {
        cleanPath = cleanPath.replace(/\\/g, '/');
        if (/^[a-zA-Z]:/.test(cleanPath)) {
            const drive = cleanPath.charAt(0).toLowerCase();
            const rest = cleanPath.slice(2);
            cleanPath = `/mnt/${drive}${rest}`;
        }
    }

    // Échappement des guillemets simples pour la compatibilité shell POSIX
    return cleanPath.replace(/'/g, "'\"'\"'");
};

// --- Génération du contenu du fichier de motifs ---

export const generatePatternFileContent = (preset: BackupPreset): string => {
    const lines: string[] = [`# Fichier de motifs pour : ${preset.name}`];

    // Ajout des sources (R = Root)
    preset.sources.forEach(src => {
        lines.push(`R ${formatPathForPatternFile(src)}`);
    });

    // Ajout des exclusions explicites (!)
    preset.exclusions.forEach(exc => {
        lines.push(`! ${formatPathForPatternFile(exc)}`);
    });

    // Gestion des règles par extension
    preset.extensionRules.forEach(rule => {
        const recursivePattern = rule.pattern.replace('*.', '**/*.');

        if (preset.includeOnlyMode) {
            if (rule.mode === 'include') lines.push(`+ ${recursivePattern}`);
        } else {
            if (rule.mode === 'exclude') lines.push(`- ${recursivePattern}`);
        }
    });

    // Si on est en mode "Inclusion uniquement", on exclut tout le reste à la fin
    if (preset.includeOnlyMode) {
        lines.push(`- **`);
    }

    return lines.join('\n') + '\n';
};

// Nettoie le nom du préréglage pour en faire un nom de fichier valide sur le disque
export const getSafeFilename = (name: string): string => {
    const safeName = name
        .replace(/[^a-zA-Z0-9_-]/g, '_')
        .replace(/_+/g, '_')
        .slice(0, 50);

    return `patterns_${safeName || 'unnamed'}.lst`;
};

// --- Logique de Persistance ---

export async function loadPresetsFromDisk(): Promise<BackupPreset[]> {
    try {
        const fileExists = await exists('presets.json', { baseDir: BaseDirectory.AppConfig });
        if (!fileExists) return [];

        const content = await readTextFile('presets.json', { baseDir: BaseDirectory.AppConfig });
        return JSON.parse(content);
    } catch {
        return [];
    }
}

// Sauvegarde l'intégralité des préréglages et génère les fichiers .lst correspondants
export async function savePresetsToDisk(presets: BackupPreset[]): Promise<void> {

    const dirExists = await exists('', { baseDir: BaseDirectory.AppConfig });
    if (!dirExists) {
        await mkdir('', { baseDir: BaseDirectory.AppConfig, recursive: true });
    }

    await writeTextFile('presets.json', JSON.stringify(presets, null, 2), {
        baseDir: BaseDirectory.AppConfig
    });

    const writePromises = presets.map(preset => {
        const filename = getSafeFilename(preset.name);
        const content = generatePatternFileContent(preset);

        return writeTextFile(filename, content, {
            baseDir: BaseDirectory.AppConfig
        });
    });

    await Promise.all(writePromises);

}

// Sauvegarde optimisée : ne met à jour que le fichier de motifs du préréglage modifié
export async function saveSinglePresetToDisk(presets: BackupPreset[], changedPreset: BackupPreset): Promise<void> {
    const dirExists = await exists('', { baseDir: BaseDirectory.AppConfig });
    if (!dirExists) {
        await mkdir('', { baseDir: BaseDirectory.AppConfig, recursive: true });
    }

    await writeTextFile('presets.json', JSON.stringify(presets, null, 2), {
        baseDir: BaseDirectory.AppConfig
    });

    const filename = getSafeFilename(changedPreset.name);
    const content = generatePatternFileContent(changedPreset);

    await writeTextFile(filename, content, {
        baseDir: BaseDirectory.AppConfig
    });
}

// --- Validation et Planification ---

export const validateExtensionRule = (
    currentRules: ExtensionRule[],
    newPattern: string
): boolean => {
    const conflict = currentRules.find(r =>
        r.pattern.toLowerCase() === newPattern.toLowerCase()
    );
    return !conflict;
};

// Génère une chaîne au format Crontab standard à partir des réglages du préréglage
export function getCronString(preset: BackupPreset): string {
    const timeParts = preset.scheduleTime.split(':');
    const hours = Math.max(0, Math.min(23, Number(timeParts[0]) || 0));
    const minutes = Math.max(0, Math.min(59, Number(timeParts[1]) || 0));

    switch (preset.frequency) {
        case 'hourly':
            return `0 * * * *`;
        case 'daily':
            return `${minutes} ${hours} * * *`;
        case 'weekly': {
            const days: Record<string, number> = {
                'monday': 1, 'tuesday': 2, 'wednesday': 3,
                'thursday': 4, 'friday': 5, 'saturday': 6, 'sunday': 0
            };
            const dayKey = String(preset.scheduleDay).toLowerCase();
            const dayNum = days[dayKey] ?? '*';
            return `${minutes} ${hours} * * ${dayNum}`;
        }
        case 'monthly': {
            const safeDay = Math.max(1, Math.min(31, Number(preset.scheduleDay) || 1));
            return `${minutes} ${hours} ${safeDay} * *`;
        }
        default:
            return '';
    }
}

// --- Action de synchronisation système ---

/**
 * Synchronise l'état du préréglage avec le Crontab du système via le backend Rust.
 */
export async function syncPresetScheduleToSystem(preset: BackupPreset): Promise<void> {
    const clientId = localStorage.getItem('client_id');
    const username = localStorage.getItem('username');

    if (!clientId || !username) return;

    const configDir = await appConfigDir();
    const filename = getSafeFilename(preset.name);
    const fullPath = await join(configDir, filename);
    const cronStr = getCronString(preset);

    const isEnabled = preset.frequency !== 'manual' && !preset.paused;

    try {
        await invoke('update_backup_schedule', {
            username,
            presetId: preset.id,
            cronString: cronStr,
            presetPath: fullPath,
            clientId,
            enabled: isEnabled
        });
    } catch (e) {
        throw new Error('SCHEDULE_SYNC_FAILED', { cause: e });
    }
}