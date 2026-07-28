<script lang="ts">
	import type { Snippet } from 'svelte';
	import type { Brick } from '$lib/types';
	import { reader } from '$lib/state/reader.svelte';

	let {
		accent,
		href,
		brick,
		label,
		children
	}: {
		accent: 'post' | 'blog' | 'video';
		href?: string;
		// The brick this shell stands for, passed by whichever card also passes an
		// `href`, so the card-wide link can read the brick in place instead of
		// leaving the wall. Optional because two of the four cards hand this shell
		// no link at all (video and glaze own their own anchors), and a shell with
		// no link has nothing to intercept.
		brick?: Brick;
		// names the brick's <article> for screen readers, so every card carries a
		// consistent accessible name instead of only blogs carrying a heading
		label?: string;
		children: Snippet;
	} = $props();

	const accentClass = $derived(
		{
			post: 'border-brick-post/60 hover:border-brick-post',
			blog: 'border-brick-blog/60 hover:border-brick-blog',
			video: 'border-brick-video/60 hover:border-brick-video'
		}[accent]
	);
</script>

<article
	aria-label={label}
	class="group overflow-hidden rounded-card border-2 bg-chalk shadow-brick transition-[transform,box-shadow,border-color] duration-200 motion-safe:hover:-translate-y-1 motion-safe:hover:rotate-[0.6deg] hover:shadow-brick-lift dark:bg-kiln {accentClass}"
>
	{#if href}
		<!-- a real link, still: target/rel and the href itself are untouched, so
		     middle-click, cmd-click and "copy link address" all still reach the
		     source. Only a plain left click is taken, and only by `activate`,
		     which is the one place that rule lives. -->
		<a
			{href}
			target="_blank"
			rel="noopener noreferrer"
			onclick={(event) => {
				// no brick means no reader to open, so the click falls through to
				// the href exactly as it did before this existed
				if (brick) reader.activate(event, brick);
			}}
			class="block focus-visible:outline-offset-[-3px]"
		>
			{@render children()}
		</a>
	{:else}
		{@render children()}
	{/if}
</article>
