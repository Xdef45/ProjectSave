<script lang="ts">
	import { onMount } from 'svelte';
	import { fade } from 'svelte/transition';
	import { t } from '$lib/i18n';

	// Importation des fonctions du service de configuration (assurant le lien avec Rust)
	import {
		loadAppConfig,
		saveAppConfig,
		requestAdminRestart,
		type AppConfig
	} from '$lib/services/config';

	// --- État de la vue ---
	let activeTab: 'general' | 'notifications' | 'network' = 'general';
	let isLoading = true;
	let isSaving = false;

	// Suivi des modifications pour savoir si on doit afficher un avertissement avant de quitter
	let hasChanges = false;
	let initialConfigStr = '';

	// Modales de confirmation
	let showUnsavedModal = false;
	let showRestartModal = false;

	// Promesse permettant de mettre en pause l'exécution en attendant le choix de l'utilisateur
	let resolveRestart: ((value: boolean) => void) | null = null;

	// Configuration par défaut (sera écrasée par celle du backend au chargement)
	let config: AppConfig = {
		general: {
			startup: true,
			admin: false,
			start_tray: false,
			minimize_tray: true,
			prevent_sleep: false,
			battery_limit: true
		},
		notifications: {
			on_start: true,
			on_success: true,
			on_warning: true,
			on_error: true,
			sound: false,
			on_client_issue: true
		},
		network: {
			upload_rate: 0
		}
	};

	// Variable temporaire pour l'affichage de la limite réseau en Mbps (convertie en Kbps/Bps côté backend)
	let uploadRateMb = 0;

	const TABS = ['general', 'notifications', 'network'] as const;

	// --- Initialisation ---
	onMount(async () => {
		try {
			// Chargement de la configuration depuis le backend (qui lit le JSON sur le disque)
			config = await loadAppConfig();

			// On sauvegarde l'état initial sous forme de chaîne pour détecter facilement les modifications
			initialConfigStr = JSON.stringify(config);

			// Conversion du taux brut (ex: ko/s) en Mo/s pour l'affichage dans le champ texte
			uploadRateMb = Math.max(0, Math.round((config.network.upload_rate || 0) / 1000));
		} finally {
			isLoading = false;
		}
	});

	// --- Utilitaires de réactivité ---

	// Vérifie si la configuration actuelle diffère de la configuration initiale chargée
	function checkDirty() {
		hasChanges = JSON.stringify(config) !== initialConfigStr;
	}

	// Bascule générique (Toggle) pour n'importe quelle propriété booléenne de la configuration
	// L'utilisation de types génériques (<K, S>) garantit la validité des propriétés passées
	function toggle<K extends keyof AppConfig, S extends keyof AppConfig[K]>(section: K, key: S) {
		if (typeof config[section][key] === 'boolean') {
			config = {
				...config,
				[section]: {
					...config[section],
					[key]: !config[section][key]
				}
			};
			checkDirty();
		}
	}

	// Met à jour la limite de bande passante en s'assurant que la valeur est un nombre positif
	function updateNetworkRate() {
		const safeInput = Math.max(0, Number(uploadRateMb) || 0);
		uploadRateMb = safeInput;

		// Reconversion en valeur brute pour le backend (ex: Mo vers Ko)
		config.network.upload_rate = Math.round(safeInput * 1000);
		checkDirty();
	}

	// --- Flux de redémarrage (Privilèges Administrateur) ---

	// Ouvre la modale de redémarrage et crée une Promesse qui attend que l'utilisateur clique sur un bouton
	function askToRestart(): Promise<boolean> {
		showRestartModal = true;
		return new Promise((resolve) => {
			resolveRestart = resolve;
		});
	}

	// Résout la promesse avec le choix de l'utilisateur (fermant ainsi la modale)
	function handleRestartChoice(shouldRestart: boolean) {
		showRestartModal = false;
		if (resolveRestart) {
			resolveRestart(shouldRestart);
			resolveRestart = null;
		}
	}

	// --- Sauvegarde et Navigation ---

	async function saveSettings() {
		if (isSaving) return;
		isSaving = true;

		try {
			// Analyse de l'état précédent pour détecter si l'option Administrateur vient d'être activée
			const parsedConfig = JSON.parse(initialConfigStr || '{}');
			const wasAdmin = parsedConfig?.general?.admin ?? false;
			const isAdmin = config.general.admin;

			// Envoi de la nouvelle configuration au backend Rust pour l'écriture sur le disque
			await saveAppConfig(config);

			// Si l'utilisateur demande les droits administrateur pour la première fois, l'application doit redémarrer
			if (!wasAdmin && isAdmin) {
				const shouldRestart = await askToRestart();
				if (shouldRestart) {
					await requestAdminRestart();
					return; // Si on redémarre, on arrête l'exécution ici
				}
			}

			// Mise à jour de la référence "initiale" pour considérer les modifications comme sauvegardées
			initialConfigStr = JSON.stringify(config);
			hasChanges = false;

			// Retour à la page précédente (ex: Tableau de bord)
			window.history.back();
		} catch (e) {
			alert($t('errors.save_failed'));
		} finally {
			isSaving = false;
		}
	}

	// Gère le clic sur le bouton "Retour". Affiche une alerte si des modifications ne sont pas sauvegardées.
	function goBack() {
		if (hasChanges) {
			showUnsavedModal = true;
		} else {
			window.history.back();
		}
	}

	// Forcer le retour en ignorant les modifications non sauvegardées
	function confirmLeave() {
		showUnsavedModal = false;
		window.history.back();
	}
