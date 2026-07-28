<script lang="ts">
	import { untrack } from 'svelte';
	import { page } from '$app/state';
	import type { FeedTarget } from '$lib/api';
	import { feed } from '$lib/state/feed.svelte';
	import { layout } from '$lib/state/layout.svelte';
	import { pull } from '$lib/state/pull.svelte';
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
	<!-- The wall itself moves with the pull, because that is what the gesture
	     says it does: the reader has hold of the wall, not of an indicator over
	     it. `main` and not the layout's wrapper, so the header stays put; on a
	     phone that wrapper holds the fixed control bar, and a transform on it
	     would drag the bar down the screen along with the bricks.
	     `pull.offset` and not `pull.distance`: on release the wall settles onto a
	     shelf and stays open for as long as the refresh it asked for runs, so the
	     indicator has a gap to sit in and the gap closing is what says the wall
	     is laid. One number, decided in the rune module, so the wall and the
	     indicator cannot disagree about where its top is.
	     The transition is off while a finger is driving (a transition mid-drag
	     is lag) and on for both glides, onto the shelf and home, with
	     `motion-safe` gating them so a reader who asked for less motion gets
	     neither. will-change only while a finger is on it: it promotes a long
	     wall to its own layer for the drag and hands the memory back after. -->
	<main
		id="wall"
		class="pb-8 {pull.pulling ? '' : 'motion-safe:transition-transform motion-safe:duration-200'}"
		style="transform: translateY({pull.offset}px); will-change: {pull.pulling
			? 'transform'
			: 'auto'};"
	>
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
