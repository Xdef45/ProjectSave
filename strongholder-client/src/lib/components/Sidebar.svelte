<script lang="ts">
	import { page } from '$app/stores';
	import { isSidebarOpen } from '$lib/stores/ui';
	import { t } from '$lib/i18n';

	function isActive(path: string): boolean {
		return $page.url.pathname.startsWith(path);
	}
</script>

<aside
	class="border-border bg-surface flex h-full flex-col overflow-hidden border-r transition-all duration-300 ease-in-out"
	style="width: {$isSidebarOpen ? '260px' : '0px'}; opacity: {$isSidebarOpen ? '1' : '0'};"
	aria-hidden={!$isSidebarOpen}
>
	<div class="border-border flex h-16 shrink-0 items-center border-b px-6">
		<h3 class="text-primary text-xl font-bold tracking-tight"><b>{$t('sidebar.brand')}</b></h3>
	</div>

	<nav class="flex-1 space-y-1 overflow-y-auto p-4" aria-label={$t('sidebar.nav_aria')}>
		<a
			href="/dashboard"
			class="flex items-center gap-3 rounded-lg px-3 py-2.5 text-sm font-medium transition-colors
            {isActive('/dashboard')
				? 'bg-primary/10 text-primary'
				: 'text-text-muted hover:bg-bg-subtle'}"
		>
			<span aria-hidden="true">🏠</span>
			<span>{$t('sidebar.dashboard')}</span>
		</a>

		<a
			href="/save"
			class="flex items-center gap-3 rounded-lg px-3 py-2.5 text-sm font-medium transition-colors
            {isActive('/save')
				? 'bg-primary/10 text-primary'
				: 'text-text-muted hover:bg-bg-subtle'}"
		>
			<span aria-hidden="true">💾</span>
			<span>{$t('sidebar.save')}</span>
		</a>

		<a
			href="/restore"
			class="flex items-center gap-3 rounded-lg px-3 py-2.5 text-sm font-medium transition-colors
            {isActive('/restore')
				? 'bg-primary/10 text-primary'
				: 'text-text-muted hover:bg-bg-subtle'}"
		>
			<span aria-hidden="true">🔄️</span>
			<span>{$t('sidebar.restore')}</span>
		</a>

		<a
			href="/logs"
			class="flex items-center gap-3 rounded-lg px-3 py-2.5 text-sm font-medium transition-colors
            {isActive('/logs')
				? 'bg-primary/10 text-primary'
				: 'text-text-muted hover:bg-bg-subtle'}"
		>
			<span aria-hidden="true">📋</span>
			<span>{$t('sidebar.logs')}</span>
		</a>
	</nav>
</aside>
