<script lang="ts">
	import type { BlogBrick } from '$lib/types';
	import BrickShell from '../BrickShell.svelte';
	import AuthorChip from '../AuthorChip.svelte';

	// priority: an above-the-fold brick loads its cover eagerly and at high
	// fetch priority; the rest of the wall stays lazy
	let { brick, priority = false }: { brick: BlogBrick; priority?: boolean } = $props();

	const label = $derived(`blog post: ${brick.title}`);
</script>

<BrickShell accent="blog" href={brick.url} {label}>
	{#if brick.coverImage}
		<img
			src={brick.coverImage}
			alt=""
			loading={priority ? 'eager' : 'lazy'}
			fetchpriority={priority ? 'high' : undefined}
			class="aspect-[8/5] w-full bg-brick-blog/15 object-cover"
		/>
	{/if}
	<div class="flex flex-col gap-3 p-4">
		<span
			class="w-fit rounded-full bg-brick-blog/15 px-2.5 py-0.5 text-[0.7rem] font-bold tracking-wide text-brick-blog-ink uppercase dark:text-brick-blog"
		>
			{brick.publication.name}
		</span>
		<p class="font-display text-lg leading-tight font-bold">{brick.title}</p>
		{#if brick.description}
			<p class="line-clamp-3 text-sm leading-snug opacity-75">{brick.description}</p>
		{/if}
		{#if brick.tags.length}
			<div class="flex flex-wrap gap-1.5">
				{#each brick.tags.slice(0, 4) as tag (tag)}
					<span class="rounded-full bg-pop-lime/25 px-2 py-0.5 text-[0.68rem] font-semibold">#{tag}</span>
				{/each}
			</div>
		{/if}
		<AuthorChip author={brick.author} />
	</div>
</BrickShell>
