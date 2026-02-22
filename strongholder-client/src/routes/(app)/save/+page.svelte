<script lang="ts">
	import { onMount, onDestroy } from 'svelte';
	import { invoke } from '@tauri-apps/api/core';
	import { t } from '$lib/i18n';
	import { isOnline } from '$lib/stores/connection';
	import {
		backupStatus,
		initBackupListener,
		executeBackup,
		cancelActiveBackup
	} from '$lib/stores/backup';
	import { slide, fade } from 'svelte/transition';
	import {
		type BackupPreset,
		createEmptyPreset,
		validateExtensionRule,
		loadPresetsFromDisk,
		saveSinglePresetToDisk,
		savePresetsToDisk,
		syncPresetScheduleToSystem
	} from '$lib/services/presetService';
	import { isLowBattery } from '$lib/stores/power';
	import { appConfigDir, join } from '@tauri-apps/api/path';
	import { getSafeFilename, getCronString } from '$lib/services/presetService';

	// --- TYPES ---
	interface FileEntry {
		name: string;
		path: string;
		isDirectory: boolean;
		isDrive?: boolean;
	}

	type ViewMode = 'dashboard' | 'editor';
	type BackupType = 'manual' | 'scheduled';

	// --- STATE ---
	let presets: BackupPreset[] = [];
	let viewMode: ViewMode = 'dashboard';
	let isBackupCancelled = false;

	$: scheduledPresets = presets.filter((p) => p.frequency !== 'manual');
	$: manualPresets = presets.filter((p) => p.frequency === 'manual');

	// --- HELPERS ---

	function getScheduleLabel(preset: BackupPreset): string {
		if (preset.frequency === 'manual') return $t('frequency.manual');
		if (preset.frequency === 'hourly') return $t('frequency.hourly');

		const time = preset.scheduleTime;

		if (preset.frequency === 'daily') {
			return $t('frequency.daily_at', { time });
		}

		if (preset.frequency === 'weekly') {
			const dayKey = `days.${preset.scheduleDay.toLowerCase().substring(0, 3)}`;
			const day = $t(dayKey);
			return $t('frequency.weekly_at', { day, time });
		}

		if (preset.frequency === 'monthly') {
			return $t('frequency.monthly_at', { day: preset.scheduleDay, time });
		}

		return '';
	}

	async function sendNotification(
		title: string,
		body: string,
		type: 'success' | 'error' | 'info' = 'info'
	) {
		try {
			await invoke('send_app_notification', { title, body, type });
		} catch (e) {
			console.error('Failed to send notification:', e);
		}
	}

	let config = { general: { battery_limit: true } };
	let globalError: string | null = null;
	let errorTimer: ReturnType<typeof setTimeout> | null = null;

	function showError(msg: string) {
		if (errorTimer) clearTimeout(errorTimer);
		globalError = msg;
		errorTimer = setTimeout(() => {
			globalError = null;
		}, 5000);
	}

	let showDeleteModal = false;
	let showUnsavedModal = false;
	let pendingPresetId: string | null = null;
	let presetToDeleteId: string | null = null;
	let editingId: string | null = null;
	let form: BackupPreset = createEmptyPreset();
	let originalFormState: string = '';
	let formType: BackupType = 'manual';
	let currentPath: string = '';
	let dirFiles: FileEntry[] = [];
	let isLoadingDir: boolean = false;
	let editorTab: 'files' | 'rules' = 'files';
	let newExtInput = '';
	let selectedMonitorId: string | null = null;

	$: activePreset = presets.find((p) => p.id === selectedMonitorId);
	$: isRunning = (id: string) => {
		return (
			$backupStatus.presetId === id &&
			['preparing', 'running', 'success'].includes($backupStatus.state)
		);
	};

	onMount(async () => {
		try {
			presets = await loadPresetsFromDisk();
			if (presets.length > 0) selectedMonitorId = presets[0].id;
		} catch (e) {
			showError($t('errors.load_presets_failed'));
		}
	});

	onDestroy(() => {
		if (errorTimer) clearTimeout(errorTimer);
	});

	let cachedDrives: string[] | null = null;

	async function loadDrivesOrPath(path: string) {
		isLoadingDir = true;
		try {
			if (!path) {
				if (!cachedDrives) {
					cachedDrives = await invoke<string[]>('get_drives').catch(() => {
						console.warn('Failed to get drives from Rust. Using fallback.');
						return navigator.userAgent.includes('Win') ? ['C:\\'] : ['/'];
					});
				}
				currentPath = '';
				dirFiles = (cachedDrives || []).map((d) => ({
					name: d,
					path: d,
					isDirectory: true,
					isDrive: true
				}));
			} else {
				const result = await invoke<{ path: string; files: FileEntry[] }>('list_directory', {
					path
				});
				currentPath = result.path;
				dirFiles = result.files;
			}
		} catch (e) {
			showError($t('errors.dir_access'));
		} finally {
			isLoadingDir = false;
		}
	}

	function goUp() {
		if (!currentPath) return;
		const sep = currentPath.includes('\\') ? '\\' : '/';
		const parts = currentPath.split(sep).filter(Boolean);
		if (parts.length <= 1 && currentPath.includes(':')) return loadDrivesOrPath('');
		parts.pop();
		const parent =
			parts.length === 0 ? (sep === '\\' ? '' : '/') : parts.join(sep) + (sep === '\\' ? '\\' : '');
		loadDrivesOrPath(parent);
	}

	function addToSources(path: string) {
		if (form.sources.includes(path)) return;
		if (form.exclusions.includes(path)) {
			showError($t('errors.conflict_exclusion'));
			return;
		}
		form.sources = [...form.sources, path];
	}

	function addToExclusions(path: string) {
		if (form.exclusions.includes(path)) return;
		if (form.sources.includes(path)) {
			showError($t('errors.conflict_source'));
			return;
		}
		form.exclusions = [...form.exclusions, path];
	}

	function handlePresetClick(id: string) {
		if (id === selectedMonitorId && viewMode === 'dashboard') return;
		if (viewMode === 'editor' && JSON.stringify(form) !== originalFormState) {
			pendingPresetId = id;
			showUnsavedModal = true;
		} else {
			selectedMonitorId = id;
			viewMode = 'dashboard';
			editingId = null;
		}
	}

	function openEditor(preset?: BackupPreset) {
		globalError = null;
		if (viewMode === 'editor' && JSON.stringify(form) !== originalFormState) {
			showUnsavedModal = true;
			pendingPresetId = 'NEW';
			return;
		}
		if (preset) {
			form = JSON.parse(JSON.stringify(preset));
			editingId = preset.id;
			formType = form.frequency === 'manual' ? 'manual' : 'scheduled';
		} else {
			form = createEmptyPreset();
			form.frequency = 'monthly';
			form.scheduleDay = '1';
			form.scheduleTime = '12:00';
			formType = 'manual';
			editingId = null;
		}
		originalFormState = JSON.stringify(form);
		viewMode = 'editor';
		loadDrivesOrPath('');
	}

	function closeEditor() {
		if (JSON.stringify(form) !== originalFormState) {
			showUnsavedModal = true;
			return;
		}
		confirmDiscard();
	}

	function confirmDiscard() {
		showUnsavedModal = false;
		if (pendingPresetId === 'NEW') {
			pendingPresetId = null;
			openEditor();
		} else if (pendingPresetId) {
			selectedMonitorId = pendingPresetId;
			viewMode = 'dashboard';
			pendingPresetId = null;
		} else {
			viewMode = 'dashboard';
		}
		editingId = null;
	}

	function addRule(mode: 'include' | 'exclude') {
		if (!newExtInput || newExtInput.trim() === '') return;
		let pattern = newExtInput.trim();
		if (!pattern.startsWith('*.')) pattern = `*.${pattern}`;
		const cleanExt = pattern.replace('*.', '');
		const validRegex = /^[a-zA-Z0-9]+$/;
		if (!validRegex.test(cleanExt)) {
			showError($t('errors.invalid_chars'));
			return;
		}
		if (form.extensionRules.some((r) => r.pattern === pattern && r.mode === mode)) {
			showError($t('errors.rule_duplicate'));
			return;
		}
		if (form.extensionRules.some((r) => r.pattern === pattern && r.mode !== mode)) {
			showError($t('errors.rule_conflict'));
			return;
		}
		if (!validateExtensionRule(form.extensionRules, pattern)) {
			showError($t('save.error_conflict'));
			return;
		}
		form.extensionRules = [...form.extensionRules, { pattern, mode }];
		newExtInput = '';
	}

	async function saveForm() {
		if (!form.name || form.name.trim() === '') {
			showError($t('errors.name_required'));
			return;
		}
		const nameRegex = /^[a-zA-Z0-9\s\-_]+$/;
		if (!nameRegex.test(form.name)) {
			showError($t('errors.name_invalid'));
			return;
		}
		if (
			presets.some(
				(p) => p.name.toLowerCase() === form.name.trim().toLowerCase() && p.id !== editingId
			)
		) {
			showError($t('errors.name_duplicate'));
			return;
		}
		if (form.sources.length === 0) {
			showError($t('errors.source_required'));
			return;
		}
		if (formType === 'manual') {
			form.frequency = 'manual';
			form.paused = true;
		} else {
			if (form.frequency === 'manual') form.frequency = 'daily';
			if (form.frequency === 'monthly') {
				const day = parseInt(form.scheduleDay);
				if (isNaN(day) || day < 1 || day > 28) {
					showError($t('errors.invalid_day'));
					return;
				}
			}
			if (!editingId) form.paused = false;
		}
		let newPresetsList = editingId
			? presets.map((p) => (p.id === editingId ? form : p))
			: [...presets, form];

		try {
			await saveSinglePresetToDisk(newPresetsList, form);

			presets = newPresetsList;

			syncPresetScheduleToSystem(form).catch((e) => {
				console.error('Background Scheduler sync failed:', e);
			});

			if (!editingId) selectedMonitorId = form.id;
			originalFormState = JSON.stringify(form);
			viewMode = 'dashboard';
			editingId = null;
		} catch (e) {
			showError($t('errors.save_failed'));
		}
	}

	function requestDelete(id: string) {
		presetToDeleteId = id;
		showDeleteModal = true;
	}

	async function confirmDelete() {
		if (!presetToDeleteId) return;
		const presetObj = presets.find((p) => p.id === presetToDeleteId);
		const newPresetsList = presets.filter((p) => p.id !== presetToDeleteId);

		try {
			await savePresetsToDisk(newPresetsList);

			if (presetObj && presetObj.frequency !== 'manual') {
				invoke('update_backup_schedule', {
					username: localStorage.getItem('username'),
					presetId: presetObj.id,
					cronString: '',
					presetPath: '',
					clientId: '',
					enabled: false
				});
			}

			presets = newPresetsList;
			if (selectedMonitorId === presetToDeleteId) {
				selectedMonitorId = presets.length > 0 ? presets[0].id : null;
			}
			viewMode = 'dashboard';
		} catch (e) {
			showError($t('errors.delete_failed'));
		} finally {
			showDeleteModal = false;
			presetToDeleteId = null;
		}
	}

	async function launchBackup(preset: BackupPreset) {
		if (!$isOnline) {
			const msg = $t('errors.no_internet');
			showError(msg);
			sendNotification($t('backup.notif_error_title'), msg, 'error');
			return;
		}

		// @ts-ignore
		if (typeof navigator.getBattery === 'function') {
			// @ts-ignore
			const battery = await navigator.getBattery();
			if (!battery.charging && battery.level < 0.2) {
				const msg = $t('errors.low_battery');
				showError(msg);
				sendNotification($t('backup.notif_error_title'), msg, 'error');
				return;
			}
		}

		sendNotification(
			$t('backup.notif_started_title'),
			$t('backup.notif_started_body', { name: preset.name }),
			'info'
		);

		try {
			const configDir = await appConfigDir();
			const filename = getSafeFilename(preset.name);
			const fullPresetPath = await join(configDir, filename);

			const clientId = localStorage.getItem('client_id');
			const username = localStorage.getItem('username');

			if (!clientId || !username) {
				throw new Error($t('errors.identity_missing'));
			}

			await executeBackup(
				preset.id,
				preset.name,
				fullPresetPath,
				clientId,
				username,
				$t('backup.cancelled_by_user') || 'Backup cancelled by user',
				$t('errors.backup_failed'),
				$t('errors.unknown')
			);
		} catch (e: unknown) {
			let safeErrorMsg = $t('errors.unknown');
			if (e instanceof Error) safeErrorMsg = e.message;
			else if (typeof e === 'string') safeErrorMsg = e;

			showError(`${$t('errors.backup_failed')}: ${safeErrorMsg}`);
		}
	}

	async function cancelBackup() {
		try {
			const cancelMsg = $t('backup.cancelled_by_user') || 'Backup cancelled by user';
			await cancelActiveBackup(cancelMsg);
		} catch (e: unknown) {
			let safeErrorMsg = $t('errors.unknown');
			if (e instanceof Error) safeErrorMsg = e.message;
			else if (typeof e === 'string') safeErrorMsg = e;

			showError(`${$t('errors.cancel_failed')}: ${safeErrorMsg}`);
		}
	}

	async function toggleSchedule(preset: BackupPreset) {
		if (preset.frequency === 'manual') return;

		const original = preset.paused;
		preset.paused = !original;
		presets = [...presets];

		try {
			await saveSinglePresetToDisk(presets, preset);

			syncPresetScheduleToSystem(preset).catch((e) => {
				console.error('Toggle sync failed:', e);
				showError($t('errors.schedule_update_failed'));
			});
		} catch (e) {
			preset.paused = original;
			presets = [...presets];
			showError($t('errors.update_failed'));
		}
	}
