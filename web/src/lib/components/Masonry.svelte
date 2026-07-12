<script lang="ts">
	import { tick, untrack, type Snippet } from 'svelte';
	import { colsForWidth } from '$lib/columns';
	import type { Brick } from '$lib/types';

	let {
		items,
		brick
	}: {
		items: Brick[];
		brick: Snippet<[Brick]>;
	} = $props();

	type Placed = { item: Brick; fresh: boolean };

	let container = $state<HTMLElement | null>(null);
	let colCount = $state(0);
	let columns = $state<Placed[][]>([]);
	let placedCount = 0;
	let placing = false;
	// bricks that have already made their entrance; a column-count rebuild
	// re-places them WITHOUT replaying the drop-in animation
	const entered = new Set<string>();

	// Measure columns through the live DOM, not element bindings: the keyed
	// each reuses surviving column divs across rebuilds, so bind:this refs
	// captured in an array go permanently stale after a column-count change
	// (the bug where the wall never left single-column mode).
	function shortestColumn(): number {
		let best = 0;
		let bestHeight = Infinity;
		for (let i = 0; i < colCount; i++) {
			const el = container?.children[i] as HTMLElement | undefined;
			const h = el?.offsetHeight ?? 0;
			if (h < bestHeight) {
				bestHeight = h;
				best = i;
			}
		}
		return best;
	}

	// Place items one at a time into the currently-shortest column, awaiting a
	// tick between placements so offsetHeight reflects the previous placement.
	// Appends never reshuffle existing bricks.
	async function placePending() {
		if (placing || colCount === 0) return;
		placing = true;
		try {
			while (placedCount < items.length) {
				const item = items[placedCount];
				placedCount += 1;
				const fresh = !entered.has(item.id);
				entered.add(item.id);
				columns[shortestColumn()].push({ item, fresh });
				// placement must be sequential: each brick lands in the column
				// whose height reflects the previous brick
				// oxlint-disable-next-line no-await-in-loop
				await tick();
			}
		} finally {
			placing = false;
		}
		// items may have grown while the flag flipped
		if (placedCount < items.length) void placePending();
	}

	function rebuild(n: number) {
		colCount = n;
		columns = Array.from({ length: n }, () => []);
		placedCount = 0;
		void tick().then(() => void placePending());
	}

	$effect(() => {
		if (!container) return;
		const observer = new ResizeObserver((entries) => {
			const n = colsForWidth(entries[0].contentRect.width);
			if (n !== colCount) rebuild(n);
		});
		observer.observe(container);
		return () => observer.disconnect();
	});

	$effect(() => {
		// track items growth (endless scroll appends) and shrink (reset);
		// untrack the placement work; it reads and writes column state
		const len = items.length;
		untrack(() => {
			if (len < placedCount) {
				entered.clear();
				rebuild(colCount || 1);
			} else if (len > placedCount) {
				void placePending();
			}
		});
	});
</script>

<div bind:this={container} class="flex items-start gap-5">
	{#each { length: colCount } as _, i (i)}
		<div class="flex min-w-0 flex-1 flex-col gap-5">
			{#each columns[i] as placed (placed.item.id)}
				<div class={placed.fresh ? 'animate-brick-in' : undefined}>
					{@render brick(placed.item)}
				</div>
			{/each}
		</div>
	{/each}
</div>
