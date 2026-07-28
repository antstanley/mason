<script lang="ts">
	// A link's own picture, with what the link says laid over the foot of it.
	//
	// The picture is the `og:image` from the page's headers. mason does not fetch
	// it: Bluesky's AppView resolves the Open Graph tags when the link card is
	// made and hands the result over as `external.thumb`, so this is a rendering
	// change with no upstream read, no wire field and no cache behind it. That is
	// also why this only ever appears on a post brick. A blog carries its own
	// cover and a stream its own poster, and neither has a link with headers to
	// read.
	//
	// It stands in for a picture rather than beside one: a post that attached an
	// image of its own shows that, and this is what the wall shows when the only
	// picture a post has is the one belonging to the thing it linked to.
	import type { ExternalEmbed } from '$lib/types';

	let {
		external,
		priority = false,
		dense = false
	}: { external: ExternalEmbed; priority?: boolean; dense?: boolean } = $props();

	/** The host, not the whole URL. A full address is mostly path and query, wraps
	 *  badly over a picture, and the part that answers "where does this go" is the
	 *  host. `www.` goes because it says nothing. Falls back to the raw string
	 *  rather than disappearing if it will not parse; the AppView vets these, but
	 *  a card that renders nothing is worse than one that renders something ugly. */
	const host = $derived.by(() => {
		try {
			return new URL(external.uri).hostname.replace(/^www\./, '');
		} catch {
			return external.uri;
		}
	});
</script>

<figure class="relative">
	<!-- 1.91:1 is Open Graph's own ratio and what almost every og:image is cut to.
	     It is pinned rather than measured because the wire carries no aspect ratio
	     for a thumb, and an unsized image on a masonry wall reflows every brick
	     below it the moment it loads. `object-cover` takes the crop for the few
	     that are shaped differently. -->
	<img
		src={external.thumb}
		alt=""
		loading={priority ? 'eager' : 'lazy'}
		fetchpriority={priority ? 'high' : undefined}
		class="aspect-[1.91/1] w-full bg-brick-post/15 object-cover"
	/>
	<!-- The overlay sits on the picture rather than under it, so a link brick reads
	     as one thing. The gradient is what makes the text legible over an image
	     nobody chose for its contrast: it is the scrim, so it carries no text of
	     its own and is hidden from assistive tech. -->
	<figcaption
		class="absolute inset-x-0 bottom-0 bg-gradient-to-t from-ink/90 via-ink/70 to-transparent px-3 pt-8 text-chalk {dense
			? 'pb-2'
			: 'pb-3'}"
	>
		<p class="truncate text-[0.7rem] font-semibold tracking-wide uppercase opacity-80">
			{host}
		</p>
		<p class="mt-0.5 font-semibold {dense ? 'line-clamp-2 text-sm' : 'text-base'} leading-snug">
			{external.title}
		</p>
		{#if external.description}
			<p class="mt-0.5 text-xs leading-snug opacity-85 {dense ? 'line-clamp-2' : 'line-clamp-3'}">
				{external.description}
			</p>
		{/if}
	</figcaption>
</figure>
