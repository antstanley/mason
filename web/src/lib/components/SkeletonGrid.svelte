<script lang="ts">
	// Skeleton columns measured exactly like the real wall, so the loading
	// preview never resolves into a different column count.
	import { colsForWidth } from '$lib/columns';
	import SkeletonCard from './cards/SkeletonCard.svelte';

	let { count = 12 }: { count?: number } = $props();

	let container = $state<HTMLElement | null>(null);
	let cols = $state(1);

	$effect(() => {
		if (!container) return;
		const observer = new ResizeObserver((entries) => {
			cols = colsForWidth(entries[0].contentRect.width);
		});
		observer.observe(container);
		return () => observer.disconnect();
	});

	const perColumn = $derived(Math.ceil(count / cols));
</script>

<div bind:this={container} class="flex items-start gap-5" aria-hidden="true">
	{#each { length: cols } as _, c (c)}
		<div class="flex min-w-0 flex-1 flex-col gap-5">
			{#each { length: perColumn } as _, r (r)}
				<SkeletonCard variant={c + r * cols} />
			{/each}
		</div>
	{/each}
</div>
