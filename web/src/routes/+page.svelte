<script lang="ts">
	import { untrack } from 'svelte';
	import { handle } from '$lib/state/handle.svelte';
	import { feed } from '$lib/state/feed.svelte';
	import FeedGrid from '$lib/components/FeedGrid.svelte';
	import HandleForm from '$lib/components/HandleForm.svelte';

	$effect(() => {
		const actor = handle.current;
		// untrack: reset mutates feed state; tracking it would loop this effect
		if (actor) untrack(() => feed.reset(actor));
	});
</script>

{#if handle.current}
	<main class="pb-8">
		<FeedGrid />
	</main>
{:else}
	<HandleForm />
{/if}
