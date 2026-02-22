<script lang="ts">
	import { page } from '$app/stores';
	import { toggleSidebar } from '$lib/stores/ui';
	import { t } from '$lib/i18n';
	import LanguageSwitcher from '$lib/components/LanguageSwitcher.svelte';

	let isProfileOpen = false;

	function toggleProfile(event: MouseEvent) {
		event.stopPropagation();
		isProfileOpen = !isProfileOpen;
	}

	function closeProfile() {
		isProfileOpen = false;
	}

	function handleKeydown(event: KeyboardEvent) {
		if (event.key === 'Escape' && isProfileOpen) {
			closeProfile();
		}
	}
</script>

<svelte:window on:click={closeProfile} on:keydown={handleKeydown} />

<header
	class="border-border bg-surface relative z-20 flex h-16 shrink-0 items-center justify-between border-b px-6 transition-colors duration-200"
>
	<button
		on:click={toggleSidebar}
		class="text-text-muted hover:bg-bg-subtle -ml-2 rounded-md p-2 transition-colors focus:outline-none"
		aria-label={$t('header.toggle_sidebar')}
	>
		<svg
			width="24"
			height="24"
			viewBox="0 0 24 24"
			fill="none"
			stroke="currentColor"
			stroke-width="2"
			stroke-linecap="round"
			stroke-linejoin="round"
		>
			<line x1="3" y1="12" x2="21" y2="12"></line>
			<line x1="3" y1="6" x2="21" y2="6"></line>
			<line x1="3" y1="18" x2="21" y2="18"></line>
		</svg>
	</button>

	<div class="flex items-center gap-4">
		<LanguageSwitcher />

		<a
			href="/settings"
			class="hover:text-primary text-text-muted hover:bg-bg-subtle rounded-full p-2 transition-colors {$page
				.url.pathname === '/settings'
				? 'text-primary bg-bg-subtle'
				: ''}"
			title={$t('header.settings')}
		>
			<svg
				width="20"
				height="20"
				viewBox="0 0 24 24"
				fill="none"
				stroke="currentColor"
				stroke-width="2"
				stroke-linecap="round"
				stroke-linejoin="round"
			>
				<circle cx="12" cy="12" r="3"></circle>
				<path
					d="M19.4 15a1.65 1.65 0 0 0 .33 1.82l.06.06a2 2 0 0 1 0 2.83 2 2 0 0 1-2.83 0l-.06-.06a1.65 1.65 0 0 0-1.82-.33 1.65 1.65 0 0 0-1 1.51V21a2 2 0 0 1-2 2 2 2 0 0 1-2-2v-.09A1.65 1.65 0 0 0 9 19.4a1.65 1.65 0 0 0-1.82.33l-.06.06a2 2 0 0 1-2.83 0 2 2 0 0 1 0-2.83l.06-.06a1.65 1.65 0 0 0 .33-1.82 1.65 1.65 0 0 0-1.51-1H3a2 2 0 0 1-2-2 2 2 0 0 1 2-2h.09A1.65 1.65 0 0 0 4.6 9a1.65 1.65 0 0 0-.33-1.82l-.06-.06a2 2 0 0 1 0-2.83 2 2 0 0 1 2.83 0l.06.06a1.65 1.65 0 0 0 1.82.33H9a1.65 1.65 0 0 0 1-1.51V3a2 2 0 0 1 2-2 2 2 0 0 1 2 2v.09a1.65 1.65 0 0 0 1 1.51 1.65 1.65 0 0 0 1.82-.33l.06-.06a2 2 0 0 1 2.83 0 2 2 0 0 1 0 2.83l-.06.06a1.65 1.65 0 0 0-.33 1.82V9a1.65 1.65 0 0 0 1.51 1H21a2 2 0 0 1 2 2 2 2 0 0 1-2 2h-.09a1.65 1.65 0 0 0-1.51 1z"
				></path>
			</svg>
		</a>

		<div class="border-border relative border-l pl-4">
			<button
				class="flex items-center gap-2 focus:outline-none"
				on:click={toggleProfile}
				aria-label={$t('header.profile_menu')}
				aria-expanded={isProfileOpen}
			>
				<div
					class="bg-primary/10 text-primary border-primary/20 hover:bg-primary/20 flex h-9 w-9 items-center justify-center rounded-full border font-bold transition-colors"
				>
					AU
				</div>
			</button>

			{#if isProfileOpen}
				<div
					class="animate-in fade-in slide-in-from-top-2 border-border bg-surface absolute top-full right-0 z-50 mt-2 w-56 rounded-lg border py-1 shadow-lg duration-200"
					role="menu"
				>
					<div class="border-border bg-bg/50 border-b px-4 py-2">
						<p class="text-text-main text-sm font-semibold">{$t('header.user_admin')}</p>
						<p class="text-text-muted text-xs">{$t('header.user_email_placeholder')}</p>
					</div>

					<a
						href="/profile"
						class="text-text-main hover:bg-bg block px-4 py-2 text-sm"
						on:click={closeProfile}
						role="menuitem"
					>
						{$t('header.my_profile')}
					</a>

					<div class="border-border my-1 border-t"></div>

					<a
						href="/login"
						class="block px-4 py-2 text-sm text-red-600 hover:bg-red-50 dark:hover:bg-red-900/20"
						on:click={closeProfile}
						role="menuitem"
					>
						{$t('header.logout')}
					</a>
				</div>
			{/if}
		</div>
	</div>
</header>
