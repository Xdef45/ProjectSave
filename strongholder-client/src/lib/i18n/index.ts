import { writable, derived } from 'svelte/store';
import en from './en.json';
import fr from './fr.json';
import es from './es.json';
import it from './it.json';
import de from './de.json';

// --- TYPES ---

// Recursive type: A node can be a string OR a nested object of nodes
type TranslationNode = string | { [key: string]: TranslationNode };

// Helper type for interpolation variables
type InterpolationParams = Record<string, string | number>;

// --- CONFIGURATION ---

const FALLBACK_LOCALE = 'en';

// We cast the imported JSON to our recursive type so the traverser can handle it genericallly
const translations: Record<string, TranslationNode> = {
    en: en as TranslationNode,
    fr: fr as TranslationNode,
    es: es as TranslationNode,
    it: it as TranslationNode,
    de: de as TranslationNode,
};

// --- STORES ---

export const locale = writable<string>(FALLBACK_LOCALE);

export const t = derived(locale, ($locale) => {
    return (key: string, vars: InterpolationParams = {}): string => {
        const currentDict = translations[$locale] || translations[FALLBACK_LOCALE];

        if (!currentDict) {
            console.warn(`[i18n] Critical: No translations found for ${$locale}`);
            return key;
        }

        const keys = key.split('.');

        let node: TranslationNode | undefined = currentDict;

        for (const k of keys) {
            if (node && typeof node === 'object' && k in node) {
                node = (node as Record<string, TranslationNode>)[k];
            } else {
                return key;
            }
        }

        if (typeof node !== 'string') {
            return key;
        }

        if (Object.keys(vars).length > 0) {
            return node.replace(/{(\w+)}/g, (match, v) => {
                return vars[v] !== undefined ? String(vars[v]) : match;
            });
        }

        return node;
    };
});