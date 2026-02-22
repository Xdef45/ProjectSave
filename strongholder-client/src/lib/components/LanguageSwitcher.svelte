<script lang="ts">
	import { locale } from '$lib/i18n';
	import { fly } from 'svelte/transition';
	import { clickOutside } from '$lib/utils/clickOutside';

	// --- TYPES ---
	interface Language {
		id: string;
		flag: string;
		label: string;
	}

	// --- STATE ---
	let isOpen = false;

	const languages: Language[] = [
		{ id: 'fr', flag: 'fr', label: 'Français' },
		{ id: 'en', flag: 'gb', label: 'English' },
		{ id: 'es', flag: 'es', label: 'Español' },
		{ id: 'de', flag: 'de', label: 'Deutsch' },
		{ id: 'it', flag: 'it', label: 'Italiano' }
	];

	$: currentLang = languages.find((l) => l.id === $locale) || languages[0];

	// --- HANDLERS ---

	function closeMenu() {
		isOpen = false;
	}

	function toggleMenu() {
		isOpen = !isOpen;
	}

	function selectLanguage(langId: string) {
		$locale = langId;
		closeMenu();
	}
</script>

<div class="lang-container" use:clickOutside={closeMenu}>
	{#if isOpen}
		<div class="menu-stack" transition:fly={{ y: 20, duration: 200 }}>
			{#each languages as lang (lang.id)}
				{#if lang.id !== $locale}
					<button
						class="lang-btn group"
						on:click={() => selectLanguage(lang.id)}
						aria-label={`Switch to ${lang.label}`}
					>
						<span class="tooltip">{lang.label}</span>
						<div class="circle small">
							<img src="https://flagcdn.com/w80/{lang.flag}.png" alt={lang.label} loading="lazy" />
						</div>
					</button>
				{/if}
			{/each}
		</div>
	{/if}

	<button
		class="circle main {isOpen ? 'open' : ''}"
		on:click={toggleMenu}
		aria-haspopup="true"
		aria-expanded={isOpen}
		aria-label="Change Language"
	>
		<img src="https://flagcdn.com/w80/{currentLang.flag}.png" alt={currentLang.label} />
	</button>
</div>

<style>
	.lang-container {
		position: fixed;
		bottom: 24px;
		right: 24px;
		z-index: 50;
		display: flex;
		flex-direction: column;
		align-items: flex-end;
		gap: 12px;
	}

	.circle {
		border-radius: 9999px;
		border: 3px solid white;
		box-shadow: 0 10px 15px -3px rgba(0, 0, 0, 0.1);
		overflow: hidden;
		background: white;
		transition: all 0.2s ease-in-out;
		cursor: pointer;
		padding: 0;
	}

	.main {
		width: 56px;
		height: 56px;
	}

	.main:hover {
		transform: scale(1.1);
	}

	.main.open {
		opacity: 0.7;
		transform: scale(0.9);
	}

	.small {
		width: 40px;
		height: 40px;
		border-width: 2px;
	}

	.menu-stack {
		display: flex;
		flex-direction: column;
		gap: 8px;
		align-items: flex-end;
	}

	.lang-btn {
		display: flex;
		align-items: center;
		gap: 12px;
		background: none;
		border: none;
		padding: 0;
		cursor: pointer;
	}

	.tooltip {
		background: #1e293b;
		color: white;
		font-size: 10px;
		font-weight: bold;
		padding: 4px 8px;
		border-radius: 4px;
		text-transform: uppercase;
		opacity: 0;
		transform: translateX(10px);
		transition: all 0.2s;
		pointer-events: none;
		white-space: nowrap;
	}

	:global(.lang-btn:hover) .tooltip {
		opacity: 1;
		transform: translateX(0);
	}

	img {
		width: 100%;
		height: 100%;
		aspect-ratio: 1/1;
		display: block;
	}
</style>
