<script lang="ts">
	import type { VideoBrick } from '$lib/types';
	import { player } from '$lib/state/player.svelte';
	import BrickShell from '../BrickShell.svelte';
	import AuthorChip from '../AuthorChip.svelte';
	import VideoPlayer from '../VideoPlayer.svelte';

	let { brick }: { brick: VideoBrick } = $props();

	// The <video> element does not exist until the user clicks — the Wall
	// never plays anything on its own.
	let playRequested = $state(false);

	const ratio = $derived(
		brick.aspectRatio ? `${brick.aspectRatio.width} / ${brick.aspectRatio.height}` : '16 / 9'
	);
	const sourceName = $derived(brick.source === 'steam' ? 'Steam' : 'Bluesky');

	// if another card starts playing, collapse this one back to its poster
	$effect(() => {
		if (playRequested && player.activeId !== brick.id) playRequested = false;
	});
</script>

<BrickShell accent="video">
	<div class="relative">
		{#if playRequested}
			<VideoPlayer id={brick.id} playlist={brick.playlist} poster={brick.poster} aspectRatio={ratio} />
			<button
				type="button"
				onclick={() => (playRequested = false)}
				aria-label="Close video"
				class="absolute top-2 right-2 grid size-11 cursor-pointer place-items-center rounded-full bg-kiln/75 text-lg font-bold text-chalk transition-colors hover:bg-kiln"
			>
				✕
			</button>
		{:else}
			{#if brick.poster}
				<img src={brick.poster} alt="" loading="lazy" class="w-full object-cover" style:aspect-ratio={ratio} />
			{:else}
				<div class="w-full bg-brick-video/20" style:aspect-ratio={ratio}></div>
			{/if}
			<button
				type="button"
				onclick={() => {
					// claim synchronously so the collapse effect below never
					// sees this card as a loser of its own click
					player.claim(brick.id);
					playRequested = true;
				}}
				class="absolute inset-0 grid cursor-pointer place-items-center"
				aria-label="Play video: {brick.title || sourceName + ' video'}"
			>
				<span
					class="grid size-16 place-items-center rounded-full bg-brick-video pl-1 text-2xl text-white shadow-brick-lift transition-transform motion-safe:group-hover:scale-110"
					aria-hidden="true"
				>
					▶
				</span>
			</button>
			<span
				class="pointer-events-none absolute top-2 left-2 rounded-full bg-kiln/75 px-2.5 py-0.5 text-[0.7rem] font-bold text-chalk"
			>
				{brick.source === 'steam' ? '🎮 Steam' : '🦋 Bluesky'}
			</span>
		{/if}
	</div>
	<div class="flex flex-col gap-3 p-4">
		{#if brick.title}
			<p class="font-display leading-tight font-bold">{brick.title}</p>
		{/if}
		{#if brick.game}
			<p class="text-sm opacity-75">{brick.game.name}</p>
		{/if}
		<div class="flex items-center justify-between gap-2">
			{#if brick.author}
				<AuthorChip author={brick.author} />
			{:else}
				<span></span>
			{/if}
			<a
				href={brick.url}
				target="_blank"
				rel="noopener noreferrer"
				class="shrink-0 text-sm font-semibold text-brick-video-ink hover:underline dark:text-brick-video-bright"
			>
				watch on {sourceName} ↗
			</a>
		</div>
	</div>
</BrickShell>