</script>

<div
	class="border-border bg-surface flex h-[calc(100%-40px)] w-full items-start justify-start overflow-hidden rounded-2xl border shadow-sm"
>
	{#if showDeleteModal}
		<div class="modal-overlay" transition:fade={{ duration: 100 }}>
			<div class="modal-content" role="dialog" aria-modal="true">
				<h3 class="text-text-main mb-2 text-lg font-bold">{$t('save.confirm_delete_title')}</h3>
				<p class="text-text-muted mb-6 text-sm">{$t('save.delete_warning')}</p>
				<div class="flex justify-end gap-3">
					<button class="btn-action secondary" on:click={() => (showDeleteModal = false)}
						>{$t('common.cancel')}</button
					>
					<button class="btn-action danger" on:click={confirmDelete}>{$t('common.delete')}</button>
				</div>
			</div>
		</div>
	{/if}

	{#if showUnsavedModal}
		<div class="modal-overlay" transition:fade={{ duration: 100 }}>
			<div class="modal-content" role="dialog" aria-modal="true">
				<h3 class="text-text-main mb-2 text-lg font-bold">{$t('save.unsaved_title')}</h3>
				<p class="text-text-muted mb-6 text-sm">{$t('save.unsaved_msg')}</p>
				<div class="flex justify-end gap-3">
					<button class="btn-action secondary" on:click={() => (showUnsavedModal = false)}
						>{$t('common.cancel')}</button
					>
					<button class="btn-action danger" on:click={confirmDiscard}>{$t('save.discard')}</button>
				</div>
			</div>
		</div>
	{/if}

	<aside
		class="border-border bg-surface flex h-full w-[350px] shrink-0 flex-col overflow-hidden border-r transition-all duration-300"
	>
		<div class="border-border flex items-center justify-between border-b p-6">
			<h3 class="text-primary m-0 text-xl font-bold"><b>{$t('save.aside_title')}</b></h3>
			<button
				class="bg-primary/10 text-primary hover:bg-primary flex h-8 w-8 items-center justify-center rounded-full transition-colors hover:text-white"
				on:click={() => openEditor()}
				aria-label={$t('save.add_new_aria')}>+</button
			>
		</div>

		<div class="flex flex-1 flex-col gap-2 overflow-y-auto p-4">
			{#if scheduledPresets.length > 0}
				<div
					class="border-border bg-bg-subtle text-text-muted mt-2 rounded border-y px-4 py-2 text-[10px] font-bold tracking-wider uppercase first:mt-0"
				>
					{$t('frequency.scheduled')}
				</div>
				{#each scheduledPresets as preset (preset.id)}
					<button
						class="relative mb-2 w-full cursor-pointer rounded-xl border p-3 text-left transition-all duration-200 {selectedMonitorId ===
						preset.id
							? 'border-primary ring-primary bg-primary/5 ring-1'
							: 'hover:border-primary/50 border-border bg-surface hover:shadow-sm'}"
						on:click={() => handlePresetClick(preset.id)}
					>
						<div class="mb-1 flex items-center justify-between">
							<span class="text-text-main truncate pr-2 text-sm font-bold">{preset.name}</span>
							<div
								class="relative h-5 w-9 cursor-pointer rounded-full border-none p-0 transition-colors duration-300 {!preset.paused
									? 'bg-status-added'
									: 'bg-gray-400'}"
								on:click|stopPropagation={() => toggleSchedule(preset)}
								role="switch"
								aria-checked={!preset.paused}
								aria-label={$t('save.toggle_schedule_aria')}
							>
								<span
									class="absolute top-[2px] left-[2px] h-4 w-4 rounded-full bg-white shadow-sm transition-transform duration-300"
									style="transform: {!preset.paused ? 'translateX(16px)' : 'translateX(0)'}"
								></span>
							</div>
						</div>
						<div class="text-primary mb-1 text-[11px] font-bold">{getScheduleLabel(preset)}</div>
						<div class="text-text-muted flex gap-3 text-xs">
							<span>📂 {preset.sources.length}</span><span>🚫 {preset.exclusions.length}</span>
						</div>
						{#if isRunning(preset.id)}
							<div class="bg-bg mt-2 h-1 w-full overflow-hidden rounded-full">
								<div
									class="bg-primary h-full transition-all duration-300"
									style="width:{$backupStatus.progress}%"
								></div>
							</div>
						{/if}
					</button>
				{/each}
			{/if}

			{#if manualPresets.length > 0}
				<div
					class="border-border bg-bg-subtle text-text-muted mt-2 rounded border-y px-4 py-2 text-[10px] font-bold tracking-wider uppercase"
				>
					{$t('frequency.manual')}
				</div>
				{#each manualPresets as preset (preset.id)}
					<button
						class="relative mb-2 w-full cursor-pointer rounded-xl border p-3 text-left transition-all duration-200 {selectedMonitorId ===
						preset.id
							? 'border-primary ring-primary bg-primary/5 ring-1'
							: 'hover:border-primary/50 border-border bg-surface hover:shadow-sm'}"
						on:click={() => handlePresetClick(preset.id)}
					>
						<div class="mb-1 flex items-center justify-between">
							<span class="text-text-main truncate pr-2 text-sm font-bold">{preset.name}</span><span
								class="badge badge-manual">M</span
							>
						</div>
						<div class="text-text-muted flex gap-3 text-xs">
							<span>📂 {preset.sources.length}</span><span>🚫 {preset.exclusions.length}</span>
						</div>
						{#if isRunning(preset.id)}
							<div class="bg-bg mt-2 h-1 w-full overflow-hidden rounded-full">
								<div
									class="bg-primary h-full transition-all duration-300"
									style="width:{$backupStatus.progress}%"
								></div>
							</div>
						{/if}
					</button>
				{/each}
			{/if}

			{#if presets.length === 0}
				<div class="text-text-muted p-8 text-center text-sm">
					<p class="mb-2">{$t('save.empty_list')}</p>
				</div>
			{/if}
		</div>
	</aside>

	<main class="bg-bg relative flex h-full flex-1 flex-col overflow-hidden">
		{#if globalError}
			<div
				transition:slide
				class="absolute top-4 left-1/2 z-50 flex -translate-x-1/2 items-center gap-4 rounded-lg border border-red-200 bg-red-50 px-6 py-3 text-sm font-bold text-red-700 shadow-lg"
				role="alert"
			>
				<span>⚠️ {globalError}</span>
				<button
					on:click={() => (globalError = null)}
					class="opacity-50 hover:opacity-100"
					aria-label={$t('common.close')}>✕</button
				>
			</div>
		{/if}

		{#if viewMode === 'editor'}
			<div
				class="border-border bg-surface flex shrink-0 items-start justify-between border-b px-8 py-6"
			>
				<div>
					<h2 class="text-text-main text-xl font-bold">
						{editingId ? $t('save.edit_title') : $t('save.new_title')}
					</h2>
					<p class="text-text-muted mt-1 text-xs">{$t('save.settings_general')}</p>
				</div>
				<div class="flex gap-3">
					{#if editingId}<button
							class="btn-action danger"
							on:click={() => requestDelete(editingId!)}>{$t('common.delete')}</button
						>{/if}
					<button class="btn-action secondary" on:click={closeEditor}>{$t('common.cancel')}</button>
					<button
						class="btn-action primary"
						disabled={!$isOnline || ($isLowBattery && config.general.battery_limit)}
						on:click={saveForm}>{$t('common.save')}</button
					>
				</div>
			</div>

			<div class="flex-1 overflow-y-auto p-8">
				<div class="border-border bg-surface mb-6 rounded-xl border p-6 shadow-sm">
					<div class="grid grid-cols-1 gap-6 md:grid-cols-2">
						<div>
							<label class="text-text-muted mb-2 block text-xs font-bold uppercase"
								>{$t('save.field_name')}</label
							>
							<input
								type="text"
								bind:value={form.name}
								class="form-input"
								placeholder={$t('save.placeholder_name')}
							/>
						</div>
						<div>
							<label class="text-text-muted mb-2 block text-xs font-bold uppercase"
								>{$t('save.field_freq_type')}</label
							>
							<div class="flex gap-4 pt-1">
								<label
									class="hover:bg-bg flex cursor-pointer items-center gap-2 rounded-lg border border-transparent p-2 transition-colors"
								>
									<input
										type="radio"
										bind:group={formType}
										value="manual"
										class="accent-primary h-4 w-4"
									/>
									<span class="text-text-main text-sm font-medium">{$t('frequency.manual')}</span>
								</label>
								<label
									class="hover:bg-bg flex cursor-pointer items-center gap-2 rounded-lg border border-transparent p-2 transition-colors"
								>
									<input
										type="radio"
										bind:group={formType}
										value="scheduled"
										class="accent-primary h-4 w-4"
									/>
									<span class="text-text-main text-sm font-medium">{$t('frequency.scheduled')}</span
									>
								</label>
							</div>
						</div>
					</div>

					{#if formType === 'scheduled'}
						<div
							class="border-border mt-6 grid grid-cols-1 gap-6 border-t pt-6 md:grid-cols-3"
							transition:slide
						>
							<div>
								<label class="text-text-muted mb-2 block text-xs font-bold uppercase"
									>{$t('save.field_freq_select')}</label
								>
								<select bind:value={form.frequency} class="bg-surface form-input cursor-pointer">
									<option value="hourly">{$t('frequency.hourly')}</option>
									<option value="daily">{$t('frequency.daily')}</option>
									<option value="weekly">{$t('frequency.weekly')}</option>
									<option value="monthly">{$t('frequency.monthly')}</option>
								</select>
							</div>
							{#if form.frequency !== 'hourly'}
								<div>
									<label class="text-text-muted mb-2 block text-xs font-bold uppercase"
										>{$t('save.field_time')}</label
									><input type="time" bind:value={form.scheduleTime} class="form-input" />
								</div>
							{/if}
							{#if form.frequency === 'weekly'}
								<div>
									<label class="text-text-muted mb-2 block text-xs font-bold uppercase"
										>{$t('save.field_day_week')}</label
									>
									<select
										bind:value={form.scheduleDay}
										class="bg-surface form-input cursor-pointer"
									>
										{#each ['Monday', 'Tuesday', 'Wednesday', 'Thursday', 'Friday', 'Saturday', 'Sunday'] as day}
											<option value={day}>{$t(`days.${day.toLowerCase().substring(0, 3)}`)}</option>
										{/each}
									</select>
								</div>
							{:else if form.frequency === 'monthly'}
								<div>
									<label class="text-text-muted mb-2 block text-xs font-bold uppercase"
										>{$t('save.field_day_month')}</label
									><input
										type="number"
										min="1"
										max="28"
										bind:value={form.scheduleDay}
										class="form-input"
									/>
								</div>
							{/if}
						</div>
					{/if}
				</div>

				<div
					class="border-border bg-surface flex h-[500px] flex-col overflow-hidden rounded-xl border shadow-sm"
				>
					<div class="border-border bg-bg-subtle flex border-b">
						<button
							class="flex-1 py-4 text-sm font-bold transition-colors {editorTab === 'files'
								? 'text-primary border-primary bg-surface border-t-2'
								: 'text-text-muted hover:text-text-main'}"
							on:click={() => (editorTab = 'files')}>📁 {$t('save.tab_files')}</button
						>
						<button
							class="flex-1 py-4 text-sm font-bold transition-colors {editorTab === 'rules'
								? 'text-primary border-primary bg-surface border-t-2'
								: 'text-text-muted hover:text-text-main'}"
							on:click={() => (editorTab = 'rules')}>⚙️ {$t('save.tab_rules')}</button
						>
					</div>

					<div class="relative flex-1 overflow-hidden p-0">
						{#if editorTab === 'files'}
							<div class="flex h-full flex-col">
								<div class="border-border bg-bg-subtle flex items-center gap-3 border-b p-3">
									<button
										class="border-border bg-surface hover:bg-bg rounded-md border p-1.5"
										on:click={goUp}
										title={$t('save.go_up_tooltip')}>⬆️</button
									>
									<div class="relative flex-1">
										<input
											type="text"
											value={currentPath || $t('save.drives_label')}
											readonly
											class="border-border bg-surface text-text-main w-full rounded-md border px-3 py-1.5 font-mono text-sm outline-none"
										/>
									</div>
								</div>
								<div class="flex flex-1 overflow-hidden">
									<div class="border-border flex w-2/3 flex-col border-r">
										<div class="bg-surface flex-1 overflow-y-auto">
											{#if isLoadingDir}<div class="text-text-muted p-8 text-center text-sm">
													{$t('save.loading_dir')}
												</div>
											{:else}
												{#each dirFiles as file}
													<div
														class="group border-border bg-surface hover:bg-bg-subtle flex cursor-pointer items-center justify-between border-b p-2.5 text-sm transition-colors"
														role="button"
														tabindex="0"
														on:dblclick={() =>
															(file.isDirectory || file.isDrive) && loadDrivesOrPath(file.path)}
													>
														<div class="flex flex-1 items-center gap-3 overflow-hidden">
															<span class="text-lg opacity-60"
																>{file.isDrive ? '💽' : file.isDirectory ? '📁' : '📄'}</span
															><span class="text-text-main truncate">{file.name}</span>
														</div>
														<div
															class="flex gap-2 opacity-0 transition-opacity group-focus-within:opacity-100 group-hover:opacity-100"
														>
															<button
																class="rounded border border-green-200 bg-[#f0fdf4] px-2.5 py-1 text-[10px] font-bold text-[#166534] hover:bg-green-100 dark:border-green-800 dark:bg-green-900/30 dark:text-green-300"
																on:click|stopPropagation={() => addToSources(file.path)}
																>{$t('save.btn_add_src')}</button
															>
															<button
																class="rounded border border-red-200 bg-[#fef2f2] px-2.5 py-1 text-[10px] font-bold text-[#991b1b] hover:bg-red-100 dark:border-red-800 dark:bg-red-900/30 dark:text-red-300"
																on:click|stopPropagation={() => addToExclusions(file.path)}
																>{$t('save.btn_add_exc')}</button
															>
														</div>
													</div>
												{/each}
											{/if}
										</div>
									</div>
									<div class="bg-bg-subtle flex w-1/3 flex-col">
										<div class="border-border flex-1 overflow-y-auto border-b p-4">
											<h4
												class="text-status-added mb-3 flex items-center gap-2 text-xs font-bold uppercase"
											>
												<span class="bg-status-added h-2 w-2 rounded-full"></span>
												{$t('save.sources_title')}
											</h4>
											{#each form.sources as src}
												<div
													class="group border-border bg-surface mb-2 flex items-center justify-between rounded border p-2 text-xs shadow-sm"
												>
													<span class="text-text-main flex-1 truncate font-mono">{src}</span><button
														class="text-text-muted ml-2 font-bold opacity-0 group-hover:opacity-100 hover:text-red-500"
														on:click={() => (form.sources = form.sources.filter((s) => s !== src))}
														>×</button
													>
												</div>
											{/each}
										</div>
										<div class="flex-1 overflow-y-auto bg-[#fef2f2]/30 p-4 dark:bg-red-900/10">
											<h4
												class="text-status-deleted mb-3 flex items-center gap-2 text-xs font-bold uppercase"
											>
												<span class="bg-status-deleted h-2 w-2 rounded-full"></span>
												{$t('save.exclusions_title')}
											</h4>
											{#each form.exclusions as exc}
												<div
													class="group bg-surface mb-2 flex items-center justify-between rounded border border-red-100 p-2 text-xs shadow-sm dark:border-red-900/50"
												>
													<span class="text-text-main flex-1 truncate font-mono">{exc}</span><button
														class="text-text-muted ml-2 font-bold opacity-0 group-hover:opacity-100 hover:text-red-500"
														on:click={() =>
															(form.exclusions = form.exclusions.filter((e) => e !== exc))}
														>×</button
													>
												</div>
											{/each}
										</div>
									</div>
								</div>
							</div>
						{:else}
							<div class="flex h-full flex-col">
								<div class="border-border bg-surface border-b p-4">
									<div
										class="bg-bg-subtle border-border mb-4 flex items-center justify-between rounded-lg border p-3"
									>
										<div>
											<span class="text-text-main text-sm font-bold">
												{form.includeOnlyMode
													? $t('save.mode_include_only')
													: $t('save.mode_exclusion')}
											</span>
											<p class="text-text-muted text-[10px]">
												{form.includeOnlyMode
													? $t('save.desc_include_only')
													: $t('save.desc_exclusion')}
											</p>
										</div>
										<button
											class="relative h-5 w-9 cursor-pointer rounded-full border-none p-0 transition-all {form.includeOnlyMode
												? 'bg-primary'
												: 'bg-gray-400'}"
											on:click={() => {
												form.includeOnlyMode = !form.includeOnlyMode;
												newExtInput = ''; // Clear input on swap
											}}
										>
											<span
												class="absolute top-[2px] left-[2px] h-4 w-4 rounded-full bg-white transition-transform"
												style="transform: {form.includeOnlyMode
													? 'translateX(16px)'
													: 'translateX(0)'}"
											></span>
										</button>
									</div>

									<div class="flex items-center gap-3">
										<input
											type="text"
											bind:value={newExtInput}
											class="form-input"
											placeholder={form.includeOnlyMode
												? // REPLACED: Hardcoded placeholder
													$t('save.placeholder_include')
												: $t('save.placeholder_exclude')}
											on:keydown={(e) =>
												e.key === 'Enter' && addRule(form.includeOnlyMode ? 'include' : 'exclude')}
										/>

										{#if form.includeOnlyMode}
											<button
												class="btn-action success w-32 justify-center"
												on:click={() => addRule('include')}
											>
												{$t('save.btn_include')}
											</button>
										{:else}
											<button
												class="btn-action danger w-32 justify-center"
												on:click={() => addRule('exclude')}
											>
												{$t('save.btn_exclude')}
											</button>
										{/if}
									</div>
								</div>

								<div class="flex-1 overflow-y-auto p-4">
									{#if form.includeOnlyMode}
										<div in:fade={{ duration: 200 }} class="h-full">
											<h4
												class="text-status-added mb-3 flex items-center gap-2 text-xs font-bold uppercase"
											>
												{$t('save.included_label')}
											</h4>
											<div class="flex flex-wrap gap-2">
												{#each form.extensionRules.filter((r) => r.mode === 'include') as rule}
													<div
														class="flex items-center gap-2 rounded border border-green-200 bg-green-50 px-3 py-1.5 text-xs font-bold text-green-700"
													>
														<span class="font-mono">{rule.pattern.replace(/[^a-z0-9]/gi, '')}</span>
														<button
															on:click={() =>
																(form.extensionRules = form.extensionRules.filter(
																	(r) => r !== rule
																))}>×</button
														>
													</div>
												{/each}
												<div
													class="flex items-center gap-2 rounded border border-dashed border-red-300 bg-transparent px-3 py-1.5 text-xs font-bold text-red-400 opacity-60"
												>
													<span class="font-mono">- **</span>
													<span class="text-[9px]">{$t('save.auto_exclude_others')}</span>
												</div>
											</div>
										</div>
									{:else}
										<div in:fade={{ duration: 200 }} class="h-full">
											<h4
												class="text-status-deleted mb-3 flex items-center gap-2 text-xs font-bold uppercase"
											>
												{$t('save.excluded_label')}
											</h4>
											<div class="flex flex-wrap gap-2">
												{#each form.extensionRules.filter((r) => r.mode === 'exclude') as rule}
													<div
														class="flex items-center gap-2 rounded border border-red-200 bg-red-50 px-3 py-1.5 text-xs font-bold text-red-700"
													>
														<span class="font-mono">{rule.pattern.replace(/[^a-z0-9]/gi, '')}</span>
														<button
															on:click={() =>
																(form.extensionRules = form.extensionRules.filter(
																	(r) => r !== rule
																))}>×</button
														>
													</div>
												{/each}
											</div>
										</div>
									{/if}
								</div>
							</div>
						{/if}
					</div>
				</div>
			</div>
		{:else if activePreset}
			<div
				class="border-border bg-surface flex shrink-0 items-start justify-between border-b px-8 py-6"
			>
				<div>
					<h1 class="text-text-main mb-1 flex items-center gap-2 text-2xl font-bold">
						{activePreset.name}<span class="badge badge-manual"
							>{activePreset.frequency === 'manual'
								? $t('frequency.manual')
								: $t(`frequency.${activePreset.frequency}`)}</span
						>
					</h1>
					<div class="text-primary text-sm font-bold">{getScheduleLabel(activePreset)}</div>
				</div>
				<div class="flex gap-3">
					<button class="btn-action secondary" on:click={() => openEditor(activePreset)}>
						⚙️ {$t('common.edit')}
					</button>

					{#if isRunning(activePreset.id)}
						<button
							class="btn-action danger flex animate-pulse items-center gap-2"
							on:click={cancelBackup}
						>
							⏹ {$t('common.cancel')}
						</button>
					{:else}
						<button
							class="btn-action primary"
							disabled={!$isOnline}
							on:click={() => launchBackup(activePreset)}
						>
							▶ {$t('backup.manual_launch')}
						</button>
					{/if}
				</div>
			</div>

			<div class="flex-1 overflow-y-auto p-8">
				{#if isRunning(activePreset.id)}
					<div
						class="border-primary/20 bg-surface mb-8 rounded-xl border p-6 shadow-sm"
						transition:slide
					>
						<div class="mb-2 flex items-end justify-between">
							<div>
								<h4 class="text-primary mb-1 text-xs font-bold tracking-wide uppercase">
									{$t(`backup.state_${$backupStatus.state}`)}
								</h4>
								<span class="text-text-main text-3xl font-bold">{$backupStatus.progress}%</span>
							</div>
							<span class="text-text-muted max-w-[200px] truncate font-mono text-xs">
								{$backupStatus.currentFile}
							</span>
						</div>
						<div class="bg-bg h-3 w-full overflow-hidden rounded-full">
							<div
								class="bg-primary h-full transition-all duration-300 ease-out"
								style="width: {$backupStatus.progress}%"
							></div>
						</div>
					</div>
				{/if}

				<div class="grid grid-cols-1 gap-6 md:grid-cols-2">
					<div class="border-border bg-surface rounded-xl border p-6 shadow-sm">
						<h4 class="text-text-muted mb-4 text-xs font-bold uppercase">
							📂 {$t('save.sources_title')}
						</h4>
						<ul class="max-h-40 space-y-2 overflow-y-auto">
							{#each activePreset.sources as src}<li
									class="text-text-main flex gap-2 font-mono text-sm"
								>
									<span class="text-status-added">↳</span>{src}
								</li>{/each}
						</ul>
					</div>
					<div class="border-border bg-surface rounded-xl border p-6 shadow-sm">
						<h4 class="text-text-muted mb-4 text-xs font-bold uppercase">
							🚫 {$t('save.exclusions_title')}
						</h4>
						{#if activePreset.exclusions.length === 0}<p class="text-text-muted text-sm italic">
								{$t('save.none_label')}
							</p>{:else}<ul class="max-h-40 space-y-2 overflow-y-auto">
								{#each activePreset.exclusions as excl}<li
										class="text-text-main flex gap-2 font-mono text-sm"
									>
										<span class="text-status-deleted">x</span>{excl}
									</li>{/each}
							</ul>{/if}
					</div>
					<div
						class="border-border bg-surface col-span-1 rounded-xl border p-6 shadow-sm md:col-span-2"
					>
						<h4 class="text-text-muted mb-4 text-xs font-bold uppercase">
							📜 {$t('save.rules_header')}
						</h4>

						<div class="grid grid-cols-1 gap-4">
							{#if activePreset.includeOnlyMode}
								<div class="rounded-lg border border-green-50 bg-green-50/20 p-4">
									<h5 class="text-status-added mb-2 text-[10px] font-bold uppercase">
										{$t('save.include_only_mode_active')}
									</h5>
									<div class="flex flex-wrap gap-2">
										{#each activePreset.extensionRules.filter((r) => r.mode === 'include') as rule}
											<span
												class="inline-flex items-center rounded border border-green-200 bg-green-100 px-2 py-0.5 text-[10px] font-bold text-green-800 uppercase"
											>
												{rule.pattern.replace(/[^a-z0-9]/gi, '')}
											</span>
										{/each}
										<span
											class="inline-flex items-center rounded border border-red-200 bg-red-50 px-2 py-0.5 text-[10px] font-bold text-red-400 italic"
										>
											{$t('save.exclude_others')}
										</span>
									</div>
								</div>
							{:else}
								<div class="rounded-lg border border-red-50 bg-red-50/20 p-4">
									<h5 class="text-status-deleted mb-2 text-[10px] font-bold uppercase">
										{$t('save.exclusions_mode_active')}
									</h5>
									<div class="flex flex-wrap gap-2">
										{#each activePreset.extensionRules.filter((r) => r.mode === 'exclude') as rule}
											<span
												class="inline-flex items-center rounded border border-red-200 bg-red-100 px-2 py-0.5 text-[10px] font-bold text-red-800 uppercase"
											>
												{rule.pattern.replace(/[^a-z0-9]/gi, '')}
											</span>
										{/each}
										{#if !activePreset.extensionRules.some((r) => r.mode === 'exclude')}
											<span class="text-text-muted text-xs italic">{$t('save.none_label')}</span>
										{/if}
									</div>
								</div>
							{/if}
						</div>
					</div>
				</div>
			</div>
		{:else}
			<div class="text-text-muted flex h-full flex-col items-center justify-center">
				<div class="mb-4 text-6xl" role="img" aria-label={$t('save.icon_label')}>💾</div>
				<p>{$t('save.no_presets_selected')}</p>
			</div>
		{/if}
	</main>
</div>
