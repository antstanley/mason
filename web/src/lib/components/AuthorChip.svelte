<script lang="ts">
	import type { Author } from '$lib/types';

	// avatarClass sizes the avatar; callers that want a bigger one (the glaze
	// pill) pass their own size-* rather than every card inheriting it.
	let { author, avatarClass = 'size-7' }: { author: Author; avatarClass?: string } = $props();
</script>

<div class="flex min-w-0 items-center gap-2">
	{#if author.avatar}
		<img src={author.avatar} alt="" class="{avatarClass} rounded-full border border-ink/10" loading="lazy" />
	{:else}
		<div class="{avatarClass} rounded-full bg-pop-sun"></div>
	{/if}
	<div class="min-w-0 leading-tight">
		<p class="truncate text-sm font-semibold">{author.displayName ?? author.handle}</p>
		<p class="truncate text-xs opacity-75">@{author.handle}</p>
	</div>
</div>
