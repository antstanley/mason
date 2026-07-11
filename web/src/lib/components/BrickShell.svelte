<script lang="ts">
	import type { Snippet } from 'svelte';

	let {
		accent,
		href,
		children
	}: {
		accent: 'post' | 'blog' | 'video';
		href?: string;
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
	class="group overflow-hidden rounded-card border-2 bg-chalk shadow-brick transition-all duration-200 hover:-translate-y-1 hover:rotate-[0.6deg] hover:shadow-brick-lift dark:bg-kiln {accentClass}"
>
	{#if href}
		<a {href} target="_blank" rel="noopener noreferrer" class="block">
			{@render children()}
		</a>
	{:else}
		{@render children()}
	{/if}
</article>
