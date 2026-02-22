<script lang="ts">
	import { onMount, onDestroy } from 'svelte';
	import { t } from '$lib/i18n';
	import { fade, slide } from 'svelte/transition';

	// Importation de la couche de service stricte pour la restauration
	import {
		fetchArchivesList,
		fetchArchiveFiles,
		askSavePath,
		downloadAndSaveArchive,
		restoreArchiveInPlace,
		cancelActiveRestore,
		getSystemDrives,
		type ArchiveFileRaw
	} from '$lib/services/restore';

	import { loadAppConfig, type AppConfig } from '$lib/services/config';

	// Importation des stores de surveillance (réseau et batterie)
	import { isOnline } from '$lib/stores/connection';
	import { isLowBattery, startPowerMonitoring } from '$lib/stores/power';

	// --- Types de données ---
	interface Backup {
		id: string;
		date: string;
		label: string;
	}

	interface ExplorerItem {
		name: string;
		type: 'folder' | 'file' | 'image' | 'sheet';
		fullPath: string;
	}

	interface Notification {
		message: string;
		type: 'success' | 'error' | 'info';
	}

	// --- État de l'interface ---
	let backups: Backup[] = [];
	let allRawFiles: ArchiveFileRaw[] = [];

	let isLoadingBackups = false;
	let isLoadingFiles = false;
	let fetchError = '';

	let isCancelled = false;

	let selectedBackupId: string | null = null;
	let currentPath = '';

	let selectedFiles = new Set<string>();
	let lastSelectedPath: string | null = null;

	let config = { general: { battery_limit: true } };

	let availableRoots: string[] = [];

	let showConfirmModal = false;
	let pendingRestoreMode: 'save' | 'inplace' | null = null;
	let globalNotification: Notification | null = null;
	let notificationTimer: ReturnType<typeof setTimeout> | null = null;

	// --- Propriétés réactives (Computed) ---
	$: breadcrumbs = currentPath ? currentPath.split('/').filter(Boolean) : [];
	$: visibleItems = computeVisibleItems(allRawFiles, currentPath);
	$: isAppLocked = !$isOnline || ($isLowBattery && config.general.battery_limit);
	$: selectionSummary = getSelectionSummary(selectedFiles);

	$: canRestoreInPlace = checkCanRestoreInPlace(
		selectedBackupId,
		selectedFiles,
		allRawFiles,
		availableRoots
	);

	// --- MOTEUR DE L'EXPLORATEUR ---

	/**
	 * Transforme une liste plate de chemins de fichiers en une structure de dossier
	 * pour le niveau actuel de navigation (currentPath).
	 */
	function computeVisibleItems(allFiles: ArchiveFileRaw[], path: string): ExplorerItem[] {
		if (!allFiles.length) return [];
		const uniqueItems = new Map<string, ExplorerItem>();
		const prefix = path;

		for (const file of allFiles) {
			if (!file.path.startsWith(prefix)) continue;
			const relative = file.path.slice(prefix.length);
			if (!relative) continue;

			const parts = relative.split('/');
			const name = parts[0];
			const isFolder = parts.length > 1 || file.type === 'd';

			if (!uniqueItems.has(name)) {
				uniqueItems.set(name, {
					name: name,
					type: isFolder ? 'folder' : getUiType(name),
					fullPath: prefix + name + (isFolder ? '/' : '')
				});
			}
		}

		// Tri : Dossiers en premier, puis fichiers par ordre alphabétique
		return Array.from(uniqueItems.values()).sort((a, b) => {
			if (a.type === 'folder' && b.type !== 'folder') return -1;
			if (a.type !== 'folder' && b.type === 'folder') return 1;
			return a.name.localeCompare(b.name);
		});
	}

	function getUiType(filename: string) {
		const lower = filename.toLowerCase();
		if (/\.(png|jpg|jpeg|gif|webp|svg)$/.test(lower)) return 'image';
		if (/\.(xlsx|xls|csv|ods)$/.test(lower)) return 'sheet';
		return 'file';
	}

	function formatBackupLabel(isoString: string): string {
		try {
			return new Intl.DateTimeFormat(navigator.language, {
				day: '2-digit',
				month: 'short',
				year: 'numeric',
				hour: '2-digit',
				minute: '2-digit'
			}).format(new Date(isoString));
		} catch (e) {
			return isoString;
		}
	}

	function getSelectionSummary(selected: Set<string>) {
		if (!selectedBackupId) return $t('restore.target_none');
		if (selected.size === 0) return $t('restore.target_all');
		if (selected.size === 1) {
			const firstItem = [...selected][0];
			return firstItem.split('/').pop() || firstItem;
		}
		return $t('restore.summary_files', { count: selected.size });
	}

	/**
	 * Vérifie si les chemins de l'archive correspondent à des lecteurs réellement
	 * connectés sur la machine actuelle pour autoriser la restauration "sur place".
	 */
	function checkCanRestoreInPlace(
		backupId: string | null,
		selected: Set<string>,
		allFiles: ArchiveFileRaw[],
		roots: string[]
	): boolean {
		if (!backupId || roots.length === 0 || allFiles.length === 0) return false;
		const pathsToCheck = selected.size > 0 ? Array.from(selected) : allFiles.map((f) => f.path);

		return pathsToCheck.every((path) => {
			if (/^[A-Z]:\//i.test(path)) {
				return roots.some((root) => path.toUpperCase().startsWith(root.toUpperCase()));
			}
			return roots.includes('/');
		});
	}

	function showNotification(msg: string, type: 'success' | 'error' | 'info' = 'info') {
		if (notificationTimer) clearTimeout(notificationTimer);
		globalNotification = { message: msg, type };
		notificationTimer = setTimeout(() => {
			globalNotification = null;
		}, 5000);
	}

	// --- ACTIONS UTILISATEUR ---

	async function selectBackup(id: string) {
		selectedBackupId = id;
		currentPath = '';
		selectedFiles = new Set();
		isLoadingFiles = true;
		allRawFiles = [];

		try {
			const raw = await fetchArchiveFiles(id);
			// Conversion des chemins Linux (/mnt/c/) en chemins Windows (C:/) pour l'affichage
			allRawFiles = raw.map((f) => ({
				...f,
				path: f.path.replace(/^(\/)?mnt\/([a-z])\//i, (match, slash, drive) => {
					return `${drive.toUpperCase()}:/`;
				})
			}));
		} catch (e) {
			showNotification('Échec du chargement de la liste des fichiers', 'error');
		} finally {
			isLoadingFiles = false;
		}
	}

	function handleItemClick(item: ExplorerItem, event: MouseEvent) {
		if (item.type === 'folder') {
			if (isLoadingFiles) return;
			currentPath = item.fullPath;
			return;
		}
		handleFileSelection(item, event);
	}

	/**
	 * Gère la sélection complexe (multi-sélection avec Ctrl et Shift)
	 */
	function handleFileSelection(item: ExplorerItem, event: MouseEvent) {
		event.stopPropagation();
		const id = item.fullPath;
		const nextSelection = new Set(selectedFiles);

		if (event.ctrlKey || event.metaKey) {
			if (nextSelection.has(id)) nextSelection.delete(id);
			else nextSelection.add(id);
			lastSelectedPath = id;
		} else if (event.shiftKey && lastSelectedPath) {
			const currentIndex = visibleItems.findIndex((i) => i.fullPath === id);
			const lastIndex = visibleItems.findIndex((i) => i.fullPath === lastSelectedPath);

			if (currentIndex !== -1 && lastIndex !== -1) {
				const start = Math.min(currentIndex, lastIndex);
				const end = Math.max(currentIndex, lastIndex);
				nextSelection.clear();

				for (let i = start; i <= end; i++) {
					const it = visibleItems[i];
					if (it.type !== 'folder') nextSelection.add(it.fullPath);
				}
			}
		} else {
			nextSelection.clear();
			nextSelection.add(id);
			lastSelectedPath = id;
		}

		selectedFiles = nextSelection;
	}

	function navigateUp() {
		if (!currentPath) return;
		const parts = currentPath.split('/').filter(Boolean);
		parts.pop();
		currentPath = parts.length > 0 ? parts.join('/') + '/' : '';
	}

	function navigateToBreadcrumb(index: number) {
		const parts = currentPath.split('/').filter(Boolean);
		const newParts = parts.slice(0, index + 1);
		currentPath = newParts.join('/') + '/';
	}

	function handleBackgroundClick(e: MouseEvent) {
		if (e.target === e.currentTarget) {
			selectedFiles = new Set();
		}
	}

	// --- CYCLE DE VIE ---

	onMount(async () => {
		startPowerMonitoring();
		isLoadingBackups = true;
		try {
			const rawDrives = await getSystemDrives();
			availableRoots = rawDrives.map((d) => d.replace(/\\/g, '/'));

			const loadedConfig = await loadAppConfig().catch(() => null);
			if (loadedConfig) config = loadedConfig;

			const archives = await fetchArchivesList();
			backups = archives
				.map((a) => ({ id: a.archive, date: a.time, label: formatBackupLabel(a.time) }))
				.sort((a, b) => new Date(b.date).getTime() - new Date(a.date).getTime());
		} catch (e) {
			fetchError = 'Impossible de charger les sauvegardes';
		} finally {
			isLoadingBackups = false;
		}
	});

	function getIcon(type: string) {
		if (type === 'folder') return '📁';
		if (type === 'image') return '🖼️';
		if (type === 'sheet') return '📊';
		return '📄';
	}

	// --- LOGIQUE DE RESTAURATION ---

	function initiateRestore(mode: 'save' | 'inplace') {
		if (isAppLocked || !selectedBackupId) return;
		pendingRestoreMode = mode;

		if (mode === 'inplace') {
			showConfirmModal = true;
		} else {
			executeRestore();
		}
	}

	async function cancelRestore() {
		isCancelled = true;
		isLoadingFiles = false;
		pendingRestoreMode = null;

		await cancelActiveRestore();
		showNotification($t('common.cancel') + '...', 'info');
	}

	async function executeRestore() {
		showConfirmModal = false;
		if (!selectedBackupId || !pendingRestoreMode) return;

		isCancelled = false;

		try {
			if (pendingRestoreMode === 'save') {
				const targetPath = await askSavePath(selectedBackupId);

				if (!targetPath || isCancelled) {
					pendingRestoreMode = null;
					return;
				}

				isLoadingFiles = true;
				const savedPath = await downloadAndSaveArchive(selectedBackupId, targetPath);

				if (isCancelled) return;
				showNotification(`Enregistré sous : ${savedPath}`, 'success');
			} else {
				isLoadingFiles = true;
				const resultMsg = await restoreArchiveInPlace(selectedBackupId);

				if (isCancelled) return;
				showNotification(resultMsg, 'success');
			}
		} catch (err: unknown) {
			if (isCancelled) return;

			let safeErrorMsg = $t('errors.unknown') || 'Erreur inconnue';
			if (typeof err === 'string') safeErrorMsg = err;
			else if (err instanceof Error) safeErrorMsg = err.message;

			showNotification(`${$t('restore.error_msg')}: ${safeErrorMsg}`, 'error');
		} finally {
			if (!isCancelled) {
				isLoadingFiles = false;
				pendingRestoreMode = null;
			}
		}
	}
