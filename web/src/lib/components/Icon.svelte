<script lang="ts">
	// every lucide path the app draws, in one place. Icons are inline svg, never
	// glyph characters: glyphs sit on a font baseline and render unevenly across
	// platforms
	type IconName =
		| 'chevron-left'
		| 'chevron-right'
		| 'chevron-down'
		| 'chevrons-up'
		| 'chevrons-down'
		| 'arrow-left-right'
		| 'arrow-up-right'
		| 'check'
		| 'x'
		| 'play'
		| 'heart'
		| 'repeat-2';

	const ICONS: Record<IconName, { d: string[]; fill?: boolean }> = {
		'chevron-left': { d: ['m15 18-6-6 6-6'] },
		'chevron-right': { d: ['m9 18 6-6-6-6'] },
		'chevron-down': { d: ['m6 9 6 6 6-6'] },
		'chevrons-up': { d: ['m17 11-5-5-5 5', 'm17 18-5-5-5 5'] },
		'chevrons-down': { d: ['m7 6 5 5 5-5', 'm7 13 5 5 5-5'] },
		'arrow-left-right': { d: ['M8 3 4 7l4 4', 'M4 7h16', 'm16 21 4-4-4-4', 'M20 17H4'] },
		'arrow-up-right': { d: ['M7 7h10v10', 'M7 17 17 7'] },
		check: { d: ['M20 6 9 17l-5-5'] },
		x: { d: ['M18 6 6 18', 'm6 6 12 12'] },
		play: { d: ['m6 3 14 9-14 9z'], fill: true },
		heart: {
			d: [
				'M19 14c1.49-1.46 3-3.21 3-5.5A5.5 5.5 0 0 0 16.5 3c-1.76 0-3 .5-4.5 2-1.5-1.5-2.74-2-4.5-2A5.5 5.5 0 0 0 2 8.5c0 2.3 1.5 4.05 3 5.5l7 7Z',
			],
			fill: true,
		},
		'repeat-2': {
			d: ['m2 9 3-3 3 3', 'M13 18H7a2 2 0 0 1-2-2V6', 'm22 15-3 3-3-3', 'M11 6h6a2 2 0 0 1 2 2v10'],
		},
	};

	let { name, class: cls = 'size-4' }: { name: IconName; class?: string } = $props();
	const icon = $derived(ICONS[name]);
</script>

<svg
	viewBox="0 0 24 24"
	class={cls}
	fill={icon.fill ? 'currentColor' : 'none'}
	stroke={icon.fill ? 'none' : 'currentColor'}
	stroke-width="2"
	stroke-linecap="round"
	stroke-linejoin="round"
	aria-hidden="true"
>
	{#each icon.d as d (d)}
		<path {d} />
	{/each}
</svg>
