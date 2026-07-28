<script lang="ts">
	// One feed generator, as the picker lists it. NOT in components/cards/, which
	// is brick renderers: a feed is a wall you could lay, not a brick on one.
	//
	// It is a real link with a real href, in the same language as a brick's, so
	// it copies, opens in a tab and shares. Choosing it lays that feed and
	// remembers it in `mason:feeds`, which is what puts it in the picker's recent
	// row next time; the recents write happens on any activation the browser
	// reports, because a feed opened in a new tab is still a feed this reader
	// opened. That is why there are two listeners and not one: a middle click
	// dispatches auxclick and no click at all, so an onclick on its own remembered
	// nothing for the reader who opens feeds in background tabs. The rule for
	// which buttons count lives in `feeds.rememberFromLink`, once, because the
	// switcher panel's recents are the same link with the same promise.
	import { feeds, type FeedListing } from '$lib/state/feeds.svelte';
	import Icon from './Icon.svelte';

	let { feed }: { feed: FeedListing } = $props();

	// the wall this card stands for. Encoded, because an AT-URI carries `:` and
	// `/` and the picker must hand the address bar one parameter rather than a
	// path
	const href = $derived(`/?feed=${encodeURIComponent(feed.uri)}`);

	// Display names are not unique (two "Discover" feeds are ordinary), so the
	// accessible name carries the creator too: it is what tells a screen-reader
	// reader which of two identically named feeds they are about to lay.
	const label = $derived(feed.creator ? `${feed.name}, by @${feed.creator}` : feed.name);
</script>

<a
	{href}
	onclick={(event) => feeds.rememberFromLink(event, feed)}
	onauxclick={(event) => feeds.rememberFromLink(event, feed)}
	aria-label={label}
	class="group flex h-full min-h-11 w-full items-start gap-3 rounded-card border-2 border-ink/10 bg-chalk p-3 text-left shadow-brick transition-[transform,box-shadow,border-color] duration-200 hover:border-brick-post/60 hover:shadow-brick-lift motion-safe:hover:-translate-y-1 dark:border-chalk/10 dark:bg-kiln"
>
	{#if feed.avatar}
		<img src={feed.avatar} alt="" class="size-10 shrink-0 rounded-lg bg-brick-post/15 object-cover" />
	{:else}
		<!-- a generator published without an avatar still needs a face-sized
		     anchor, or its row of cards reads as a ragged list of text -->
		<span
			class="grid size-10 shrink-0 place-items-center rounded-lg bg-brick-post/15 text-lg font-bold uppercase"
			aria-hidden="true"
		>
			{feed.name.slice(0, 1)}
		</span>
	{/if}
	<span class="flex min-w-0 flex-1 flex-col gap-0.5">
		<span class="truncate font-display font-bold">{feed.name}</span>
		{#if feed.creator}
			<span class="truncate text-xs opacity-70">by @{feed.creator}</span>
		{/if}
		{#if feed.description}
			<!-- two lines, like a card's: a generator's description can be an essay
			     and this is a list of doors, not a page of prose -->
			<span class="line-clamp-2 text-sm leading-snug opacity-80">{feed.description}</span>
		{/if}
		{#if feed.likeCount > 0}
			<!-- hidden at zero, like every tally on the wall: a fresh feed showing a
			     0 reads as a verdict on it -->
			<span
				aria-label="{feed.likeCount} likes"
				class="mt-0.5 inline-flex items-center gap-1 text-xs opacity-70"
			>
				<Icon name="heart" class="size-3" />
				{feed.likeCount}
			</span>
		{/if}
	</span>
</a>
