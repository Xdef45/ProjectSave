<script lang="ts">
	import { onMount } from 'svelte';
	import { t } from '$lib/i18n';

	let isDark = false;

	onMount(() => {
		const storedTheme = localStorage.getItem('theme');
		const systemPrefersDark = window.matchMedia('(prefers-color-scheme: dark)').matches;

		if (storedTheme === 'dark' || (!storedTheme && systemPrefersDark)) {
			isDark = true;
			document.documentElement.classList.add('dark');
		} else {
			isDark = false;
			document.documentElement.classList.remove('dark');
		}
	});

	function toggleTheme() {
		isDark = !isDark;
		if (isDark) {
			document.documentElement.classList.add('dark');
			localStorage.setItem('theme', 'dark');
		} else {
			document.documentElement.classList.remove('dark');
			localStorage.setItem('theme', 'light');
		}
	}
</script>

<div class="fixed right-24 bottom-6 z-50 flex flex-col gap-4">
	<button
		class="main-fab group cursor-pointer"
		on:click={toggleTheme}
		aria-label={$t('theme.toggle')}
	>
		<div class="relative h-6 w-6">
			<svg
				class="absolute inset-0 h-full w-full transform transition-all duration-300
                {isDark ? 'scale-100 rotate-0 opacity-100' : 'scale-0 -rotate-90 opacity-0'} 
                text-yellow-400"
				fill="none"
				viewBox="0 0 24 24"
				stroke="currentColor"
				stroke-width="2"
			>
				<path
					stroke-linecap="round"
					stroke-linejoin="round"
					d="M12 3v1m0 16v1m9-9h-1M4 12H3m15.364 6.364l-.707-.707M6.343 6.343l-.707-.707m12.728 0l-.707.707M6.343 17.657l-.707.707M16 12a4 4 0 11-8 0 4 4 0 018 0z"
				/>
			</svg>

			<svg
				class="absolute inset-0 h-full w-full transform transition-all duration-300
                {isDark ? 'scale-0 rotate-90 opacity-0' : 'scale-100 rotate-0 opacity-100'} 
                text-primary"
				fill="none"
				viewBox="0 0 24 24"
				stroke="currentColor"
				stroke-width="2"
			>
				<path
					stroke-linecap="round"
					stroke-linejoin="round"
					d="M20.354 15.354A9 9 0 018.646 3.646 9.003 9.003 0 0012 21a9.003 9.003 0 008.354-5.646z"
				/>
			</svg>
		</div>
	</button>
</div>
