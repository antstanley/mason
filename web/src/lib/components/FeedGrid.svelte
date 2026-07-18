<script lang="ts">
	import { tick } from 'svelte';
	import { goto } from '$app/navigation';
	import { page } from '$app/state';
	import { feed } from '$lib/state/feed.svelte';
	import { cleanHandle, lastHandle } from '$lib/state/handle.svelte';
	import type { Brick } from '$lib/types';
	import { layout } from '$lib/state/layout.svelte';
	import Bento from './Bento.svelte';
	import Masonry from './Masonry.svelte';
	import PostCard from './cards/PostCard.svelte';
	import GlazeCard from './cards/GlazeCard.svelte';
	import BlogCard from './cards/BlogCard.svelte';
	import VideoCard from './cards/VideoCard.svelte';
	import SkeletonGrid from './SkeletonGrid.svelte';

	// glaze is a layout AND the images-only algorithm; on the retry paths, keep
	// whichever wall the reader is on
	const isGlaze = $derived(layout.id === 'glaze');
	const currentMode = $derived(isGlaze ? 'glaze' : undefined);

	let sentinel = $state<HTMLElement | null>(null);
	let retryInput = $state<HTMLInputElement | null>(null);
	let retryValue = $state('');

	const currentActor = $derived(page.url.searchParams.get('actor') ?? '');

	// the dead-end fix: a wall you cannot see stays a door to another one. Both
	// the mistyped handle and the sealed wall drop the reader into the handle
	// box; the typo keeps its text to fix, the sealed wall clears it (there is
	// nothing to correct, only somewhere else to go).
	$effect(() => {
		if (feed.error === 'handle-not-found') {
			retryValue = currentActor;
		} else if (feed.error === 'login-required') {
			retryValue = '';
		} else {
			return;
		}
		// select() only works once Svelte has written the value to the DOM
		void tick().then(() => {
			retryInput?.focus();
			retryInput?.select();
		});
	});

	function retrySubmit(event: SubmitEvent) {
		event.preventDefault();
		const handle = cleanHandle(retryValue);
		if (!handle) return;
		lastHandle.remember(handle);
		if (handle === currentActor) {
			// same handle, fresh attempt; URL wouldn't change, reset directly
			feed.reset(handle, currentMode);
		} else {
			void goto(`/?actor=${encodeURIComponent(handle)}`);
		}
	}

	// How far below the fold we start laying the next bricks. The reader should
	// meet a wall that is already there, not a spinner.
	const PREFETCH_MARGIN = 1200;

	function withinReach() {
		if (!sentinel) return false;
		return sentinel.getBoundingClientRect().top < window.innerHeight + PREFETCH_MARGIN;
	}

	let pumping = $state(false);

	/** Keep laying bricks while the bottom of the wall is still within reach.
	 *
	 *  A plain observer callback is not enough, and this is the bug that used to
	 *  strand the wall: IntersectionObserver fires on a CHANGE of state, and a
	 *  page that comes back short (mortar serves what it has rather than make
	 *  you wait for a full one) does not grow the wall enough to push the
	 *  sentinel back out of the margin. It stays intersecting, so no second
	 *  event ever arrives, and the wall stops for good with a cursor still in
	 *  its hand. So we pull, rather than wait to be told. */
	async function pump() {
		if (pumping) return;
		pumping = true;
		try {
			let stalls = 0;
			while (!feed.done && withinReach()) {
				const before = feed.items.length;
				await feed.loadMore();
				await tick();
				if (feed.items.length > before) {
					stalls = 0;
					continue;
				}
				// a page that added nothing: the snapshot is still warming, or it
				// failed. Back off a little, then a little more, and let the next
				// scroll try again rather than spin on the spot.
				if (feed.error || ++stalls > 3) break;
				await new Promise((resume) => setTimeout(resume, 400 * stalls));
			}
		} finally {
			pumping = false;
		}
	}

	$effect(() => {
		if (!sentinel) return;
		const observer = new IntersectionObserver(
			(entries) => {
				if (entries[0].isIntersecting) void pump();
			},
			{ rootMargin: `${PREFETCH_MARGIN}px` }
		);
		observer.observe(sentinel);
		return () => observer.disconnect();
	});
