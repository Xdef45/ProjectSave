<script lang="ts">
	import '../app.css';
	import { onMount, onDestroy } from 'svelte';
	import { invoke } from '@tauri-apps/api/core';
	import { listen } from '@tauri-apps/api/event';
	import { fade, fly, slide } from 'svelte/transition';

	import { t } from '$lib/i18n';

	import TitleBar from '$lib/components/TitleBar.svelte';
	import InternetStatus from '$lib/components/InternetStatus.svelte';
	import LanguageSwitcher from '$lib/components/LanguageSwitcher.svelte';
	import ThemeToggle from '$lib/components/ThemeToggle.svelte';

	// --- État global de l'interface ---
	// Représente la machine à états du démarrage de l'application
	let status: 'checking' | 'loading' | 'ready' | 'reboot_needed' | 'error' | 'storage_error' =
		'checking';

	// --- État des textes et de l'interface utilisateur ---
	let currentStepKey = 'startup.initializing';
	let detailedLog = '...';
	let waitMessageKey = 'startup.wait.privacy';
	let stepIndex = 0;

	// --- Gestionnaire de messages d'attente ---
	let messageInterval: ReturnType<typeof setInterval> | null = null;

	// Clés de traduction pour faire défiler des messages rassurants pendant les longs chargements
	const friendlyMessagesKeys = [
		'startup.wait.minutes',
		'startup.wait.securing',
		'startup.wait.configuring',
		'startup.wait.coffee'
	];

	// Fonction de nettoyage pour l'écouteur d'événements Tauri
	let unlistenFn: (() => void) | null = null;

	// Cycle de vie : Exécuté au moment où le composant est inséré dans le DOM
	onMount(() => {
		const init = async () => {
			// Écoute les journaux émis par le backend Rust pour mettre à jour l'interface en temps réel
			unlistenFn = await listen<string>('setup_log', (event) => {
				detailedLog = event.payload;
				updateStepTitle(event.payload);
			});
			await runSystemChecks();
		};

		init();
	});

	// Cycle de vie : Exécuté quand le composant est détruit (nettoyage de la mémoire)
	onDestroy(() => {
		if (unlistenFn) unlistenFn();
		stopMessageCycle();
	});

	// Modifie dynamiquement le titre de l'étape en fonction du contenu du journal renvoyé par Rust
	function updateStepTitle(log: string) {
		const logLower = log.toLowerCase();
		if (logLower.includes('downloading ubuntu')) currentStepKey = 'startup.step.downloading_ubuntu';
		else if (logLower.includes('importing')) currentStepKey = 'startup.step.decompressing';
		else if (logLower.includes('cleaning')) currentStepKey = 'startup.step.preparing_dir';
		else if (logLower.includes('ssh')) currentStepKey = 'startup.step.config_ssh';
	}

	// Fait tourner les messages d'attente toutes les 6 secondes
	function startMessageCycle() {
		if (messageInterval) return;
		let i = 0;
		messageInterval = setInterval(() => {
			i = (i + 1) % friendlyMessagesKeys.length;
			waitMessageKey = friendlyMessagesKeys[i];
		}, 6000);
	}

	function stopMessageCycle() {
		if (messageInterval) {
			clearInterval(messageInterval);
			messageInterval = null;
		}
	}

	// --- LOGIQUE PRINCIPALE ---
	// Vérifie l'état du système et lance les installations manquantes si nécessaire
	async function runSystemChecks() {
		try {
			// ÉTAPE 1 : Pré-requis critiques (Espace disque)
			const hasSpace = await invoke<boolean>('check_disk_space');

			if (!hasSpace) {
				status = 'storage_error';
				return;
			}

			// ÉTAPE 2 : Vérifications système et installation

			// 2.1 Moteur WSL
			const wslInstalled = await invoke<boolean>('check_wsl_installed');

			if (!wslInstalled) {
				switchToLoadingMode('startup.step.config_windows', 1);
				await invoke('install_wsl_engine');
				status = 'reboot_needed';
				return;
			}

			// 2.2 Distribution Ubuntu
			const ubuntuInstalled = await invoke<boolean>('check_ubuntu_installed');

			if (!ubuntuInstalled) {
				switchToLoadingMode('startup.step.setup_env', 2);
				await invoke('install_ubuntu_silent');
			}

			// 2.3 Paquet SSH
			const sshInstalled = await invoke<boolean>('check_ssh_installed');

			if (!sshInstalled) {
				switchToLoadingMode('startup.step.config_network', 3);
				await invoke('install_ssh_silent');
			}

			// Si toutes les vérifications sont passées, on autorise l'accès à l'application complète
			stopMessageCycle();
			status = 'ready';
		} catch (error: unknown) {
			currentStepKey = 'startup.error.failed_title';

			// Extraction robuste de l'erreur renvoyée par le backend Rust
			if (typeof error === 'string') {
				detailedLog = error;
			} else if (error instanceof Error) {
				detailedLog = error.message;
			} else if (error && typeof error === 'object' && 'message' in error) {
				detailedLog = String(error.message);
			} else {
				detailedLog = 'Une erreur native inattendue est survenue.';
			}

			status = 'error';
			stopMessageCycle();
		}
	}

	// Bascule l'interface sur l'écran de chargement animé avec le bon numéro d'étape
	function switchToLoadingMode(stepKey: string, stepIdx: number) {
		if (status !== 'loading') {
			status = 'loading';
			startMessageCycle();
		}
		currentStepKey = stepKey;
		stepIndex = stepIdx;
	}

	function restartPC() {
		invoke('restart_computer');
	}

	// Bloque le clic droit globalement, sauf pour les éléments nécessitant nativement un copier/coller
	function disableContextMenu(e: MouseEvent) {
		const target = e.target as HTMLElement;
		if (target.closest('input, textarea, [contenteditable="true"]')) return;
		e.preventDefault();
	}