</script>

{#if globalNotification}
	<div
		transition:slide
		class="absolute top-6 left-1/2 z-[60] flex -translate-x-1/2 items-center gap-4 rounded-lg border px-6 py-4 text-sm font-bold shadow-xl backdrop-blur-md"
		class:bg-red-50={globalNotification.type === 'error'}
		class:border-red-200={globalNotification.type === 'error'}
		class:text-red-700={globalNotification.type === 'error'}
		class:bg-green-50={globalNotification.type === 'success'}
		class:border-green-200={globalNotification.type === 'success'}
		class:text-green-700={globalNotification.type === 'success'}
		class:bg-blue-50={globalNotification.type === 'info'}
		class:border-blue-200={globalNotification.type === 'info'}
		class:text-blue-700={globalNotification.type === 'info'}
		role="alert"
	>
		<span>
			{#if globalNotification.type === 'error'}⚠️{/if}
			{#if globalNotification.type === 'success'}✅{/if}
			{#if globalNotification.type === 'info'}ℹ️{/if}
			{globalNotification.message}
		</span>
		<button
			on:click={() => (globalNotification = null)}
			class="opacity-50 hover:opacity-100"
			aria-label={$t('common.close')}>✕</button
		>
	</div>
{/if}

{#if isLoadingFiles && pendingRestoreMode}
	<div
		class="fixed inset-0 z-[70] flex flex-col items-center justify-center bg-black/50 backdrop-blur-sm"
		transition:fade
	>
		<div
			class="bg-bg text-text-main flex w-full max-w-md flex-col items-center gap-6 rounded-xl p-8 shadow-2xl"
		>
			<h3 class="animate-pulse text-xl font-bold">
				{pendingRestoreMode === 'save' ? 'Downloading & Saving...' : 'Restoring Files...'}
			</h3>

			<div class="h-3 w-full overflow-hidden rounded-full bg-gray-200 dark:bg-gray-700">
				<div
					class="bg-primary h-full w-full origin-left animate-[progress_1.5s_ease-in-out_infinite] rounded-full"
				></div>
			</div>

			<button class="btn-action secondary w-full justify-center" on:click={cancelRestore}>
				{$t('common.cancel')}
			</button>
		</div>
	</div>

	<style>
		@keyframes progress {
			0% {
				transform: translateX(-100%);
			}
			100% {
				transform: translateX(100%);
			}
		}
	</style>
{/if}

{#if showConfirmModal}
	<div
		class="fixed inset-0 z-[60] flex items-center justify-center bg-black/60 backdrop-blur-sm"
		transition:fade={{ duration: 100 }}
	>
		<div
			class="bg-bg text-text-main w-full max-w-md rounded-xl p-6 shadow-2xl"
			role="dialog"
			aria-modal="true"
		>
			<div class="mb-4 flex items-center gap-3 text-amber-500">
				<span class="text-3xl">⚠️</span>
				<h3 class="text-lg font-bold">
					{$t('restore.warn_overwrite_title') || 'Overwrite Warning'}
				</h3>
			</div>

			<p class="text-text-muted mb-8 text-sm leading-relaxed">
				{$t('restore.warn_overwrite_body') ||
					'This action will restore files to their original locations and overwrite any existing files with the same name. Are you sure you want to proceed?'}
			</p>

			<div class="flex justify-end gap-3">
				<button class="btn-action secondary px-4" on:click={() => (showConfirmModal = false)}>
					{$t('common.cancel')}
				</button>
				<button class="btn-action danger px-4" on:click={executeRestore}>
					{$t('restore.confirm_overwrite') || 'Yes, Overwrite'}
				</button>
			</div>
		</div>
	</div>
{/if}

<div class="dashboard-wrapper">
	<h2 class="section-title">{$t('restore.title')}</h2>

	<div class="grid h-[calc(100vh-25vh)] grid-cols-1 gap-6 md:grid-cols-[1fr_2fr]">
		<div class="card flex flex-col overflow-hidden p-0">
			<div
				class="border-border bg-bg text-text-main flex justify-between border-b p-3 text-sm font-bold"
			>
				{$t('restore.backups_list')}
				{#if isLoadingBackups}
					<span
						class="border-primary h-4 w-4 animate-spin rounded-full border-2 border-t-transparent"
					></span>
				{/if}
			</div>
			<div class="flex-1 overflow-y-auto p-2">
				{#if backups.length === 0 && !isLoadingBackups}
					<div class="text-text-muted p-4 text-center text-sm italic">
						{$t('restore.no_backups')}
					</div>
				{:else}
					{#each backups as backup (backup.id)}
						<button
							class="tree-item w-full text-left {selectedBackupId === backup.id ? 'selected' : ''}"
							on:click={() => selectBackup(backup.id)}
						>
							<span class="text-lg">📦</span>
							<div class="flex flex-col truncate">
								<span class="font-medium">{backup.label}</span>
								<span class="text-text-muted text-[10px]">{backup.id}</span>
							</div>
						</button>
					{/each}
				{/if}
			</div>
		</div>

		<div class="card flex flex-col overflow-hidden p-0">
			<div
				class="border-border bg-bg text-text-main flex items-center justify-between border-b p-3 text-sm font-bold"
			>
				<span>{$t('restore.files_in_backup')}</span>
				{#if isLoadingFiles && !pendingRestoreMode}
					<span
						class="border-primary h-4 w-4 animate-spin rounded-full border-2 border-t-transparent"
					></span>
				{/if}
			</div>

			{#if selectedBackupId}
				<div
					class="border-border flex items-center gap-1 overflow-x-auto border-b bg-gray-50 p-2 text-xs whitespace-nowrap dark:bg-black/20"
				>
					<button
						class="rounded px-2 py-1 transition-colors hover:bg-gray-200 dark:hover:bg-white/10 {currentPath ===
						''
							? 'text-primary font-bold'
							: 'text-text-muted'}"
						on:click={() => (currentPath = '')}
					>
						🏠 {$t('restore.path_root') || 'Root'}
					</button>
					{#each breadcrumbs as part, i}
						<span class="text-text-muted">/</span>
						<button
							class="hover:text-primary rounded px-2 py-1 font-medium transition-colors hover:bg-gray-200 dark:hover:bg-white/10"
							on:click={() => navigateToBreadcrumb(i)}
						>
							{part}
						</button>
					{/each}
				</div>
			{/if}

			<div
				class="relative flex-1 cursor-default overflow-y-auto p-2 outline-none"
				on:click={handleBackgroundClick}
			>
				{#if !selectedBackupId}
					<div
						class="text-text-muted flex h-full flex-col items-center justify-center p-8 text-center text-sm italic"
					>
						<span class="mb-2 text-4xl opacity-30">⬅️</span>
						<p>{$t('restore.select_prompt')}</p>
					</div>
				{:else if isLoadingFiles && !pendingRestoreMode}
					<div class="space-y-2 p-4 opacity-50">
						<div class="h-6 w-2/3 rounded bg-gray-200/20"></div>
						<div class="h-6 w-1/2 rounded bg-gray-200/20"></div>
					</div>
				{:else}
					{#if currentPath !== ''}
						<button class="tree-item text-text-muted mb-1 w-full text-left" on:click={navigateUp}>
							<span class="text-lg">⤴️</span> <span>..</span>
						</button>
					{/if}
					{#each visibleItems as item (item.name)}
						<button
							class="tree-item w-full text-left {selectedFiles.has(item.fullPath)
								? 'selected'
								: ''}"
							on:click={(e) => handleItemClick(item, e)}
						>
							<span class="text-lg">{getIcon(item.type)}</span>
							<span class="truncate select-none">{item.name}</span>
						</button>
					{/each}
					{#if visibleItems.length === 0}
						<div class="text-text-muted p-8 text-center text-sm italic">
							{$t('restore.empty_folder')}
						</div>
					{/if}
				{/if}
			</div>

			<div class="border-border bg-bg border-t p-4">
				<div class="text-text-main mb-3 flex items-center justify-between text-sm">
					<strong>{$t('restore.target_label')} :</strong>
					{#if selectedFiles.size === 0}
						<span class="text-primary bg-primary/10 rounded px-2 py-1 text-xs font-bold uppercase"
							>{$t('restore.target_all')}</span
						>
					{:else}
						<span class="text-primary max-w-[200px] truncate font-medium">{selectionSummary}</span>
					{/if}
				</div>
				<div class="flex w-full gap-3">
					<button
						class="btn-action primary flex-1 justify-center"
						disabled={!selectedBackupId || isAppLocked || isLoadingFiles}
						on:click={() => initiateRestore('save')}
					>
						{#if !$isOnline}
							⚠️ {$t('errors.offline')}
						{:else}
							💾 {$t('restore.btn_save_as')}
						{/if}
					</button>

					<button
						class="btn-action primary flex-1 justify-center"
						disabled={!selectedBackupId || isAppLocked || isLoadingFiles || !canRestoreInPlace}
						on:click={() => initiateRestore('inplace')}
						title={!canRestoreInPlace && selectedBackupId ? $t('restore.drive_missing') : ''}
					>
						{#if !$isOnline}
							⚠️ {$t('errors.offline')}
						{:else if !canRestoreInPlace && selectedBackupId && !isLoadingFiles}
							❌ {$t('restore.btn_inplace')}
						{:else}
							🔄 {$t('restore.btn_inplace')}
						{/if}
					</button>
				</div>
			</div>
		</div>
	</div>
</div>
