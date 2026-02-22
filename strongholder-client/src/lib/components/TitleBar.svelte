<script lang="ts">
	import { getCurrentWindow } from '@tauri-apps/api/window';
	import { t } from '$lib/i18n';

	const appWindow = getCurrentWindow();

	function minimize() {
		appWindow.minimize();
	}

	function toggleMaximize() {
		appWindow.toggleMaximize();
	}

	function close() {
		appWindow.close();
	}
</script>

<div data-tauri-drag-region class="titlebar">
	<div class="pointer-events-none flex items-center gap-3 select-none">
		<img
			src="/strongholder_logo128x128.png"
			alt={$t('window.logo_alt')}
			class="h-7 w-7 object-contain"
		/>
		<span class="text-text-main text-xs font-bold tracking-wide uppercase opacity-80">
			{$t('common.brand_name')}
		</span>
	</div>

	<div class="window-controls">
		<button
			type="button"
			class="control-btn"
			on:click={minimize}
			title={$t('window.minimize')}
			aria-label={$t('window.minimize')}
		>
			<svg width="10" height="1" viewBox="0 0 10 1" aria-hidden="true">
				<rect width="10" height="1" fill="currentColor" />
			</svg>
		</button>

		<button
			type="button"
			class="control-btn"
			on:click={toggleMaximize}
			title={$t('window.maximize')}
			aria-label={$t('window.maximize')}
		>
			<svg width="10" height="10" viewBox="0 0 10 10" aria-hidden="true">
				<rect width="10" height="10" stroke="currentColor" stroke-width="1" fill="none" />
			</svg>
		</button>

		<button
			type="button"
			class="control-btn close"
			on:click={close}
			title={$t('window.close')}
			aria-label={$t('window.close')}
		>
			<svg width="10" height="10" viewBox="0 0 10 10" aria-hidden="true">
				<path
					d="M1 1L9 9M9 1L1 9"
					stroke="currentColor"
					stroke-width="1.2"
					stroke-linecap="round"
				/>
			</svg>
		</button>
	</div>
</div>