</script>

<svelte:window on:contextmenu={disableContextMenu} />

<div class="relative z-[100000]">
	<TitleBar />
	<InternetStatus />
</div>

{#if status === 'storage_error'}
	<div
		class="bg-background fixed inset-0 z-[50] flex flex-col items-center justify-center p-8 text-center"
		in:fade
	>
		<div
			class="bg-surface-secondary/30 border-status-error/20 w-full max-w-md rounded-xl border p-8"
		>
			<div class="text-status-error mb-6 flex justify-center text-5xl">💾</div>
			<h2 class="text-text-main mb-4 text-2xl font-bold">
				{$t('startup.storage.title')}
			</h2>
			<p class="text-text-muted mb-6">
				{$t('startup.storage.desc_pre')}
				<span class="text-primary font-bold">3 GB</span>
				{$t('startup.storage.desc_post')}
			</p>
			<div class="text-text-muted mb-8 rounded bg-black/40 p-4 text-sm">
				{$t('startup.storage.action_hint')}
			</div>
			<button
				class="btn-action secondary w-full justify-center py-3"
				on:click={() => window.location.reload()}
			>
				{$t('common.check_again')}
			</button>
		</div>
	</div>
{:else if status === 'checking'}
	<div
		class="bg-background fixed inset-0 z-[50] flex flex-col items-center justify-center transition-opacity duration-500"
		out:fade
	>
		<div class="relative flex flex-col items-center justify-center">
			<div class="relative h-12 w-12 opacity-80">
				<div
					class="border-surface-secondary absolute inset-0 rounded-full border-2 opacity-30"
				></div>
				<div
					class="border-t-primary absolute inset-0 animate-spin rounded-full border-2 border-r-transparent border-b-transparent border-l-transparent"
				></div>
			</div>
			<p class="text-text-muted mt-4 animate-pulse text-sm font-medium tracking-wide">
				{$t('startup.checking_system')}
			</p>
		</div>
	</div>
{:else if status === 'loading'}
	<div
		class="bg-background fixed inset-0 z-[50] flex flex-col items-center justify-center p-6 text-center select-none"
		in:fade={{ duration: 300 }}
	>
		<div
			class="bg-surface-secondary/30 w-full max-w-md rounded-xl border border-white/5 p-8 shadow-2xl backdrop-blur-sm"
		>
			<div class="relative mx-auto mb-6 h-16 w-16">
				<div
					class="border-surface-secondary absolute inset-0 rounded-full border-4 opacity-30"
				></div>
				<div
					class="border-t-primary absolute inset-0 animate-spin rounded-full border-4 border-r-transparent border-b-transparent border-l-transparent"
				></div>
			</div>

			<h2 class="text-text-main mb-2 text-xl font-bold tracking-wide transition-all duration-300">
				{$t(currentStepKey)}
			</h2>

			<p
				class="text-primary mb-6 min-h-[1.25rem] animate-pulse text-sm font-medium transition-opacity duration-500"
			>
				{$t(waitMessageKey)}
			</p>

			<p
				class="text-text-muted mb-8 h-4 overflow-hidden font-mono text-xs text-ellipsis whitespace-nowrap opacity-50"
			>
				&gt; {detailedLog}
			</p>

			<div class="mb-8 space-y-3 text-left">
				<div
					class="flex items-center gap-3 transition-colors duration-300 {stepIndex >= 1
						? 'opacity-100'
						: 'opacity-40'}"
				>
					<div
						class="flex h-5 w-5 items-center justify-center rounded-full border {stepIndex > 1
							? 'bg-primary border-primary'
							: stepIndex === 1
								? 'border-primary animate-pulse'
								: 'border-text-muted'}"
					>
						{#if stepIndex > 1}<span class="text-xs font-bold text-black">✓</span>{/if}
					</div>
					<span class="text-sm {stepIndex === 1 ? 'text-primary font-medium' : 'text-text-muted'}">
						{$t('startup.indicator.windows_core')}
					</span>
				</div>
				<div
					class="flex items-center gap-3 transition-colors duration-300 {stepIndex >= 2
						? 'opacity-100'
						: 'opacity-40'}"
				>
					<div
						class="flex h-5 w-5 items-center justify-center rounded-full border {stepIndex > 2
							? 'bg-primary border-primary'
							: stepIndex === 2
								? 'border-primary animate-pulse'
								: 'border-text-muted'}"
					>
						{#if stepIndex > 2}<span class="text-xs font-bold text-black">✓</span>{/if}
					</div>
					<span class="text-sm {stepIndex === 2 ? 'text-primary font-medium' : 'text-text-muted'}">
						{$t('startup.indicator.ubuntu_env')}
					</span>
				</div>
				<div
					class="flex items-center gap-3 transition-colors duration-300 {stepIndex >= 3
						? 'opacity-100'
						: 'opacity-40'}"
				>
					<div
						class="flex h-5 w-5 items-center justify-center rounded-full border {stepIndex > 3
							? 'bg-primary border-primary'
							: stepIndex === 3
								? 'border-primary animate-pulse'
								: 'border-text-muted'}"
					>
						{#if stepIndex > 3}<span class="text-xs font-bold text-black">✓</span>{/if}
					</div>
					<span class="text-sm {stepIndex === 3 ? 'text-primary font-medium' : 'text-text-muted'}">
						{$t('startup.indicator.network_sec')}
					</span>
				</div>
			</div>

			{#if stepIndex === 1}
				<div
					transition:slide
					class="rounded border border-yellow-500/20 bg-yellow-500/10 p-4 text-left"
				>
					<div class="flex items-start gap-3">
						<span class="text-lg text-yellow-500">⚠</span>
						<div class="text-text-muted text-xs leading-relaxed">
							<strong class="mb-1 block text-yellow-500">{$t('common.action_required')}</strong>
							{$t('startup.warning.admin_prompts')}
							<br />
							{$t('startup.warning.do_not_close')}
						</div>
					</div>
				</div>
			{/if}
		</div>
	</div>
{:else if status === 'reboot_needed'}
	<div
		class="bg-background fixed inset-0 z-[50] flex flex-col items-center justify-center p-8 text-center"
		in:fade
	>
		<div class="bg-surface-secondary/30 w-full max-w-md rounded-xl border border-white/5 p-8">
			<div class="text-status-added mb-6 flex justify-center text-5xl">
				<div
					class="bg-status-added/20 text-status-added flex h-20 w-20 items-center justify-center rounded-full"
				>
					✔
				</div>
			</div>
			<h2 class="text-text-main mb-4 text-2xl font-bold">
				{$t('startup.reboot.title')}
			</h2>
			<p class="text-text-muted mb-8">
				{$t('startup.reboot.desc')}
				<br /><br />
				<span class="text-primary font-medium">{$t('startup.reboot.action')}</span>
			</p>
			<button
				class="btn-action danger w-full justify-center py-4 text-lg font-bold"
				on:click={restartPC}
			>
				{$t('common.restart_now')}
			</button>
		</div>
	</div>
{:else if status === 'error'}
	<div
		class="bg-background fixed inset-0 z-[50] flex flex-col items-center justify-center p-8 text-center"
		in:fade
	>
		<div
			class="bg-surface-secondary/30 border-status-error/20 w-full max-w-md rounded-xl border p-8"
		>
			<div class="text-status-error mb-6 flex justify-center text-5xl">⚠</div>
			<h2 class="text-text-main mb-2 text-2xl font-bold">{$t('startup.error.incomplete_title')}</h2>
			<p class="text-text-muted mb-6 text-sm">{$t('startup.error.generic_desc')}</p>
			<div
				class="mb-8 max-h-32 overflow-y-auto rounded border border-white/5 bg-black/40 p-4 text-left"
			>
				<p class="text-status-error font-mono text-xs break-all">{detailedLog}</p>
			</div>
			<div class="flex gap-4">
				<button
					class="btn-action secondary flex-1 justify-center"
					on:click={() => window.location.reload()}
				>
					{$t('common.try_again')}
				</button>
			</div>
		</div>
	</div>
{:else}
	<div in:fly={{ y: 20, duration: 600 }}>
		<slot />
	</div>
{/if}

<div class="relative z-[9999]">
	<LanguageSwitcher />
	<ThemeToggle />
</div>
