<script lang="ts">
	// The layout toggle behaves like a slider: a single thumb slides between the
	// two halves and the labels crossfade as it passes under them. Native radios
	// underneath keep it keyboard- and screen-reader-honest (arrow keys move the
	// selection), and the thumb holds still under prefers-reduced-motion.
	import { LAYOUTS, layout, type LayoutId } from '$lib/state/layout.svelte';

	const selected = $derived(Math.max(
		0,
		LAYOUTS.findIndex((option) => option.id === layout.id)
	));
</script>

<fieldset class="rounded-full border-2 border-ink/15 p-1 dark:border-chalk/20">
	<legend class="sr-only">Wall layout</legend>
	<div class="relative flex">
		<span
			aria-hidden="true"
			class="pointer-events-none absolute inset-y-0 left-0 w-1/2 rounded-full bg-ink shadow-brick transition-transform duration-300 ease-[cubic-bezier(0.16,1,0.3,1)] motion-reduce:transition-none dark:bg-chalk"
			style:transform="translateX({selected * 100}%)"
		></span>
		{#each LAYOUTS as option (option.id)}
			<label class="relative z-10 flex-1 cursor-pointer">
				<input
					type="radio"
					name="layout"
					value={option.id}
					checked={layout.id === option.id}
					onchange={() => layout.set(option.id as LayoutId)}
					class="peer sr-only"
				/>
				<span
					class="flex min-h-9 items-center justify-center gap-1.5 rounded-full px-4 text-sm font-semibold text-ink transition-colors duration-300 peer-checked:text-chalk peer-focus-visible:outline-3 peer-focus-visible:outline-offset-2 peer-focus-visible:outline-pop-pink-deep motion-reduce:transition-none dark:text-chalk dark:peer-checked:text-ink"
				>
					<span aria-hidden="true" class="text-2xl leading-none sm:text-sm">{option.icon}</span>
					<span class="sr-only sm:not-sr-only">{option.label}</span>
				</span>
			</label>
		{/each}
	</div>
</fieldset>
