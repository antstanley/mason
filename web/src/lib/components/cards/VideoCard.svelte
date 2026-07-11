<script lang="ts">
	import type { VideoBrick } from '$lib/types';
	import BrickShell from '../BrickShell.svelte';
	import AuthorChip from '../AuthorChip.svelte';

	let { brick }: { brick: VideoBrick } = $props();

	// Poster + play button only — the <video> element is mounted on click
	// (VideoPlayer, M3). Videos never play on their own.
	let playRequested = $state(false);
</script>

<BrickShell accent="video">
	<div class="relative">
		{#if brick.poster}
			<img
				src={brick.poster}
				alt=""
				loading="lazy"
				class="w-full object-cover"
				style:aspect-ratio={brick.aspectRatio
					? `${brick.aspectRatio.width} / ${brick.aspectRatio.height}`
					: '16 / 9'}
			/>
		{:else}
			<div class="aspect-video w-full bg-brick-video/20"></div>
		{/if}
		{#if !playRequested}
			<button
				type="button"
				onclick={() => (playRequested = true)}
				class="absolute inset-0 grid cursor-pointer place-items-center"
				aria-label="Play video"
			>
				<span
					class="grid size-16 place-items-center rounded-full bg-brick-video pl-1 text-2xl text-white shadow-brick-lift transition-transform group-hover:scale-110"
				>
					▶
				</span>
			</button>
		{:else}
			<!-- M3 replaces this with the HLS VideoPlayer -->
			<div class="absolute inset-0 grid place-items-center bg-kiln/80 p-4 text-center">
				<p class="text-sm font-semibold text-chalk">player lands in M3 🧱</p>
			</div>
		{/if}
		<span
			class="absolute top-2 left-2 rounded-full bg-kiln/75 px-2.5 py-0.5 text-[0.7rem] font-bold text-chalk"
		>
			{brick.source === 'steam' ? '🎮 Steam' : '🦋 Bluesky'}
		</span>
	</div>
	<div class="flex flex-col gap-3 p-4">
		<p class="font-display leading-tight font-bold">{brick.title}</p>
		{#if brick.game}
			<p class="text-sm opacity-70">{brick.game.name}</p>
		{/if}
		{#if brick.author}
			<AuthorChip author={brick.author} />
		{/if}
	</div>
</BrickShell>
