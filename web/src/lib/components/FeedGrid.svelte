<script lang="ts">
	import { feed } from '$lib/state/feed.svelte';
	import type { Brick } from '$lib/types';
	import Masonry from './Masonry.svelte';
	import PostCard from './cards/PostCard.svelte';
	import BlogCard from './cards/BlogCard.svelte';
	import VideoCard from './cards/VideoCard.svelte';
	import SkeletonCard from './cards/SkeletonCard.svelte';

	let sentinel = $state<HTMLElement | null>(null);

	$effect(() => {
		if (!sentinel) return;
		const observer = new IntersectionObserver(
			(entries) => {
				if (entries[0].isIntersecting) void feed.loadMore();
			},
			{ rootMargin: '1200px' }
		);
		observer.observe(sentinel);
		return () => observer.disconnect();
	});
</script>

{#snippet brick(item: Brick)}
	{#if item.kind === 'post'}
		<PostCard brick={item} />
	{:else if item.kind === 'blog'}
		<BlogCard brick={item} />
	{:else}
		<VideoCard brick={item} />
	{/if}
{/snippet}

{#if feed.initialLoad}
	<div class="grid grid-cols-1 gap-5 sm:grid-cols-2 lg:grid-cols-3 min-[1440px]:grid-cols-4">
		{#each { length: 12 } as _, i (i)}
			<SkeletonCard variant={i} />
		{/each}
	</div>
{:else if feed.error && feed.items.length === 0}
	<div class="mx-auto max-w-md py-24 text-center">
		<p class="text-5xl">🧱💥</p>
		<h2 class="font-display mt-4 text-2xl font-bold">
			{feed.error === 'handle-not-found' ? 'That handle isn’t on the wall' : 'The wall crumbled'}
		</h2>
		<p class="mt-2 opacity-70">
			{feed.error === 'handle-not-found'
				? 'Double-check the spelling and try again.'
				: 'Something went wrong fetching your feed. Give it another go.'}
		</p>
	</div>
{:else}
	<Masonry items={feed.items} {brick} />
	<div bind:this={sentinel} class="h-1"></div>
	{#if feed.loading}
		<div class="grid grid-cols-1 gap-5 pt-5 sm:grid-cols-2 lg:grid-cols-3 min-[1440px]:grid-cols-4">
			{#each { length: 4 } as _, i (i)}
				<SkeletonCard variant={i} />
			{/each}
		</div>
	{/if}
	{#if feed.done}
		<p class="py-16 text-center font-display text-lg font-bold opacity-60">
			🏁 you've reached the bottom of the internet
		</p>
	{/if}
{/if}
