<script lang="ts">
	// The glaze brick: the picture is the brick. One image fills the card at its
	// natural aspect; two or three lay out in a grid (grout showing between
	// them); four or more become an in-card filmstrip — a full-frame strip you
	// swipe, or page with the arrows, committing to the next image only once
	// you've dragged in more than 60% of it. At rest the card is just the picture;
	// on hover an opaque author pill fades in bottom-left and the post's caption
	// slides up underneath it on its frosted bar. The caption carries an ALT
	// button when any image has a description; clicking it expands the frosted
	// section over the whole card, the descriptions in a section under the caption.
	//
	// Each image is its own link and the controls sit outside them, so a tap opens
	// the post while a drag scrolls the strip, and the arrows / ALT button / panel
	// never trip the navigation. Touch has no hover, so there the pill and caption
	// stay shown. Under prefers-reduced-motion nothing slides.
	import type { PostBrick } from '$lib/types';
	import { clientUrl } from '$lib/state/client.svelte';
	import BrickShell from '../BrickShell.svelte';
	import AuthorChip from '../AuthorChip.svelte';
	import Sensitive from '../Sensitive.svelte';

	let { brick }: { brick: PostBrick } = $props();

	const images = $derived(brick.images);
	const count = $derived(images.length);
	const kind = $derived(count >= 4 ? 'carousel' : count >= 2 ? 'grid' : 'single');
	const first = $derived(images[0] ?? null);
	// descriptions to surface, tagged with their 1-based image number
	const alts = $derived(
		images.map((im, i) => ({ n: i + 1, text: (im.alt ?? '').trim() })).filter((a) => a.text)
	);

	let showAlt = $state(false);
	// the committed slide (what the counter shows); `anchor` mirrors it for the
	// threshold maths, which must not depend on reactive timing
	let index = $state(0);
	let anchor = 0;
	let strip = $state<HTMLDivElement | null>(null);
	// true while a programmatic correction is scrolling, so its own scroll events
	// do not re-trigger the settle logic
	let correcting = false;
	let settleTimer: ReturnType<typeof setTimeout> | undefined;

	function ratio(
		im: { aspectRatio: { width: number; height: number } | null } | null | undefined
	): string | undefined {
		return im?.aspectRatio ? `${im.aspectRatio.width} / ${im.aspectRatio.height}` : undefined;
	}

	// one slide's worth of scroll (slides are full-frame, so this is the strip's
	// own width), measured live off the DOM
	function slideStep(): number {
		const kids = strip?.children;
		if (!kids || kids.length < 2) return strip?.clientWidth ?? 1;
		return (kids[1] as HTMLElement).offsetLeft - (kids[0] as HTMLElement).offsetLeft;
	}

	function goTo(target: number) {
		if (!strip) return;
		anchor = Math.min(count - 1, Math.max(0, target));
		index = anchor;
		correcting = true;
		clearTimeout(settleTimer);
		// scroll-behavior (smooth, or auto under reduced-motion) lives on the strip
		strip.scrollTo({ left: anchor * slideStep() });
		settleTimer = setTimeout(() => (correcting = false), 320);
	}

	// After a free scroll settles, commit to a neighbour only once MORE THAN 60%
	// of it has been dragged in; anything less snaps back to where we were.
	function settle() {
		if (!strip || correcting) return;
		const step = slideStep();
		if (step <= 0) return;
		const delta = strip.scrollLeft / step - anchor;
		const target = delta > 0 ? anchor + Math.floor(delta + 0.4) : anchor + Math.ceil(delta - 0.4);
		goTo(target);
	}

	// scrollend is the clean signal (fires after touch + momentum); the debounced
	// fallback covers browsers without it. Both defer to `correcting`.
	function onScrollEnd() {
		settle();
	}
	function onScroll() {
		if (correcting) return;
		clearTimeout(settleTimer);
		settleTimer = setTimeout(settle, 140);
	}

	function slide(dir: number) {
		goTo(anchor + dir);
	}
</script>

