<script lang="ts">
	let { variant = 0 }: { variant?: number } = $props();

	const heights = ['h-40', 'h-64', 'h-52'];
	const tints = ['bg-brick-post/15', 'bg-brick-blog/15', 'bg-brick-video/15'];
	const i = $derived(((variant % 3) + 3) % 3);
</script>

<div class="animate-pulse overflow-hidden rounded-card border-2 border-ink/10 bg-chalk shadow-brick dark:border-chalk/10 dark:bg-kiln">
	<div class="{heights[i]} {tints[i]} w-full"></div>
	<div class="flex flex-col gap-2.5 p-4">
		<div class="h-3.5 w-4/5 rounded bg-ink/10 dark:bg-chalk/10"></div>
		<div class="h-3.5 w-3/5 rounded bg-ink/10 dark:bg-chalk/10"></div>
		<div class="mt-1 flex items-center gap-2">
			<div class="size-7 rounded-full bg-ink/10 dark:bg-chalk/10"></div>
			<div class="h-3 w-24 rounded bg-ink/10 dark:bg-chalk/10"></div>
		</div>
	</div>
</div>
