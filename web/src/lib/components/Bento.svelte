<script lang="ts">
	import { untrack, type Snippet } from 'svelte';
	import { colsForWidth } from '$lib/columns';
	import type { Brick } from '$lib/types';

	let {
		items,
		brick,
		filler = false
	}: {
		items: Brick[];
		brick: Snippet<[Brick, boolean]>;
		// glaze wall: lay the whole grid on a muted field so every gap the dense
		// packing leaves — the holes between bricks and the seams around them —
		// reads as a solid muted filler block, grout between the pictures.
		filler?: boolean;
	} = $props();

	// A bento wall is a CSS grid, not a hand-packed set of columns. Every brick
	// keeps its natural height by spanning a whole number of thin row tracks;
	// feature bricks earn a second column. `grid-auto-flow: dense` then backfills
	// the holes those wide bricks leave behind, so the wall stays tight.
	const ROW = 4; // px per auto-row track; smaller packs bricks tighter
	const GAP = 12; // px; matches the gap-3 between bricks
	// the roughly-first-screen bricks load eagerly and at high priority; the
	// rest stay lazy so the wall's tail costs nothing until it is scrolled to
	const EAGER_BRICKS = 6;

	let container = $state<HTMLElement | null>(null);
	let cols = $state(1);

	// bricks that have already made their entrance; a re-measure or a column
	// change must not replay the drop-in animation on a brick already on the wall
	const entered = new Set<string>();
	let seenLen = 0;

	$effect(() => {
		if (!container) return;
		const observer = new ResizeObserver((entries) => {
			cols = colsForWidth(entries[0].contentRect.width);
		});
		observer.observe(container);
		return () => observer.disconnect();
	});

	// After each render, everything on the wall counts as entered. A shrink
	// (feed reset) forgets the old wall so the fresh one drops in again.
	$effect(() => {
		const len = items.length;
		untrack(() => {
			if (len < seenLen) entered.clear();
			for (const item of items) entered.add(item.id);
			seenLen = len;
		});
	});

	// A feature brick is one worth a wider footprint: video always (the richest
	// card), a blog or post that brought a landscape image along.
	function isFeature(item: Brick): boolean {
		if (item.kind === 'video') return true;
		if (item.kind === 'blog') return item.coverImage !== null;
		const img = item.images[0];
		return !!img?.aspectRatio && img.aspectRatio.width > img.aspectRatio.height;
	}

	// Never span past what the wall is wide enough to hold; a phone stays 1-up.
	function colSpan(item: Brick): number {
		if (cols <= 1) return 1;
		return isFeature(item) ? Math.min(2, cols) : 1;
	}

	// Measure the brick's natural height (offsetHeight ignores the entrance
	// transform) and reserve enough row tracks to hold it. Re-runs whenever the
	// content reflows: a wider column, a late-loading image.
	function autoRows(node: HTMLElement) {
		const content = node.firstElementChild as HTMLElement | null;
		if (!content) return;
		const apply = () => {
			const span = Math.max(1, Math.ceil((content.offsetHeight + GAP) / (ROW + GAP)));
			node.style.gridRowEnd = `span ${span}`;
		};
		const observer = new ResizeObserver(apply);
		observer.observe(content);
		return { destroy: () => observer.disconnect() };
	}
</script>

<!-- grid-auto-flow: row dense (below) backfills the holes wide bricks leave, so a
     later brick can be painted before an earlier one; the visual order is not the
     DOM order. DOM order stays the feed order on purpose, so tab and screen-reader
     order keep following the feed rather than the packed layout. -->
<div
	bind:this={container}
	class="grid items-start gap-3 {filler
		? 'rounded-3xl bg-ink/[0.11] p-3 ring-1 ring-ink/10 ring-inset dark:bg-chalk/[0.09] dark:ring-chalk/10'
		: ''}"
	style:grid-template-columns="repeat({cols}, minmax(0, 1fr))"
	style:grid-auto-rows="{ROW}px"
	style:grid-auto-flow="row dense"
>
	{#each items as item, i (item.id)}
		<div class="min-w-0" style:grid-column="span {colSpan(item)}" use:autoRows>
			<div class={entered.has(item.id) ? undefined : 'animate-brick-in'}>
				{@render brick(item, i < EAGER_BRICKS)}
			</div>
		</div>
	{/each}
</div>