<BrickShell accent="post">
	<div class="relative">
		{#if kind === 'carousel'}
			<Sensitive blur={brick.blur}>
				<div
					bind:this={strip}
					onscroll={onScroll}
					onscrollend={onScrollEnd}
					class="flex snap-none overflow-x-auto scroll-smooth bg-brick-post/15 [-ms-overflow-style:none] [scrollbar-width:none] motion-reduce:scroll-auto [&::-webkit-scrollbar]:hidden"
					style:aspect-ratio={ratio(images[0])}
				>
					{#each images as im, i (i)}
						<a
							href={clientUrl(brick.url)}
							target="_blank"
							rel="noopener noreferrer"
							class="block h-full w-full shrink-0 focus-visible:outline-offset-[-3px]"
						>
							<img src={im.src} alt={im.alt} loading="lazy" class="h-full w-full object-cover" />
						</a>
					{/each}
				</div>
			</Sensitive>

			<!-- controls: siblings of the strip, so they page it instead of opening
			     the post. A native swipe pages it too. -->
			<div
				class="pointer-events-none absolute inset-x-2 top-1/2 flex -translate-y-1/2 justify-between opacity-0 transition-opacity group-hover:opacity-100 motion-reduce:transition-none [@media(hover:none)]:opacity-100"
			>
				<button
					type="button"
					onclick={() => slide(-1)}
					aria-label="Previous image"
					class="pointer-events-auto grid size-9 cursor-pointer place-items-center rounded-full bg-ink/55 text-lg font-bold text-chalk backdrop-blur-sm transition-colors hover:bg-ink/70"
				>
					<span aria-hidden="true">‹</span>
				</button>
				<button
					type="button"
					onclick={() => slide(1)}
					aria-label="Next image"
					class="pointer-events-auto grid size-9 cursor-pointer place-items-center rounded-full bg-ink/55 text-lg font-bold text-chalk backdrop-blur-sm transition-colors hover:bg-ink/70"
				>
					<span aria-hidden="true">›</span>
				</button>
			</div>
			<div
				class="pointer-events-none absolute top-2 right-2 rounded-full bg-ink/55 px-2 py-0.5 text-xs font-semibold text-chalk backdrop-blur-sm"
			>
				{index + 1}/{count}
			</div>
		{:else}
			<a
				href={clientUrl(brick.url)}
				target="_blank"
				rel="noopener noreferrer"
				class="block focus-visible:outline-offset-[-3px]"
			>
				<Sensitive blur={brick.blur}>
					{#if kind === 'single' && first}
						<img
							src={first.src}
							alt={first.alt}
							loading="lazy"
							class="block w-full bg-brick-post/15 object-cover"
							style:aspect-ratio={ratio(first)}
						/>
					{:else}
						<!-- 2-up (two full-height columns), or 3-up with the first image
						     big on the left. gap-1 shows the card behind it, grout. -->
						<div class="grid aspect-[3/2] grid-cols-2 grid-rows-2 gap-1 bg-brick-post/15">
							{#each images as im, i (i)}
								<img
									src={im.src}
									alt={im.alt}
									loading="lazy"
									class="h-full w-full object-cover {i === 0 || count === 2 ? 'row-span-2' : ''}"
								/>
							{/each}
						</div>
					{/if}
				</Sensitive>
			</a>
		{/if}

		<!-- author pill on top, frosted caption underneath. At rest both are hidden
		     for a clean image (especially the grid). On hover the pill fades in and
		     stays put while the caption slides up underneath it. On touch, where
		     there is no hover, both stay shown. -->
		<div class="pointer-events-none absolute inset-x-0 bottom-0 flex flex-col items-start">
			<div
				class="m-3 max-w-[calc(100%-1.5rem)] rounded-full bg-chalk py-1.5 pr-4 pl-1.5 opacity-0 shadow-brick transition-opacity duration-300 dark:bg-kiln [@media(hover:hover)]:group-hover:opacity-100 [@media(hover:none)]:opacity-100 motion-reduce:transition-none"
			>
				<AuthorChip author={brick.author} avatarClass="size-10" />
			</div>
			{#if brick.text || alts.length}
				<div
					class="max-h-0 w-full overflow-hidden opacity-0 transition-all duration-300 ease-out [@media(hover:hover)]:group-hover:max-h-40 [@media(hover:hover)]:group-hover:opacity-100 [@media(hover:none)]:max-h-40 [@media(hover:none)]:opacity-100 motion-reduce:transition-none"
				>
					<div
						class="flex w-full items-start gap-2 border-t border-chalk/25 bg-chalk/55 p-3 backdrop-blur-md dark:border-kiln/30 dark:bg-kiln/50"
					>
						<p class="line-clamp-2 flex-1 text-[0.9rem] leading-snug [@media(hover:hover)]:line-clamp-4">
							{brick.text}
						</p>
						{#if alts.length}
							<button
								type="button"
								onclick={() => (showAlt = true)}
								aria-label="Show image description"
								class="pointer-events-auto mt-0.5 shrink-0 cursor-pointer rounded-md bg-ink/10 px-1.5 py-0.5 text-[0.65rem] font-bold tracking-wide text-ink/80 transition-colors hover:bg-ink/20 dark:bg-chalk/15 dark:text-chalk/80 dark:hover:bg-chalk/25"
							>
								ALT
							</button>
						{/if}
					</div>
				</div>
			{/if}
		</div>

		<!-- clicked ALT: the frosted section fills the card, caption then the
		     image description(s) under it, numbered when there is more than one -->
		{#if showAlt && alts.length}
			<div
				class="pointer-events-auto absolute inset-0 flex flex-col gap-3 overflow-auto bg-chalk/85 p-4 backdrop-blur-md dark:bg-kiln/85"
			>
				<div class="flex items-start justify-between gap-2">
					<AuthorChip author={brick.author} avatarClass="size-8" />
					<button
						type="button"
						onclick={() => (showAlt = false)}
						aria-label="Hide image description"
						class="shrink-0 cursor-pointer rounded-full bg-ink/10 px-2 py-1 text-sm font-bold text-ink/80 transition-colors hover:bg-ink/20 dark:bg-chalk/15 dark:text-chalk/80 dark:hover:bg-chalk/25"
					>
						✕
					</button>
				</div>
				{#if brick.text}
					<p class="text-[0.9rem] leading-snug">{brick.text}</p>
				{/if}
				<div class="border-t border-ink/10 pt-3 dark:border-chalk/10">
					<p class="text-[0.65rem] font-bold tracking-wide uppercase opacity-60">alt text</p>
					{#each alts as a (a.n)}
						<p class="mt-1 text-sm leading-snug">
							{#if alts.length > 1}<span class="font-semibold opacity-60">{a.n}.</span> {/if}{a.text}
						</p>
					{/each}
				</div>
			</div>
		{/if}
	</div>
</BrickShell>
