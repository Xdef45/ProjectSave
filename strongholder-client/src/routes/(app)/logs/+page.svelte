<script lang="ts">
	import { onMount } from 'svelte';
	import { t, locale } from '$lib/i18n';
	import { slide, fade } from 'svelte/transition';
	import { getBackupLogs, type LogEntry } from '$lib/services/logs';

	// --- CONFIGURATION DES STATUTS ---
	// Mappe les codes de statut renvoyés par Borg Backup vers des styles et des traductions
	const STATUS_CONFIG: Record<string, { labelKey: string; className: string; bgClass: string }> = {
		A: {
			labelKey: 'logs.filter.added',
			className: 'text-status-added',
			bgClass: 'bg-green-50 dark:bg-green-900/20'
		},
		M: {
			labelKey: 'logs.filter.modified',
			className: 'text-status-modified',
			bgClass: 'bg-yellow-50 dark:bg-yellow-900/20'
		},
		D: {
			labelKey: 'logs.filter.deleted',
			className: 'text-status-deleted',
			bgClass: 'bg-red-50 dark:bg-red-900/20'
		},
		d: {
			labelKey: 'logs.filter.deleted',
			className: 'text-status-deleted',
			bgClass: 'bg-red-50 dark:bg-red-900/20'
		},
		E: {
			labelKey: 'logs.filter.error',
			className: 'text-status-error',
			bgClass: 'bg-red-50 dark:bg-red-900/20'
		},
		U: {
			labelKey: 'logs.filter.unchanged',
			className: 'text-text-muted',
			bgClass: 'bg-bg-subtle'
		}
	};

	// --- ÉTAT ---
	let logs: LogEntry[] = [];
	let loading = true;
	let globalError: string | null = null;

	// --- FORMATEURS RÉACTIFS ---
	$: dateFormat = new Intl.DateTimeFormat($locale, {
		day: '2-digit',
		month: 'short',
		year: 'numeric'
	});
	$: timeFormat = new Intl.DateTimeFormat($locale, { hour: '2-digit', minute: '2-digit' });

	// --- FONCTIONS UTILITAIRES ---

	// Formate la taille des données en unités lisibles
	function formatSize(bytes: number): string {
		const safeBytes = Math.max(0, Number(bytes) || 0);
		if (safeBytes === 0) return '0 B';

		const k = 1024;
		const sizes = ['B', 'KB', 'MB', 'GB', 'TB'];
		const i = Math.floor(Math.log(safeBytes) / Math.log(k));
		const safeIndex = Math.min(i, sizes.length - 1);

		return `${parseFloat((safeBytes / Math.pow(k, safeIndex)).toFixed(2))} ${sizes[safeIndex]}`;
	}

	// Formate la durée de l'opération de sauvegarde
	function formatDuration(seconds: number): string {
		const safeSeconds = Math.max(0, Number(seconds) || 0);
		if (safeSeconds < 1) return '< 1s';

		const h = Math.floor(safeSeconds / 3600);
		const m = Math.floor((safeSeconds % 3600) / 60);
		const s = Math.floor(safeSeconds % 60);

		return h > 0 ? `${h}h ${m}m ${s}s` : m > 0 ? `${m}m ${s}s` : `${safeSeconds.toFixed(1)}s`;
	}

	// Alterne l'affichage des détails d'un journal spécifique
	function toggleLog(index: number) {
		// Mise à jour réactive ciblée : seul l'élément modifié est mis à jour dans le tableau
		logs[index] = { ...logs[index], isOpen: !logs[index].isOpen };
	}

	// Logique de filtrage des fichiers à l'intérieur d'un journal
	function getVisibleFiles(log: LogEntry) {
		if (!log.files) return [];
		const term = (log.filterTerm || '').toLowerCase();
		const state = log.filterState || 'all';

		// Retour rapide si aucun filtre n'est appliqué (limité à 500 pour les performances)
		if (!term && state === 'all') return log.files.slice(0, 500);

		return log.files
			.filter((f) => {
				const matchesSearch = !term || f.path.toLowerCase().includes(term);
				const matchesState = state === 'all' || f.status === state;
				return matchesSearch && matchesState;
			})
			.slice(0, 500); // Limite de sécurité pour éviter de geler l'application avec trop de nœuds DOM
	}

	// Chargement initial
	onMount(async () => {
		try {
			logs = await getBackupLogs();
		} catch (e: unknown) {
			globalError = e instanceof Error ? e.message : String(e);
		} finally {
			loading = false;
		}
	});
</script>

