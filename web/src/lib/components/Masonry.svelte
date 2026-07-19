<script lang="ts">
	import { untrack, type Snippet } from 'svelte';
	import { colsForWidth } from '$lib/columns';
	import type { Brick } from '$lib/types';

	let {
		items,
		brick,
		warming = false
	}: {
		items: Brick[];
		brick: Snippet<[Brick, boolean]>;
		// while the wall warms the arrangement reflows under us, so each update is
		// re-placed from scratch rather than appended
		warming?: boolean;
	} = $props();

	// where a brick sits on the wall: its column and its offset down that column
	type Slot = { col: number; y: number };

	const GAP = 20; // px between bricks on both axes; the wall's old gap-5
	// the roughly-first-screen bricks load eagerly and at high priority; the
	// rest stay lazy so the wall's tail costs nothing until it is scrolled to
	const EAGER_BRICKS = 6;

	let container = $state<HTMLElement | null>(null);
	let colCount = $state(0);
	let colWidth = $state(0);
	let wallHeight = $state(0);
	// id -> slot, replaced wholesale by each layout pass; a brick with no slot
	// yet (not measured) stays hidden until its measurement lands
	let slots = $state<Record<string, Slot>>({});

	// measured wrapper heights, fed by one shared ResizeObserver; an entry lives
	// and dies with its wrapper node (see the measure action). Observer entries
	// arrive after the browser has laid out, so placement never forces a layout.
	const heights = new Map<string, number>();
	const idOf = new WeakMap<Element, string>();
	const observer = new ResizeObserver((entries) => {
		for (const entry of entries) {
			const id = idOf.get(entry.target);
			if (id === undefined) continue;
			heights.set(id, entry.borderBoxSize.at(0)?.blockSize ?? entry.contentRect.height);
		}
		if (reassignOnMeasure) {
			reassignOnMeasure = false;
			assigned.clear();
		}
		layout();
	});

	function measure(node: HTMLElement, id: string) {
		idOf.set(node, id);
		observer.observe(node);
		return {
			destroy() {
				observer.unobserve(node);
				heights.delete(id);
			}
		};
	}

	// each brick keeps the column it first landed in, so an append or a late
	// height change (a slow image, a webfont) shifts bricks down their own
	// column instead of reshuffling the wall; only a re-place clears it
	const assigned = new Map<string, number>();
	// a column-count change lays out at once with the pre-resize heights so the
	// wall moves immediately, then re-places when the remeasured heights land
	let reassignOnMeasure = false;

	// bricks that have already made their entrance; a re-place must not replay
	// the drop-in animation on a brick already on the wall
	const entered = new Set<string>();
	let seenLen = 0;

	function shortest(colHeights: number[]): number {
		let best = 0;
		for (let i = 1; i < colHeights.length; i++) {
			if (colHeights[i] < colHeights[best]) best = i;
		}
		return best;
	}

	// One numeric pass over the feed order: drop each measured brick into its
	// kept column (or the currently-shortest for a brick without one) and
	// accumulate the column heights. No DOM reads, one render.
	function layout() {
		if (colCount === 0) return;
		const colHeights = Array.from({ length: colCount }, () => 0);
		const next: Record<string, Slot> = {};
		for (const item of items) {
			const h = heights.get(item.id);
			// not measured yet (just appended): it stays hidden and slots in when
			// its observer entry lands, a frame later
			if (h === undefined) continue;
			let col = assigned.get(item.id);
			if (col === undefined || col >= colCount) {
				col = shortest(colHeights);
				assigned.set(item.id, col);
			}
			next[item.id] = { col, y: colHeights[col] };
			colHeights[col] += h + GAP;
		}
		slots = next;
		wallHeight = Math.max(0, Math.max(...colHeights) - GAP);
	}

	$effect(() => {
		if (!container) return;
		// layout() writes the container's height and this observer watches its
		// content box, so it fires again after every brick-height pass; bail
		// when the width is unchanged rather than lay the wall out twice
		let lastWidth = -1;
		const ro = new ResizeObserver((entries) => {
			const width = entries[0].contentRect.width;
			if (width === lastWidth) return;
			lastWidth = width;
			const n = colsForWidth(width);
			colWidth = (width - GAP * (n - 1)) / n;
			if (n !== colCount) {
				colCount = n;
				assigned.clear();
				reassignOnMeasure = true;
			}
			layout();
		});
		ro.observe(container);
		return () => ro.disconnect();
	});

	// warming is a distinct mode: the arrangement can reorder between updates,
	// so kept columns mean nothing. `wasWarming` catches the freeze: the feed
	// flips warming off in the same update that delivers the committed order
	// (feed.freeze sets both in one continuation), so that update is re-placed
	// from scratch here rather than misread as an append or a reset.
	let wasWarming = false;

	$effect(() => {
		// track items identity + length (endless-scroll appends and resets) and
		// the warming flag; untrack the placement work, which writes slots
		const list = items;
		const isWarming = warming;
		const len = list.length;
		untrack(() => {
			if (isWarming || wasWarming) {
				// warming, or the freeze that just ended it: re-place from scratch
				assigned.clear();
			} else if (len < seenLen) {
				// a reset to a fresh wall: it should drop in again
				entered.clear();
				assigned.clear();
			}
			layout();
			// after each render everything on the wall counts as entered; the
			// class was decided when the brick's node was created, so marking it
			// here only stops a later re-render replaying the entrance
			for (const item of list) entered.add(item.id);
			seenLen = len;
			wasWarming = isWarming;
		});
	});
</script>

<!-- One keyed list in feed order, placed into columns with transforms rather
     than split across per-column blocks: a brick's node survives re-places and
     column moves (in-card state and keyboard focus stay put), and DOM order
     stays the feed order on purpose, so tab and screen-reader order follow the
     feed rather than running column by column. -->
<div bind:this={container} class="relative" style:height="{wallHeight}px">
	{#if colCount > 0}
		{#each items as item, i (item.id)}
			{@const slot = slots[item.id]}
			<div
				use:measure={item.id}
				class="absolute top-0 left-0"
				class:invisible={!slot}
				style:width="{colWidth}px"
				style:transform={slot
					? `translate(${slot.col * (colWidth + GAP)}px, ${slot.y}px)`
					: undefined}
			>
				<div class={entered.has(item.id) ? undefined : 'animate-brick-in'}>
					{@render brick(item, i < EAGER_BRICKS)}
				</div>
			</div>
		{/each}
	{/if}
</div>
