<script lang="ts">
	import { untrack } from 'svelte';
	import { page } from '$app/state';
	import { feed } from '$lib/state/feed.svelte';
	import { layout } from '$lib/state/layout.svelte';
	import FeedGrid from '$lib/components/FeedGrid.svelte';
	import HandleForm from '$lib/components/HandleForm.svelte';

	// the URL is the source of truth: /?actor=handle; shareable walls
	const actor = $derived(page.url.searchParams.get('actor'));

	$effect(() => {
		const current = actor;
		// the glaze layout is also an algorithm: choosing it re-fetches an
		// images-only wall, the same way switching actor does. Tracking layout.id
		// makes the switch to (or from) glaze a fresh wall.
		const mode = layout.id === 'glaze' ? 'glaze' : undefined;
		// untrack: reset mutates feed state; tracking it would loop this effect
		if (current) untrack(() => feed.reset(current, mode));
	});
</script>

{#if actor}
	<main id="wall" class="pb-8">
		<h1 class="sr-only">@{actor}'s wall on mason</h1>
		<FeedGrid />
	</main>
{:else}
	<HandleForm />
{/if}