</script>

{#if showUnsavedModal}
	<div
		class="modal-overlay fixed inset-0 z-50 flex items-center justify-center bg-black/50 backdrop-blur-sm"
		transition:fade={{ duration: 100 }}
		role="dialog"
		aria-modal="true"
	>
		<div class="modal-content bg-surface border-border w-96 rounded-xl border p-6 shadow-2xl">
			<h3 class="text-text-main mb-2 text-lg font-bold">{$t('settings.unsaved_title')}</h3>
			<p class="text-text-muted mb-6 text-sm">{$t('settings.unsaved_desc')}</p>
			<div class="flex justify-end gap-3">
				<button
					class="btn-action secondary rounded-lg px-4 py-2 transition-colors hover:bg-gray-100"
					on:click={() => (showUnsavedModal = false)}>{$t('common.stay')}</button
				>
				<button
					class="btn-action danger rounded-lg px-4 py-2 text-red-500 transition-colors hover:bg-red-50"
					on:click={confirmLeave}>{$t('common.leave')}</button
				>
			</div>
		</div>
	</div>
{/if}

{#if showRestartModal}
	<div
		class="modal-overlay fixed inset-0 z-50 flex items-center justify-center bg-black/50 backdrop-blur-sm"
		transition:fade={{ duration: 100 }}
		role="dialog"
		aria-modal="true"
	>
		<div class="modal-content bg-surface border-border w-96 rounded-xl border p-6 shadow-2xl">
			<div
				class="mb-4 flex h-12 w-12 items-center justify-center rounded-full bg-blue-100 text-blue-600"
			>
				<svg
					xmlns="http://www.w3.org/2000/svg"
					fill="none"
					viewBox="0 0 24 24"
					stroke-width="1.5"
					stroke="currentColor"
					class="h-6 w-6"
					><path
						stroke-linecap="round"
						stroke-linejoin="round"
						d="M16.023 9.348h4.992v-.001M2.985 19.644v-4.992m0 0h4.992m-4.993 0l3.181 3.183a8.25 8.25 0 0013.803-3.7M4.031 9.865a8.25 8.25 0 0113.803-3.7l3.181 3.182m0-4.991v4.99"
					/></svg
				>
			</div>
			<h3 class="mb-2 text-lg font-bold text-gray-800">{$t('settings.restart_title')}</h3>
			<p class="mb-6 text-sm text-gray-500">{$t('settings.restart_desc')}</p>
			<div class="flex justify-end gap-3">
				<button
					class="btn-action secondary rounded-lg px-4 py-2 transition-colors hover:bg-gray-100"
					on:click={() => handleRestartChoice(false)}>{$t('common.later')}</button
				>
				<button
					class="btn-action primary rounded-lg bg-blue-600 px-4 py-2 text-white transition-colors hover:bg-blue-700"
					on:click={() => handleRestartChoice(true)}>{$t('common.restart_now')}</button
				>
			</div>
		</div>
	</div>
{/if}

{#if isLoading}
	<div class="text-text-muted flex h-full items-center justify-center">{$t('common.loading')}</div>
{:else}
	<div
		class="bg-bg border-border flex h-[calc(100%-40px)] flex-col overflow-hidden rounded-2xl border shadow-sm"
	>
		<div class="bg-surface border-border shrink-0 border-b px-8 pt-6 pb-0">
			<h2 class="text-text-main mb-6 text-2xl font-bold">{$t('settings.title')}</h2>
			<div class="flex gap-8">
				{#each TABS as tab}
					<button
						class="border-b-2 pb-4 text-sm font-bold transition-colors {activeTab === tab
							? 'text-primary border-primary'
							: 'text-text-muted hover:text-text-main border-transparent'}"
						on:click={() => (activeTab = tab)}>{$t(`settings.tab_${tab}`)}</button
					>
				{/each}
			</div>
		</div>

		<div class="relative flex-1 overflow-y-auto p-8">
			{#if activeTab === 'general'}
				<div in:fade={{ duration: 200 }} class="mx-auto max-w-3xl space-y-6">
					<div class="settings-group">
						<h4 class="text-primary mb-4 text-xs font-bold tracking-wider uppercase">
							{$t('settings.general.header_startup')}
						</h4>
						<div class="settings-row">
							<div class="settings-label">
								<span class="settings-title">{$t('settings.general.startup_title')}</span>
								<span class="settings-desc">{$t('settings.general.startup_desc')}</span>
							</div>
							<button
								type="button"
								role="switch"
								aria-checked={config.general.startup}
								class="toggle-switch {config.general.startup ? 'active' : ''}"
								on:click={() => toggle('general', 'startup')}
							></button>
						</div>
						<div class="settings-row">
							<div class="settings-label">
								<span class="settings-title">{$t('settings.general.admin_title')}</span>
								<span class="settings-desc">{$t('settings.general.admin_desc')}</span>
							</div>
							<button
								type="button"
								role="switch"
								aria-checked={config.general.admin}
								class="toggle-switch {config.general.admin ? 'active' : ''}"
								on:click={() => toggle('general', 'admin')}
							></button>
						</div>
						<div class="settings-row">
							<div class="settings-label">
								<span class="settings-title">{$t('settings.general.start_tray_title')}</span>
								<span class="settings-desc">{$t('settings.general.start_tray_desc')}</span>
							</div>
							<button
								type="button"
								role="switch"
								aria-checked={config.general.start_tray}
								class="toggle-switch {config.general.start_tray ? 'active' : ''}"
								on:click={() => toggle('general', 'start_tray')}
							></button>
						</div>
						<div class="settings-row">
							<div class="settings-label">
								<span class="settings-title">{$t('settings.general.minimize_tray_title')}</span>
								<span class="settings-desc">{$t('settings.general.minimize_tray_desc')}</span>
							</div>
							<button
								type="button"
								role="switch"
								aria-checked={config.general.minimize_tray}
								class="toggle-switch {config.general.minimize_tray ? 'active' : ''}"
								on:click={() => toggle('general', 'minimize_tray')}
							></button>
						</div>
					</div>
					<div class="settings-group">
						<h4 class="text-primary mb-4 text-xs font-bold tracking-wider uppercase">
							{$t('settings.general.header_power')}
						</h4>
						<div class="settings-row">
							<div class="settings-label">
								<span class="settings-title">{$t('settings.general.prevent_sleep_title')}</span>
								<span class="settings-desc">{$t('settings.general.prevent_sleep_desc')}</span>
							</div>
							<button
								type="button"
								role="switch"
								aria-checked={config.general.prevent_sleep}
								class="toggle-switch {config.general.prevent_sleep ? 'active' : ''}"
								on:click={() => toggle('general', 'prevent_sleep')}
							></button>
						</div>
						<div class="settings-row">
							<div class="settings-label">
								<span class="settings-title">{$t('settings.general.battery_limit_title')}</span>
								<span class="settings-desc">{$t('settings.general.battery_limit_desc')}</span>
							</div>
							<button
								type="button"
								role="switch"
								aria-checked={config.general.battery_limit}
								class="toggle-switch {config.general.battery_limit ? 'active' : ''}"
								on:click={() => toggle('general', 'battery_limit')}
							></button>
						</div>
					</div>
				</div>
			{/if}

			{#if activeTab === 'notifications'}
				<div in:fade={{ duration: 200 }} class="mx-auto max-w-3xl space-y-6">
					<div class="settings-group">
						<h4 class="text-primary mb-4 text-xs font-bold tracking-wider uppercase">
							{$t('settings.notifications.header')}
						</h4>
						{#each [{ key: 'on_start', danger: false }, { key: 'on_success', danger: false }, { key: 'on_warning', danger: false }, { key: 'on_error', danger: true }, { key: 'on_client_issue', danger: false }, { key: 'sound', danger: false }] as item}
							<div class="settings-row">
								<div class="settings-label">
									<span class="settings-title {item.danger ? 'text-status-deleted font-bold' : ''}"
										>{$t(`settings.notifications.${item.key}`)}</span
									>
									<span class="settings-desc {item.danger ? 'text-status-deleted' : ''}"
										>{$t(`settings.notifications.${item.key}_desc`)}</span
									>
								</div>
								<button
									type="button"
									role="switch"
									aria-checked={config.notifications[item.key as keyof typeof config.notifications]}
									class="toggle-switch {config.notifications[
										item.key as keyof typeof config.notifications
									]
										? 'active'
										: ''}"
									on:click={() =>
										toggle('notifications', item.key as keyof typeof config.notifications)}
								></button>
							</div>
						{/each}
					</div>
				</div>
			{/if}

			{#if activeTab === 'network'}
				<div in:fade={{ duration: 200 }} class="mx-auto max-w-3xl space-y-6">
					<div class="settings-group">
						<h4 class="text-primary mb-4 text-xs font-bold tracking-wider uppercase">
							{$t('settings.network.header')}
						</h4>
						<div class="bg-bg border-border rounded-lg border p-6">
							<label class="text-text-main mb-4 block text-sm font-bold"
								>{$t('settings.network.upload_limit')}</label
							>
							<div class="flex items-center gap-6">
								<input
									type="range"
									min="0"
									max="1000"
									step="10"
									bind:value={uploadRateMb}
									on:input={updateNetworkRate}
									class="bg-border accent-primary h-2 flex-1 cursor-pointer appearance-none rounded-lg"
								/>
							</div>
							<div class="mt-2 text-right text-sm font-medium">
								{#if uploadRateMb === 0}
									<span class="text-status-added">∞ {$t('settings.network.unlimited')}</span>
								{:else}
									<span class="text-text-muted"
										>{$t('settings.network.limit_value', { rate: uploadRateMb })}</span
									>
								{/if}
							</div>
							<p class="text-text-muted mt-4 text-xs">
								<span class="font-bold">{$t('common.note')}:</span>
								{$t('settings.network.note')}
							</p>
						</div>
					</div>
				</div>
			{/if}
		</div>

		<div class="bg-surface border-border flex shrink-0 justify-end gap-3 border-t p-6">
			<button class="btn-action secondary" on:click={goBack}>{$t('common.cancel')}</button>
			<button
				class="btn-action primary"
				on:click={saveSettings}
				disabled={isSaving}
				class:opacity-50={isSaving}
			>
				{#if isSaving}{$t('common.saving')}{:else}{$t('common.save_changes')}{/if}
			</button>
		</div>
	</div>
{/if}