</script>

{#snippet brick(item: Brick, priority: boolean)}
	{#if item.kind === 'post'}
		{#if isGlaze}
			<GlazeCard brick={item} {priority} />
		{:else}
			<PostCard brick={item} {priority} />
		{/if}
	{:else if item.kind === 'blog'}
		<BlogCard brick={item} {priority} />
	{:else}
		<VideoCard brick={item} {priority} />
	{/if}
{/snippet}

{#if feed.initialLoad}
	<SkeletonGrid count={12} />
{:else if feed.error && feed.items.length === 0}
	{@const sealed = feed.error === 'login-required'}
	{@const notFound = feed.error === 'handle-not-found'}
	<div class="mx-auto max-w-md py-20 text-center">
		<p class="text-5xl" aria-hidden="true">{sealed ? '🧱🔒' : '🧱💥'}</p>
		<h1 class="font-display mt-4 text-2xl font-bold">
			{#if notFound}no wall for that handle{:else if sealed}this wall is sealed{:else}the wall wouldn't
				load{/if}
		</h1>
		<p class="mt-2 opacity-75">
			{#if notFound}handles look like name.bsky.social. check the spelling, or try someone else:{:else if sealed}this
				waller asked to be seen by signed-in visitors only. mason reads walls logged out, so this
				one stays bricked up. try another wall:{:else}mason could not reach the network. check your
				connection and try again.{/if}
		</p>
		{#if notFound || sealed}
			<form onsubmit={retrySubmit} class="mt-6 flex gap-2">
				<label class="sr-only" for="retry-handle">Your Bluesky handle</label>
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
				onclick={() => feed.reset(currentActor, currentMode)}
				class="mt-6 cursor-pointer rounded-full bg-pop-pink-deep px-6 py-3 font-display font-bold text-white shadow-brick transition-transform motion-safe:hover:scale-105 motion-safe:active:scale-95"
			>
				try again
			</button>
		{/if}
		<p class="mt-6 text-sm">
			<a
				href="/?actor=demo"
				class="inline-flex min-h-11 items-center px-2 font-semibold text-brick-post-ink hover:underline dark:text-brick-post"
			>
				or wander the demo wall
			</a>
		</p>
	</div>
{:else}
	{#if layout.id === 'masonry'}
		<Masonry items={feed.items} {brick} />
	{:else if layout.id === 'glaze'}
		<Bento items={feed.items} {brick} filler />
	{:else}
		<Bento items={feed.items} {brick} />
	{/if}
	<div bind:this={sentinel} class="h-1"></div>
	{#if feed.error && feed.items.length > 0}
		<div class="flex justify-center py-10">
			<button
				type="button"
				onclick={() => void pump()}
				class="cursor-pointer rounded-full border-2 border-brick-blog/60 bg-chalk px-6 py-3 font-display font-bold shadow-brick transition-transform motion-safe:hover:scale-105 motion-safe:active:scale-95 dark:bg-kiln"
			>
				more bricks did not arrive. tap to retry
			</button>
		</div>
	{:else if feed.loading || pumping}
		<!-- pumping, not just loading: between attempts the pump is briefly idle
		     while the snapshot warms, and letting the skeletons blink out would
		     read as a wall that had given up rather than one still being laid -->
		<div class="pt-5">
			<SkeletonGrid count={4} />
		</div>
	{/if}
	{#if feed.done && !feed.error}
		<p class="py-16 text-center font-display text-lg font-bold opacity-70">
			that is every brick. the wall is finished.
		</p>
	{/if}
{/if}
