<script lang="ts">
	import type { VideoBrick } from '$lib/types';
	import { player } from '$lib/state/player.svelte';
	import { clientUrl } from '$lib/state/client.svelte';
	import BrickShell from '../BrickShell.svelte';
	import AuthorChip from '../AuthorChip.svelte';
	import VideoPlayer from '../VideoPlayer.svelte';

	let { brick }: { brick: VideoBrick } = $props();

	// The <video> element does not exist until the user clicks; the Wall
	// never plays anything on its own. That holds for live streams too: a
	// wall of talking heads is nobody's idea of a good time.
	let playRequested = $state(false);

	const ratio = $derived(
		brick.aspectRatio ? `${brick.aspectRatio.width} / ${brick.aspectRatio.height}` : '16 / 9'
	);
	const sourceName = $derived(brick.source === 'streamplace' ? 'Streamplace' : 'Bluesky');

	// Hours and minutes; a stream runs long enough that seconds are noise.
	// Not every archived video is a long one though: clips of a few seconds
	// exist, and rounding those to "0m" makes the card look broken.
	function runtime(ms: number): string {
		const seconds = Math.round(ms / 1000);
		if (seconds < 60) return `${seconds}s`;
		const minutes = Math.round(seconds / 60);
		const hours = Math.floor(minutes / 60);
		return hours > 0 ? `${hours}h ${minutes % 60}m` : `${minutes}m`;
	}

	const viewers = $derived(
		brick.viewerCount === 1 ? '1 watching' : `${brick.viewerCount} watching`
	);

	// if another card starts playing, collapse this one back to its poster
	$effect(() => {
		if (playRequested && player.activeId !== brick.id) playRequested = false;
	});
</script>

<BrickShell accent="video">
	<div class="relative">
		{#if playRequested}
			<VideoPlayer
				id={brick.id}
				playlist={brick.playlist}
				poster={brick.poster}
				aspectRatio={ratio}
				live={brick.live}
			/>
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
				<img
					src={brick.poster}
					alt=""
					loading="lazy"
					class="w-full bg-brick-video/15 object-cover"
					style:aspect-ratio={ratio}
				/>
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
				class="absolute inset-0 grid cursor-pointer place-items-center focus-visible:outline-offset-[-3px]"
				aria-label={brick.live
					? `Watch live: ${brick.title || sourceName + ' stream'}`
					: `Play video: ${brick.title || sourceName + ' video'}`}
			>
				<span
					class="grid size-16 place-items-center rounded-full pl-1 text-2xl text-white shadow-brick-lift transition-transform motion-safe:group-hover:scale-110 {brick.live
						? 'bg-live'
						: 'bg-brick-video'}"
					aria-hidden="true"
				>
					▶
				</span>
			</button>
			{#if brick.live}
				<span
					class="pointer-events-none absolute top-2 left-2 flex items-center gap-1.5 rounded-full bg-live px-2.5 py-0.5 text-[0.7rem] font-bold tracking-wide text-white uppercase"
				>
					<span class="size-1.5 rounded-full bg-white motion-safe:animate-pulse" aria-hidden="true"
					></span>
					live
				</span>
			{:else}
				<span
					class="pointer-events-none absolute top-2 left-2 rounded-full bg-kiln/75 px-2.5 py-0.5 text-[0.7rem] font-bold text-chalk"
				>
					{brick.source === 'streamplace' ? '📺 Streamplace' : '🦋 Bluesky'}
				</span>
			{/if}
			{#if brick.durationMs}
				<span
					class="pointer-events-none absolute right-2 bottom-2 rounded-full bg-kiln/75 px-2 py-0.5 text-[0.7rem] font-bold text-chalk tabular-nums"
				>
					{runtime(brick.durationMs)}
				</span>
			{/if}
		{/if}
	</div>
	<div class="flex flex-col gap-3 p-4">
		{#if brick.title}
			<p class="font-display leading-tight font-bold">{brick.title}</p>
		{/if}
		{#if brick.activity || (brick.live && brick.viewerCount !== null)}
			<p class="flex flex-wrap items-center gap-x-2 text-sm opacity-75">
				{#if brick.live && brick.viewerCount !== null}
					<span class="font-semibold text-live dark:text-live-bright">{viewers}</span>
				{/if}
				{#if brick.activity}
					<span>{brick.activity}</span>
				{/if}
			</p>
		{/if}
		<div class="flex flex-wrap items-center justify-between gap-x-2 gap-y-2">
			<AuthorChip author={brick.author} />
			<a
				href={clientUrl(brick.url)}
				target="_blank"
				rel="noopener noreferrer"
				class="shrink-0 text-sm font-semibold text-brick-video-ink hover:underline dark:text-brick-video-bright"
			>
				{brick.live ? 'watch live' : 'watch'} on {sourceName} ↗
			</a>
		</div>
	</div>
</BrickShell>
