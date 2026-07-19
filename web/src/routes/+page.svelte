<script lang="ts">
	import { untrack } from 'svelte';
	import { page } from '$app/state';
	import { feed } from '$lib/state/feed.svelte';
	import { layout } from '$lib/state/layout.svelte';
	import FeedGrid from '$lib/components/FeedGrid.svelte';
	import HandleForm from '$lib/components/HandleForm.svelte';

	// the URL is the source of truth: /?actor=handle; shareable walls
	const actor = $derived(page.url.searchParams.get('actor'));

	// the glaze layout is also an algorithm: choosing it re-fetches an
	// images-only wall, the same way switching actor does. Only the glaze
	// transition changes this value, so bento <-> masonry no longer re-mixes;
	// $derived only propagates on a real change.
	const mode = $derived(layout.id === 'glaze' ? 'glaze' : undefined);

	$effect(() => {
		const current = actor;
		const currentMode = mode;
		// untrack: reset mutates feed state; tracking it would loop this effect
		if (current) untrack(() => feed.reset(current, currentMode));
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
