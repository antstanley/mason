<script lang="ts">
	import type { PostBrick } from '$lib/types';
	import { clientUrl } from '$lib/state/client.svelte';
	import BrickShell from '../BrickShell.svelte';
	import AuthorChip from '../AuthorChip.svelte';
	import Sensitive from '../Sensitive.svelte';
	import LinkPreview from '../LinkPreview.svelte';
	import Icon from '../Icon.svelte';

	// priority: an above-the-fold brick loads its image eagerly and at high
	// fetch priority; the rest of the wall stays lazy
	let { brick, priority = false }: { brick: PostBrick; priority?: boolean } = $props();

	const img = $derived(brick.images[0] ?? null);
	// A picture the post attached wins over one belonging to a link it mentioned:
	// the first is what somebody chose to show, the second is what a page happened
	// to advertise. So the link's og:image only stands in when there is no image
	// of its own, and when there is neither, the embed falls back to the text
	// block below.
	const ogPreview = $derived(!img && brick.external?.thumb ? brick.external : null);
	const label = $derived(`post by ${brick.author.displayName ?? brick.author.handle}`);
</script>

<BrickShell accent="post" href={clientUrl(brick.url)} {brick} {label}>
	{#if img}
		<Sensitive id={brick.id} blur={brick.blur}>
			<img
				src={img.src}
				alt={img.alt}
				loading={priority ? 'eager' : 'lazy'}
				fetchpriority={priority ? 'high' : undefined}
				class="w-full bg-brick-post/15 object-cover"
				style:aspect-ratio={img.aspectRatio ? `${img.aspectRatio.width} / ${img.aspectRatio.height}` : undefined}
			/>
		</Sensitive>
	{:else if ogPreview}
		<!-- behind the same reveal an attached image gets: a `!warn` label covers
		     the brick's media, and a picture pulled from a stranger's page is media
		     by the same measure -->
		<Sensitive id={brick.id} blur={brick.blur}>
			<LinkPreview external={ogPreview} {priority} dense />
		</Sensitive>
	{/if}
	<div class="flex flex-col gap-3 p-4">
		{#if brick.text}
			<p class="text-[0.95rem] leading-snug">{brick.text}</p>
		{/if}
		<!-- only when the link has no picture of its own, or the post's own image
		     already took the top of the brick: the preview above says all of this
		     and saying it twice reads as a duplicate -->
		{#if brick.external && !ogPreview}
			<div class="rounded-xl border border-ink/10 bg-plaster-deep/50 p-3 dark:border-chalk/10 dark:bg-kiln-deep/60">
				<p class="truncate text-sm font-semibold">{brick.external.title}</p>
				<p class="line-clamp-2 text-xs opacity-75">{brick.external.description}</p>
			</div>
		{/if}
		<div class="flex flex-wrap items-center justify-between gap-x-2 gap-y-2">
			<AuthorChip author={brick.author} />
			<!-- a fresh brick has no tallies yet; zeros would just read as neglect -->
			{#if brick.likeCount > 0 || brick.repostCount > 0}
				<div class="flex shrink-0 gap-2 text-xs font-semibold opacity-75">
					{#if brick.likeCount > 0}
						<span class="inline-flex items-center gap-1" aria-label="{brick.likeCount} likes">
							<Icon name="heart" class="size-3" />
							{brick.likeCount}
						</span>
					{/if}
					{#if brick.repostCount > 0}
						<span class="inline-flex items-center gap-1" aria-label="{brick.repostCount} reposts">
							<Icon name="repeat-2" class="size-3" />
							{brick.repostCount}
						</span>
					{/if}
				</div>
			{/if}
		</div>
	</div>
</BrickShell>
