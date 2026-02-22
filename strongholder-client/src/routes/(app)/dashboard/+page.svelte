<script lang="ts">
	import { onMount } from 'svelte';
	import { t, locale } from '$lib/i18n';

	// Importation depuis la couche de service stricte
	import { getDashboardStats, type DashboardLogEntry } from '$lib/services/logs';

	// --- État Local ---
	let lastLog: DashboardLogEntry | null = null;
	let isLoading = true;

	// --- Formateurs réactifs (S'adaptent à la langue de l'utilisateur) ---
	$: dateFormat = new Intl.DateTimeFormat($locale, {
		day: '2-digit',
		month: 'long',
		year: 'numeric'
	});

	$: timeFormat = new Intl.DateTimeFormat($locale, {
		hour: '2-digit',
		minute: '2-digit'
	});

	$: numberFormat = new Intl.NumberFormat($locale);

	// --- Fonctions Utilitaires ---

	function capitalize(str: string): string {
		if (!str) return '';
		return str.charAt(0).toUpperCase() + str.slice(1);
	}

	// Convertit une taille en octets vers une unité lisible (Mo, Go, etc.)
	function formatBytes(bytes: number): string {
		const safeBytes = Math.max(0, Number(bytes) || 0);
		if (safeBytes === 0) return '0 B';

		const k = 1024;
		const sizes = ['B', 'KB', 'MB', 'GB', 'TB'];
		const i = Math.floor(Math.log(safeBytes) / Math.log(k));
		const safeIndex = Math.min(i, sizes.length - 1);

		const formattedNumber = new Intl.NumberFormat($locale, {
			maximumFractionDigits: 1
		}).format(safeBytes / Math.pow(k, safeIndex));

		return `${formattedNumber} ${sizes[safeIndex]}`;
	}

	// Sécurité pour la conversion des dates
	function getSafeDate(dateString: string): Date {
		const d = new Date(dateString);
		return isNaN(d.getTime()) ? new Date() : d;
	}

	// Chargement initial des données depuis le backend Rust
	async function loadStats() {
		isLoading = true;
		try {
			lastLog = await getDashboardStats();
		} catch (e) {
			// L'erreur est déjà traitée par le service.
			// On bascule simplement l'interface sur l'état "Aucune sauvegarde".
			lastLog = null;
		} finally {
			isLoading = false;
		}
	}

	onMount(() => {
		loadStats();
	});
</script>

<div class="dashboard-wrapper">
	<section class="mb-8 flex gap-4">
		<a href="/save" class="btn-action primary flex-1 justify-center">
			<span class="mr-2 text-lg leading-none" aria-hidden="true">+</span>
			{$t('dashboard.action_save')}
		</a>
		<a href="/restore" class="btn-action secondary px-8">
			<svg
				width="18"
				height="18"
				viewBox="0 0 24 24"
				fill="none"
				stroke="currentColor"
				stroke-width="2"
				class="mr-2"
				aria-hidden="true"
			>
				<path d="M2.5 2v6h6M2.66 15.57a10 10 0 1 0 .57-8.38" />
			</svg>
			{$t('dashboard.action_restore')}
		</a>
	</section>

	{#if isLoading}
		<div class="stats-grid animate-pulse">
			<div class="card h-32 bg-gray-100 dark:bg-gray-800"></div>
			<div class="card h-32 bg-gray-100 dark:bg-gray-800"></div>
		</div>
	{:else if !lastLog}
		<div class="stats-grid">
			<div class="card flex flex-col justify-center">
				<h4 class="text-text-muted mb-4 text-xs font-bold tracking-wider uppercase">
					{$t('dashboard.last_backup')}
				</h4>
				<div class="text-primary mb-1 text-3xl font-bold">{$t('dashboard.not_available')}</div>
				<div class="text-text-muted mt-2 text-sm">
					0 {$t('dashboard.files_secured')}
				</div>
			</div>

			<div class="card flex flex-col justify-center">
				<h4 class="text-text-muted mb-4 text-xs font-bold tracking-wider uppercase">
					{$t('dashboard.data_protected')}
				</h4>
				<div class="text-text-main mb-1 text-3xl font-bold">0 B</div>
			</div>
		</div>
	{:else}
		<div class="stats-grid">
			<div class="card relative overflow-hidden">
				<div
					class="absolute top-0 bottom-0 left-0 w-1 {lastLog.status.toLowerCase() === 'error'
						? 'bg-status-error'
						: 'bg-status-added'}"
				></div>

				<div class="mb-4 flex items-start justify-between">
					<h4 class="text-text-muted pl-2 text-xs font-bold tracking-wider uppercase">
						{$t('dashboard.last_backup')}
					</h4>

					{#if lastLog.status.toLowerCase() === 'error'}
						<span
							class="bg-status-error/10 text-status-error rounded px-2 py-0.5 text-[10px] font-bold uppercase"
						>
							{$t('dashboard.status_error')}
						</span>
					{:else}
						<span
							class="bg-status-added/10 text-status-added rounded px-2 py-0.5 text-[10px] font-bold uppercase"
						>
							{$t('dashboard.status_success')}
						</span>
					{/if}
				</div>

				<div class="text-primary mb-1 pl-2 text-3xl font-bold">
					{capitalize(dateFormat.format(new Date(lastLog.date)))}
					<span class="text-text-muted ml-2 text-base font-normal">
						{timeFormat.format(new Date(lastLog.date))}
					</span>
				</div>
				<div class="text-text-muted mt-2 pl-2 text-sm">
					<span class="text-text-main font-medium">{numberFormat.format(lastLog.total_files)}</span>
					{$t('dashboard.files_secured')}
				</div>
			</div>

			<div class="card flex flex-col justify-between">
				<h4 class="text-text-muted mb-4 text-xs font-bold tracking-wider uppercase">
					{$t('dashboard.data_protected')}
				</h4>
				<div class="text-text-main mb-1 text-4xl font-bold">
					{formatBytes(lastLog.total_size)}
				</div>
				<div class="text-text-muted mt-2 text-sm">
					{$t('dashboard.storage_used_desc')}
				</div>
			</div>
		</div>

		<h3 class="section-title text-text-muted mt-8 mb-4 text-sm font-bold tracking-wider uppercase">
			{$t('dashboard.stats_title')}
		</h3>

		<div class="details-grid grid grid-cols-3 gap-4">
			<div class="card detail-card flex flex-col items-center justify-center p-6 text-center">
				<div class="text-status-modified mb-2 text-4xl font-bold">
					{numberFormat.format(lastLog.count_modified)}
				</div>
				<div class="text-text-muted text-sm font-medium">{$t('dashboard.modified')}</div>
			</div>

			<div class="card detail-card flex flex-col items-center justify-center p-6 text-center">
				<div class="text-status-added mb-2 text-4xl font-bold">
					+{numberFormat.format(lastLog.count_added)}
				</div>
				<div class="text-text-muted text-sm font-medium">{$t('dashboard.added')}</div>
			</div>

			<div class="card detail-card flex flex-col items-center justify-center p-6 text-center">
				<div class="text-status-deleted mb-2 text-4xl font-bold">
					-{numberFormat.format(lastLog.count_deleted)}
				</div>
				<div class="text-text-muted text-sm font-medium">{$t('dashboard.deleted')}</div>
			</div>
		</div>
	{/if}
</div>
