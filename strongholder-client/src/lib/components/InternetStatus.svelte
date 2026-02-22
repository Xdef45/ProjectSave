<script lang="ts">
	import { isOnline } from '$lib/stores/connection';
	import { slide } from 'svelte/transition';
	import { t } from '$lib/i18n';
</script>

{#if !$isOnline}
	<div
		role="alert"
		class="fixed right-0 bottom-0 left-0 z-50 bg-red-600 py-3 text-center font-bold text-white shadow-lg"
		transition:slide={{ duration: 300, axis: 'y' }}
	>
		<span>
			⚠ {$t('errors.no_internet')}
		</span>
	</div>
{/if}
