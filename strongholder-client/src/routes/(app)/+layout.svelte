<script lang="ts">
	import '../../app.css';
	import { page } from '$app/stores';
	import { fly, fade } from 'svelte/transition';
	import { cubicOut } from 'svelte/easing';
	import { onMount, onDestroy } from 'svelte';

	import Sidebar from '$lib/components/Sidebar.svelte';
	import Header from '$lib/components/Header.svelte';

	import { initBackupListener } from '$lib/stores/backup';

	let isProfileOpen = false;

	function closeProfile() {
		isProfileOpen = false;
	}

	let unlistenBackup: (() => void) | void;

	onMount(() => {
		const setupListener = async () => {
			try {
				unlistenBackup = await initBackupListener();
			} catch (e) {
				console.error('Failed to initialize backup listener:', e);
			}
		};

		setupListener();
	});

	onDestroy(() => {
		if (typeof unlistenBackup === 'function') {
			unlistenBackup();
		}
	});

	const animIn = { y: 10, duration: 300, delay: 100, easing: cubicOut, opacity: 0 };
	const animOut = { duration: 200, easing: cubicOut };
</script>

<svelte:window on:click={closeProfile} />

<div
	class="bg-bg text-text-main flex h-screen w-screen overflow-hidden transition-colors duration-300"
>
	<div class="relative z-30 h-full shrink-0">
		<Sidebar />
	</div>

	<div class="flex h-full min-w-0 flex-1 flex-col overflow-hidden">
		<div class="bg-surface border-border relative z-20 shrink-0 border-b">
			<Header />
		</div>

		<main class="bg-bg relative grid flex-1 grid-cols-1 grid-rows-1 overflow-hidden">
			<div
				class="col-start-1 row-start-1 h-full w-full overflow-y-auto p-8"
				in:fly={animIn}
				out:fade={animOut}
			>
				<slot />
			</div>
		</main>
	</div>
</div>
