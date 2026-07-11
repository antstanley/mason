<script lang="ts">
	import type { Author } from '$lib/types';

	let { author }: { author: Author } = $props();
</script>

<div class="flex items-center gap-2">
	{#if author.avatar}
		<img src={author.avatar} alt="" class="size-7 rounded-full border border-ink/10" loading="lazy" />
	{:else}
		<div class="size-7 rounded-full bg-pop-sun"></div>
	{/if}
	<div class="min-w-0 leading-tight">
		<p class="truncate text-sm font-semibold">{author.displayName ?? author.handle}</p>
		<p class="truncate text-xs opacity-60">@{author.handle}</p>
	</div>
</div>