<div class="dashboard-wrapper">
	<div class="mb-6 flex items-end justify-between">
		<h2 class="section-title mb-0">{$t('logs.title')}</h2>
		<!-- <button class="btn-action secondary h-9 text-xs">
			{$t('logs.export')}
		</button> -->
	</div>

	<div
		class="text-text-muted grid grid-cols-[2fr_1fr_1fr_1fr_30px] px-5 py-2 text-[10px] font-bold uppercase select-none"
	>
		<div>{$t('logs.headers.date')}</div>
		<div>{$t('logs.headers.status')}</div>
		<div>{$t('logs.headers.files')}</div>
		<div>{$t('logs.headers.size')}</div>
		<div></div>
	</div>

	<div class="space-y-3">
		{#if loading}
			<div class="animate-pulse space-y-3">
				<div class="h-16 w-full rounded-lg bg-gray-200 dark:bg-gray-800"></div>
				<div class="h-16 w-full rounded-lg bg-gray-200 dark:bg-gray-800"></div>
			</div>
		{:else if logs.length === 0}
			<div class="text-text-muted flex flex-col items-center justify-center p-10">
				<span class="mb-2 text-4xl opacity-20">📋</span>
				<span class="text-sm">{$t('logs.empty')}</span>
			</div>
		{:else}
			{#each logs as log, i (log.id || i)}
				<div
					class="bg-surface overflow-hidden rounded-lg border transition-all {log.isOpen
						? 'border-primary ring-primary/20 shadow-sm ring-1'
						: 'border-border hover:border-primary/50'}"
				>
					<button
						class="hover:bg-bg-subtle grid w-full cursor-pointer grid-cols-[2fr_1fr_1fr_1fr_30px] items-center p-4 text-left transition-colors outline-none"
						on:click={() => toggleLog(i)}
						aria-expanded={log.isOpen}
					>
						<div class="text-text-main text-sm font-semibold tabular-nums">
							{dateFormat.format(new Date(log.date))} • {timeFormat.format(new Date(log.date))}
						</div>

						<div class="flex items-center gap-2 text-xs font-bold uppercase">
							<span
								class="h-2 w-2 rounded-full {log.status === 'Success'
									? 'bg-status-added shadow-[0_0_0_2px_rgba(0,184,148,0.2)]'
									: 'bg-status-deleted shadow-[0_0_0_2px_rgba(255,118,117,0.2)]'}"
							></span>
							<span class={log.status === 'Success' ? 'text-status-added' : 'text-status-deleted'}>
								{log.status === 'Success' ? $t('logs.status.success') : $t('logs.status.error')}
							</span>
						</div>

						<div class="text-text-main text-sm tabular-nums">{log.total_files}</div>
						<div class="text-text-muted font-mono text-xs">{formatSize(log.total_size)}</div>

						<div
							class="text-text-muted transition-transform duration-300 {log.isOpen
								? 'text-primary rotate-180'
								: ''}"
						>
							<svg
								width="10"
								height="6"
								viewBox="0 0 10 6"
								fill="none"
								stroke="currentColor"
								stroke-width="2"><path d="M1 1L5 5L9 1" /></svg
							>
						</div>
					</button>

					{#if log.isOpen}
						<div
							class="border-border bg-bg-subtle/30 border-t p-6"
							transition:slide={{ duration: 200 }}
						>
							<div class="mb-6 grid grid-cols-2 gap-4 md:grid-cols-4">
								<div
									class="bg-surface rounded-lg border border-green-200 p-3 text-center dark:border-green-800"
								>
									<div class="text-status-added text-xl font-bold">+{log.count_added}</div>
									<div class="text-text-muted text-[10px] font-bold uppercase">
										{$t('logs.details.added')}
									</div>
								</div>
								<div
									class="bg-surface rounded-lg border border-yellow-200 p-3 text-center dark:border-yellow-800"
								>
									<div class="text-status-modified text-xl font-bold">{log.count_modified}</div>
									<div class="text-text-muted text-[10px] font-bold uppercase">
										{$t('logs.details.modified')}
									</div>
								</div>
								<div
									class="bg-surface rounded-lg border border-red-200 p-3 text-center dark:border-red-800"
								>
									<div class="text-status-deleted text-xl font-bold">-{log.count_deleted}</div>
									<div class="text-text-muted text-[10px] font-bold uppercase">
										{$t('logs.details.deleted')}
									</div>
								</div>
								<div class="border-border bg-surface rounded-lg border p-3 text-center">
									<div class="text-text-main text-xl font-bold">{formatDuration(log.duration)}</div>
									<div class="text-text-muted text-[10px] font-bold uppercase">
										{$t('logs.details.duration')}
									</div>
								</div>
							</div>

							<div class="mb-3 flex gap-3">
								<div class="relative flex-1">
									<span class="absolute top-1/2 left-3 -translate-y-1/2 opacity-40">🔍</span>
									<input
										type="text"
										placeholder={$t('logs.filter.placeholder')}
										bind:value={log.filterTerm}
										on:input={() => (logs = logs)}
										class="border-border text-text-main focus:border-primary focus:ring-primary/20 bg-surface w-full rounded-lg border py-2 pr-3 pl-9 text-sm outline-none focus:ring-1"
									/>
								</div>
								<select
									bind:value={log.filterState}
									on:change={() => (logs = logs)}
									class="border-border text-text-main focus:border-primary focus:ring-primary/20 bg-surface rounded-lg border px-4 py-2 text-sm outline-none focus:ring-1"
								>
									<option value="all">{$t('logs.filter.all')}</option>
									{#each Object.entries(STATUS_CONFIG) as [code, cfg]}
										{#if code !== 'd'}
											<option value={code}>{$t(cfg.labelKey)}</option>
										{/if}
									{/each}
								</select>
							</div>

							<div
								class="border-border bg-surface max-h-[300px] overflow-y-auto rounded-lg border shadow-sm"
							>
								{#each getVisibleFiles(log) as file}
									{@const cfg = STATUS_CONFIG[file.status] || STATUS_CONFIG['U']}
									<div
										class="hover:bg-bg-subtle border-border flex items-center justify-between border-b px-4 py-2 text-sm transition-colors last:border-0"
										in:fade
									>
										<span class="text-text-main truncate pr-4 font-mono text-xs" title={file.path}
											>{file.path}</span
										>
										<span
											class="shrink-0 rounded px-2 py-0.5 text-[10px] font-bold uppercase {cfg.bgClass} {cfg.className}"
										>
											{$t(cfg.labelKey)}
										</span>
									</div>
								{/each}

								{#if getVisibleFiles(log).length === 0}
									<div class="text-text-muted p-8 text-center text-xs italic">
										{$t('logs.no_match')}
									</div>
								{/if}
							</div>

							{#if log.files.length > 500}
								<p class="text-text-muted mt-2 text-[10px] italic">
									* Showing first 500 items. Use search to filter results.
								</p>
							{/if}
						</div>
					{/if}
				</div>
			{/each}
		{/if}
	</div>
</div>
