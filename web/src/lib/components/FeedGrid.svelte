<script lang="ts">
	import { goto } from '$app/navigation';
	import { page } from '$app/state';
	import { feed } from '$lib/state/feed.svelte';
	import { cleanHandle, lastHandle } from '$lib/state/handle.svelte';
	import type { Brick } from '$lib/types';
	import Masonry from './Masonry.svelte';
	import PostCard from './cards/PostCard.svelte';
	import BlogCard from './cards/BlogCard.svelte';
	import VideoCard from './cards/VideoCard.svelte';
	import SkeletonGrid from './SkeletonGrid.svelte';

	let sentinel = $state<HTMLElement | null>(null);
	let retryInput = $state<HTMLInputElement | null>(null);
	let retryValue = $state('');

	const currentActor = $derived(page.url.searchParams.get('actor') ?? '');

	// the dead-end fix: the failed handle stays editable, right here
	$effect(() => {
		if (feed.error === 'handle-not-found') {
			retryValue = currentActor;
			retryInput?.focus();
			retryInput?.select();
		}
	});

	function retrySubmit(event: SubmitEvent) {
		event.preventDefault();
		const handle = cleanHandle(retryValue);
		if (!handle) return;
		lastHandle.remember(handle);
		if (handle === currentActor) {
			// same handle, fresh attempt — URL wouldn't change, reset directly
			feed.reset(handle);
		} else {
			void goto(`/?actor=${encodeURIComponent(handle)}`);
		}
	}

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
	<SkeletonGrid count={12} />
{:else if feed.error && feed.items.length === 0}
	<div class="mx-auto max-w-md py-20 text-center">
		<p class="text-5xl" aria-hidden="true">🧱💥</p>
		<h1 class="font-display mt-4 text-2xl font-bold">
			{feed.error === 'handle-not-found' ? 'That handle isn’t on the wall' : 'The wall crumbled'}
		</h1>
		<p class="mt-2 opacity-75">
			{feed.error === 'handle-not-found'
				? 'Check the spelling — handles look like name.bsky.social — or try another one:'
				: 'Something went wrong fetching this feed.'}
		</p>
		{#if feed.error === 'handle-not-found'}
			<form onsubmit={retrySubmit} class="mt-6 flex gap-2">
				<label class="sr-only" for="retry-handle">Bluesky handle</label>
				<input
					id="retry-handle"
					bind:this={retryInput}
					bind:value={retryValue}
					type="text"
					autocapitalize="none"
					autocorrect="off"
					spellcheck="false"
					class="min-w-0 flex-1 rounded-full border-2 border-ink/20 bg-chalk px-5 py-3 font-semibold transition-colors dark:border-chalk/20 dark:bg-kiln"
				/>
				<button
					type="submit"
					class="shrink-0 cursor-pointer rounded-full bg-pop-pink-deep px-5 py-3 font-display font-bold text-white shadow-brick transition-transform motion-safe:hover:scale-105 motion-safe:active:scale-95"
				>
					retry
				</button>
			</form>
		{:else}
			<button
				type="button"
				onclick={() => feed.reset(currentActor)}
				class="mt-6 cursor-pointer rounded-full bg-pop-pink-deep px-6 py-3 font-display font-bold text-white shadow-brick transition-transform motion-safe:hover:scale-105 motion-safe:active:scale-95"
			>
				try again
			</button>
		{/if}
	</div>
{:else}
	<Masonry items={feed.items} {brick} />
	<div bind:this={sentinel} class="h-1"></div>
	{#if feed.error && feed.items.length > 0}
		<div class="flex justify-center py-10">
			<button
				type="button"
				onclick={() => void feed.loadMore()}
				class="cursor-pointer rounded-full border-2 border-brick-blog/60 bg-chalk px-6 py-3 font-display font-bold shadow-brick transition-transform motion-safe:hover:scale-105 motion-safe:active:scale-95 dark:bg-kiln"
			>
				couldn’t lay more bricks — tap to retry
			</button>
		</div>
	{:else if feed.loading}
		<div class="grid grid-cols-1 gap-5 pt-5 sm:grid-cols-2 lg:grid-cols-3 min-[1440px]:grid-cols-4">
			{#each { length: 4 } as _, i (i)}
				<SkeletonCard variant={i} />
			{/each}
		</div>
	{/if}
	{#if feed.done && !feed.error}
		<p class="py-16 text-center font-display text-lg font-bold opacity-70">
			🏁 you've reached the bottom of the internet
		</p>
	{/if}
{/if}
