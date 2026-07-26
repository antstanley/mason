<script lang="ts">
	import { untrack } from 'svelte';
	import { page } from '$app/state';
	import type { FeedTarget } from '$lib/api';
	import { feed } from '$lib/state/feed.svelte';
	import { layout } from '$lib/state/layout.svelte';
	import FeedGrid from '$lib/components/FeedGrid.svelte';
	import HandleForm from '$lib/components/HandleForm.svelte';

	// the URL is the source of truth, and it has two spellings: /?actor=handle
	// lays a follow graph, /?feed=<at-uri> lays a feed generator. Both are
	// shareable walls; everything else is a local preference. `feedUri` rather
	// than `feed` because `feed` in this file is the state singleton above.
	const actor = $derived(page.url.searchParams.get('actor'));
	const feedUri = $derived(page.url.searchParams.get('feed'));

	// exactly one wall, and `feed` wins when both parameters are in the URL:
	// the same precedence mortar applies, so the front and the engine can never
	// disagree about which wall a link names.
	const target: FeedTarget | null = $derived(feedUri ? { feed: feedUri } : actor ? { actor } : null);

	// the glaze layout is also an algorithm: choosing it re-fetches an
	// images-only wall, the same way switching wall does. Only the glaze
	// transition changes this value, so bento <-> masonry no longer re-mixes;
	// $derived only propagates on a real change.
	const mode = $derived(layout.id === 'glaze' ? 'glaze' : undefined);

	$effect(() => {
		const current = target;
		const currentMode = mode;
		// untrack: reset mutates feed state; tracking it would loop this effect
		if (current) untrack(() => feed.reset(current, currentMode));
	});
</script>

{#if actor || feedUri}
	<main id="wall" class="pb-8">
		<!-- when the wall cannot load, FeedGrid raises the failure as the page's
		     single h1, so the sr-only wall title steps aside to avoid a second one -->
		{#if !(feed.error && feed.items.length === 0)}
			<!-- branched on feedUri, not on `actor` being absent: with both in the
			     URL the feed is the wall being laid, so a heading that named the
			     actor would describe the wrong one. A feed generator's own name is
			     not on the wire, so this says what the reader is looking at without
			     claiming to name it. -->
			{#if feedUri}
				<h1 class="sr-only">a bluesky feed, laid on mason</h1>
			{:else}
				<h1 class="sr-only">@{actor}'s wall on mason</h1>
			{/if}
		{/if}
		<FeedGrid />
	</main>
{:else}
	<HandleForm />
{/if}
