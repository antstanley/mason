<script lang="ts">
	import type { PostBrick } from '$lib/types';
	import BrickShell from '../BrickShell.svelte';
	import AuthorChip from '../AuthorChip.svelte';

	let { brick }: { brick: PostBrick } = $props();

	const img = $derived(brick.images[0] ?? null);
</script>

<BrickShell accent="post" href={brick.url}>
	{#if img}
		<img
			src={img.src}
			alt={img.alt}
			loading="lazy"
			class="w-full object-cover"
			style:aspect-ratio={img.aspectRatio ? `${img.aspectRatio.width} / ${img.aspectRatio.height}` : undefined}
		/>
	{/if}
	<div class="flex flex-col gap-3 p-4">
		{#if brick.text}
			<p class="text-[0.95rem] leading-snug">{brick.text}</p>
		{/if}
		{#if brick.external}
			<div class="rounded-xl border border-ink/10 bg-plaster-deep/50 p-3">
				<p class="truncate text-sm font-semibold">{brick.external.title}</p>
				<p class="line-clamp-2 text-xs opacity-70">{brick.external.description}</p>
			</div>
		{/if}
		<div class="flex items-center justify-between gap-2">
			<AuthorChip author={brick.author} />
			<div class="flex shrink-0 gap-2 text-xs font-semibold opacity-60">
				<span>♥ {brick.likeCount}</span>
				<span>↻ {brick.repostCount}</span>
			</div>
		</div>
	</div>
</BrickShell>
