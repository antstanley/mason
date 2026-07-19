<script lang="ts">
	import type { Snippet } from 'svelte';

	let {
		accent,
		href,
		label,
		children
	}: {
		accent: 'post' | 'blog' | 'video';
		href?: string;
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
		<a
			{href}
			target="_blank"
			rel="noopener noreferrer"
			class="block focus-visible:outline-offset-[-3px]"
		>
			{@render children()}
		</a>
	{:else}
		{@render children()}
	{/if}
</article>
