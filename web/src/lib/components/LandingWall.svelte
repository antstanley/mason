<script lang="ts">
	// The front door shows the product: a real (demo) wall, laid behind the
	// handle form. Not decoration; it's the one thing that explains mason
	// without a word of copy. Inert: aria-hidden, no pointer events, and it
	// simply never appears if the feed can't load.
	import { fetchFeed } from '$lib/api';
	import type { Brick } from '$lib/types';
	import Bento from './Bento.svelte';
	import PostCard from './cards/PostCard.svelte';
	import BlogCard from './cards/BlogCard.svelte';
	import VideoCard from './cards/VideoCard.svelte';

	let bricks = $state<Brick[]>([]);

	$effect(() => {
		void fetchFeed('demo')
			.then((page) => (bricks = page.items))
			.catch(() => {
				// no wall behind the form; the form still works
			});
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

{#if bricks.length}
	<div
		class="landing-wall pointer-events-none absolute inset-0 -z-10 overflow-hidden select-none"
		aria-hidden="true"
		inert
	>
		<div class="px-4 pt-4 sm:px-6">
			<Bento items={bricks} {brick} />
		</div>
	</div>
{/if}

<style>
	/* the wall recedes toward the form: legible as a product at the edges,
	   quiet enough to read type over in the middle */
	.landing-wall {
		opacity: 0.45;
		filter: saturate(0.9);
		mask-image: radial-gradient(
			ellipse 62% 46% at 50% 42%,
			transparent 30%,
			rgb(0 0 0 / 0.55) 62%,
			rgb(0 0 0 / 0.95) 100%
		);
		animation: wall-settle 0.8s ease-out both;
	}

	@keyframes wall-settle {
		from {
			opacity: 0;
		}
		to {
			opacity: 0.45;
		}
	}

	@media (prefers-reduced-motion: reduce) {
		.landing-wall {
			animation: none;
		}
	}
</style>
