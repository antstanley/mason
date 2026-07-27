<script lang="ts">
	// One brick, read in place, over the wall it came from. Mounted once in
	// +layout.svelte as a SIBLING of the content wrapper, never inside it: that
	// wrapper goes `inert` while the reader is up and `inert` covers every
	// descendant, so a reader nested in the thing it dims would open
	// unfocusable and invisible to assistive tech.
	//
	// It renders what the card had to leave out, from the brick the card was
	// already holding: every image at its own aspect, the whole text, the whole
	// description, every tag. Nothing here reads the network, and nothing here
	// shows anything but the one selected brick: no replies, no thread, no
	// parent post. That boundary is what keeps this a rendering change.
	//
	// Every decision the reader makes lives in the `reader` rune, which is
	// typechecked and unit tested; a .svelte body is neither. What is left here
	// is markup, focus, the scroll lock and one key listener.
	import { tick } from 'svelte';
	import type { BlogBrick, PostBrick, VideoBrick } from '$lib/types';
	import { reader } from '$lib/state/reader.svelte';
	import { player } from '$lib/state/player.svelte';
	import { clientName, clientUrl } from '$lib/state/client.svelte';
	import { dateLabel, runtimeLabel } from '$lib/format';
	import AuthorChip from './AuthorChip.svelte';
	import Icon from './Icon.svelte';
	import LinkPreview from './LinkPreview.svelte';
	import Sensitive from './Sensitive.svelte';
	import VideoPlayer from './VideoPlayer.svelte';

	// The panel, focused as it opens so a keyboard reader lands inside the
	// reader rather than at the top of the inert wall behind it.
	let panel = $state<HTMLElement | null>(null);

	// What to render, straight off the rune. `showing` is history state AND the
	// held brick agreeing, which is the SAME predicate the layout hangs the
	// wall's `inert` and this file hangs the scroll lock on, read from one place
	// so the three cannot answer differently. Page state on its own is wider: an
	// entry outlives the rune that pushed it, so a reload then a forward
	// navigation restores an id with no brick behind it, and the answer to that
	// is a shut reader over a live wall.
	const brick = $derived(reader.showing);

	// The same answer as a boolean, read through a derived rather than off the
	// rune inside the effect below. A derived wakes its readers only when the
	// VALUE changes, and stepping replaces the history entry without changing
	// whether the reader is up. Read straight off the rune, the teardown below
	// runs on every step rather than once per read: it releases and retakes the
	// scroll lock and hands focus back to the card mid-read, which is survivable
	// today only because that card sits inside the inert wrapper and cannot take
	// focus. Stepping is what this reader does on every arrow key, so this is
	// the difference between one teardown per read and one per brick.
	const isOpen = $derived(reader.isOpen);

	// The dialog's accessible name comes off the brick itself: a blog leads with
	// its title, and every other kind leads with its author line, which is the
	// first thing the panel renders.
	const label = $derived.by(() => {
		const open = brick;
		if (!open) return '';
		return open.kind === 'blog' ? open.title : (open.author.displayName ?? `@${open.author.handle}`);
	});

	// The one video slot, claimed under an id of the READER's own, derived from
	// the brick but deliberately never equal to it. A card collapses back to its
	// poster only when player.activeId stops matching its own brick id, so a
	// reader claiming that same id would leave the card mounted and playing
	// behind the scrim: two elements, two audio streams. A distinct id makes the
	// card a loser of the claim and tears it down through the path that already
	// exists.
	const playerId = $derived(brick ? `reader:${brick.id}` : '');

	// Which player the reader has been asked for, held as that id rather than as
	// a boolean. Stepping from one video brick to the next then shows a poster
	// and a play button again, instead of handing the live player a new playlist
	// with no click behind it. Empty string is "none asked for": the id is
	// always prefixed, so it can never collide with one.
	let requested = $state('');
	const showPlayer = $derived(playerId !== '' && requested === playerId);

	// something else claimed the slot (a card, or the next brick's player):
	// collapse back to the poster, exactly as a card does
	$effect(() => {
		if (requested !== '' && player.activeId !== requested) requested = '';
	});

	// Focus moves into the reader as it opens. The binding is the signal rather
	// than a flag: it is set the moment the panel enters the DOM and cleared as
	// it leaves, so this focuses on open and does nothing at all on close.
	$effect(() => {
		panel?.focus();
	});

	/** Step one brick along the laid wall, from a key or from a control. */
	function step(delta: 1 | -1) {
		if (delta === 1) reader.next();
		else reader.prev();
		// the panel is the same element throughout a read, so its scroll offset
		// survives the swap: step off the bottom of a five-image post and the
		// next brick would open halfway down, or past its end. Every brick starts
		// at its own top.
		if (panel) panel.scrollTop = 0;
		// Two ways a step drops focus on the floor, both ending at <body>: the
		// control that stepped runs out of wall and goes `disabled`, or the step
		// crosses a kind boundary and unmounts the control entirely, which is
		// what happens to the play button, "read at <publication>" and the embed
		// anchor, since only one kind renders each. Body is outside the dialog,
		// with nothing to Tab back into while the wall is inert.
		//
		// So ask where focus ENDED UP rather than which of the two happened: a
		// control that survived keeps it, and anything else hands it to the
		// panel, which is where a fresh brick would have started anyway. The
		// question is only answerable after Svelte has swapped the body in, and
		// it does that in a microtask, hence the tick.
		void tick().then(() => {
			if (panel && !panel.contains(document.activeElement)) panel.focus();
		});
	}

	// Escape closes, and the horizontal arrows step. Bound to the document
	// rather than to the panel, so both work wherever focus has landed inside
	// the reader, and attached only while the reader is up (below), so neither
	// can reach a reader that is already shut.
	//
	// The arrows cannot collide with the wall's own navigation-key handler
	// (FeedGrid): that one matches the vertical set (ArrowDown, ArrowUp,
	// PageDown, PageUp, Home, End, space) and never the horizontal one. The
	// disjointness is what carries this, NOT the freeze on open: feed.freeze()
	// is async and leaves those listeners attached until its read resolves.
	const onKey = (event: KeyboardEvent) => {
		if (event.key === 'Escape') {
			reader.close();
			return;
		}
		if (event.key !== 'ArrowLeft' && event.key !== 'ArrowRight') return;
		// a modified arrow belongs to the browser: cmd/alt with one is history
		// navigation on macOS and Windows respectively, and stepping a brick
		// instead would be taking a gesture that means "leave"
		if (event.metaKey || event.ctrlKey || event.shiftKey || event.altKey) return;
		// a focused video owns the horizontal arrows: they seek it. Stepping the
		// wall out from under someone scrubbing a video would be a bug, and the
		// reader's own player is the one place inside it that can hold focus and
		// want these keys.
		if (event.target instanceof HTMLMediaElement) return;
		event.preventDefault();
		step(event.key === 'ArrowRight' ? 1 : -1);
	};

	// Everything the reader has to undo, keyed on `reader.isOpen` and on nothing
	// else. The back gesture shuts the reader without ever calling close(), so
	// the teardown hangs off the state rather than off a control. It must never
	// call reader.close() itself, either: close() pops history only for an entry
	// it pushed and clears that flag only inside itself, so a close() on an
	// already-shut reader would pop a SECOND entry and leave the wall.
	$effect(() => {
		if (!isOpen) return;
		const root = document.documentElement;
		const restore = root.style.overflow;
		// the wall must not scroll behind the reader. This does not stop the
		// pump: bricks may still be appended back there, which is harmless,
		// since an append never moves a laid brick and the reader locates its
		// own brick by id.
		root.style.overflow = 'hidden';
		// on the document rather than the panel: Escape and the arrows work
		// wherever focus has landed inside the reader, and this runs only while
		// the reader is up
		document.addEventListener('keydown', onKey);
		return () => {
			root.style.overflow = restore;
			document.removeEventListener('keydown', onKey);
			// back where they were on the wall: the card this was opened from.
			// Runs on unmount too, which is the only other way teardown happens.
			reader.returnFocus();
		};
	});
</script>

<!-- The trip to the source, demoted to one control inside the reader. It stays
     a real link with a real href, so it still copies, opens in a tab and
     shares; the card's own anchor is the one the reader took over, not this. -->
{#snippet sourceLink(href: string, text: string, accent: string)}
	<a
		{href}
		target="_blank"
		rel="noopener noreferrer"
		class="inline-flex min-h-11 w-fit items-center gap-1 text-sm font-semibold hover:underline {accent}"
	>
		{text}
		<Icon name="arrow-up-right" class="size-3.5" />
	</a>
{/snippet}

<!-- The three bodies, one per kind, mirroring the wall's own kind switch
     (FeedGrid). Each shows what its card could not: the card is a summary by
     design, and this is the same brick with nothing left out. -->
{#snippet postBody(post: PostBrick)}
	{@const stamp = dateLabel(post.createdAt)}
	{#if post.images.length}
		<!-- EVERY image, each at its own aspect ratio, stacked rather than the
		     first one alone. A glaze brick is a post brick, so this is also what
		     turns a filmstrip into a stack and puts every description in reach. -->
		<Sensitive id={post.id} blur={post.blur}>
			<div class="flex flex-col gap-3">
				{#each post.images as image, i (i)}
					<figure class="overflow-hidden rounded-xl bg-brick-post/15">
						<img
							src={image.src}
							alt={image.alt}
							loading={i === 0 ? 'eager' : 'lazy'}
							class="w-full object-contain"
							style:aspect-ratio={image.aspectRatio
								? `${image.aspectRatio.width} / ${image.aspectRatio.height}`
								: undefined}
						/>
						{#if image.alt.trim()}
							<!-- the description, on the page rather than only in the alt
							     attribute: the glaze card hides these behind an ALT panel
							     because the picture IS the card there, and the reader has
							     the room to just show them -->
							<figcaption class="px-3 py-2 text-xs leading-snug opacity-75">
								<span class="font-bold tracking-wide uppercase opacity-70">alt</span>
								{image.alt}
							</figcaption>
						{/if}
					</figure>
				{/each}
			</div>
		</Sensitive>
	{/if}
	{#if post.text}
		<!-- unclamped, and line breaks kept: the card gives this three lines of
		     leading-snug, and a post that was written in stanzas reads as one
		     paragraph there -->
		<p class="text-base leading-relaxed whitespace-pre-wrap">{post.text}</p>
	{/if}
	{#if post.external}
		<a
			href={post.external.uri}
			target="_blank"
			rel="noopener noreferrer"
			class="block overflow-hidden rounded-xl border border-ink/10 bg-plaster-deep/50 transition-colors hover:border-ink/25 dark:border-chalk/10 dark:bg-kiln-deep/60 dark:hover:border-chalk/30"
		>
			{#if post.external.thumb}
				<!-- the link's own picture, with what it says over the foot of it. The
				     card shows this INSTEAD of the post's images and only when there
				     are none; the reader has room for both, so a post that attached a
				     picture and linked somewhere shows the pictures above and this
				     underneath. Behind the same reveal, since it is a stranger's
				     picture either way. -->
				<Sensitive id={post.id} blur={post.blur}>
					<LinkPreview external={post.external} />
				</Sensitive>
			{:else}
				<!-- no picture on the other end, so the words carry it. The card
				     truncates this title to one line and clamps the description to
				     two; here it is the whole embed -->
				<div class="p-4">
					<p class="font-semibold">{post.external.title}</p>
					{#if post.external.description}
						<p class="mt-1 text-sm leading-snug opacity-75">{post.external.description}</p>
					{/if}
					<span class="mt-2 inline-flex items-center gap-1 text-xs font-semibold opacity-60">
						{post.external.uri}
						<Icon name="arrow-up-right" class="size-3" />
					</span>
				</div>
			{/if}
		</a>
	{/if}
	{@const postHref = clientUrl(post.url)}
	{@const postClient = clientName(postHref)}
	<!-- One line, counts on the left and the way out on the right. They were
	     stacked, which spent a whole row on a control the width of six words and
	     pushed the stepper further from the text it steps. `gap-x-3` between the
	     two halves is the floor, so the link never sits against the counts even
	     when the panel is at its narrowest; past that they wrap and the link keeps
	     the right edge on its own line. -->
	<div class="flex flex-wrap items-center justify-between gap-x-3 text-sm">
		<div class="flex flex-wrap items-center gap-x-3 gap-y-1 opacity-75">
			{#if stamp}
				<!-- when it was laid: the card has never had room for a timestamp -->
				<time datetime={post.createdAt}>{stamp}</time>
			{/if}
			{#if post.likeCount > 0}
				<span class="inline-flex items-center gap-1" aria-label="{post.likeCount} likes">
					<Icon name="heart" class="size-3.5" />
					{post.likeCount}
				</span>
			{/if}
			{#if post.repostCount > 0}
				<span class="inline-flex items-center gap-1" aria-label="{post.repostCount} reposts">
					<Icon name="repeat-2" class="size-3.5" />
					{post.repostCount}
				</span>
			{/if}
		</div>
		<!-- named after where it actually lands rather than after the setting: a
		     post whose url is not a bsky.app one passes through clientUrl untouched,
		     and "open in Twinkl" would be a promise the link does not keep. `ms-auto`
		     rather than relying on justify-between alone, which would strand the
		     link on the left when a post has no timestamp and no counts. -->
		{@render sourceLink(
			postHref,
			postClient ? `open in ${postClient}` : 'open the post',
			'ms-auto text-brick-post-ink dark:text-brick-post'
		)}
	</div>
{/snippet}

{#snippet blogBody(blog: BlogBrick)}
	{@const stamp = dateLabel(blog.publishedAt)}
	{#if blog.coverImage}
		<!-- the cover at full width, where the card gives it a fixed 8/5 strip at
		     card size. It goes behind the same reveal as every other piece of
		     media in the reader even though a blog brick carries no blur today:
		     the label tiers ride on Bluesky records and never reach a document,
		     so this renders its child and nothing else, and there is one less
		     exception to remember if that ever stops being true. -->
		<Sensitive id={blog.id}>
			<img src={blog.coverImage} alt="" class="w-full rounded-xl bg-brick-blog/15 object-cover" />
		</Sensitive>
	{/if}
	<span
		class="w-fit rounded-full bg-brick-blog/15 px-2.5 py-0.5 text-[0.7rem] font-bold tracking-wide text-brick-blog-ink uppercase dark:text-brick-blog"
	>
		{blog.publication.name}
	</span>
	<h2 class="font-display text-2xl leading-tight font-bold">{blog.title}</h2>
	{#if blog.description}
		<!-- the whole description; the card clamps it to three lines -->
		<p class="text-base leading-relaxed">{blog.description}</p>
	{/if}
	{#if blog.tags.length}
		<!-- every tag, where the card shows the first four and drops the rest.
		     Keyed by index and NOT by the tag itself: a document's tags reach the
		     client exactly as the record wrote them, with no dedupe anywhere on
		     the way (mortar's standardsite source passes `tags: doc.tags`
		     straight through), so a blog tagged ['a', 'b', 'a'] hands this block
		     the same key twice. Svelte answers a repeated key by THROWING
		     each_key_duplicate as it renders, which here means the reader never
		     opens at all: the click does nothing and no dialog appears. -->
		<div class="flex flex-wrap gap-1.5">
			{#each blog.tags as tag, i (i)}
				<span class="rounded-full bg-pop-lime/25 px-2 py-0.5 text-[0.68rem] font-semibold">#{tag}</span>
			{/each}
		</div>
	{/if}
	{#if stamp}
		<p class="text-sm opacity-75">
			<time datetime={blog.publishedAt}>{stamp}</time>
		</p>
	{/if}
	<!-- the primary control on a blog, not a footnote: mason never parses the
	     content union of a document, so the publication is where the article
	     itself lives and the reader says so plainly -->
	<a
		href={blog.url}
		target="_blank"
		rel="noopener noreferrer"
		class="inline-flex min-h-11 w-fit items-center gap-2 rounded-full bg-pop-pink-deep px-6 py-3 font-display font-bold text-white shadow-brick transition-transform motion-safe:hover:scale-105 motion-safe:active:scale-95"
	>
		read at {blog.publication.name}
		<Icon name="arrow-up-right" class="size-4" />
	</a>
{/snippet}

{#snippet videoBody(video: VideoBrick)}
	{@const ratio = video.aspectRatio
		? `${video.aspectRatio.width} / ${video.aspectRatio.height}`
		: '16 / 9'}
	{@const stamp = dateLabel(video.createdAt)}
	{@const runtime = runtimeLabel(video.durationMs ?? 0)}
	<Sensitive id={video.id} blur={video.blur}>
		<div class="relative overflow-hidden rounded-xl">
			{#if showPlayer}
				<VideoPlayer
					id={playerId}
					playlist={video.playlist}
					poster={video.poster}
					aspectRatio={ratio}
					live={video.live}
					captions={video.captions ?? []}
				/>
			{:else}
				{#if video.poster}
					<img
						src={video.poster}
						alt=""
						loading="eager"
						class="w-full bg-brick-video/15 object-cover"
						style:aspect-ratio={ratio}
					/>
				{:else}
					<div class="w-full bg-brick-video/20" style:aspect-ratio={ratio}></div>
				{/if}
				<button
					type="button"
					onclick={() => {
						// claim synchronously, and under the reader's own id: the card
						// behind the scrim is watching player.activeId for exactly this,
						// and stops matching the moment this lands. Claiming here rather
						// than leaving it to the player also keeps the collapse effect
						// above from reading this reader as a loser of its own click.
						player.claim(playerId);
						requested = playerId;
					}}
					class="absolute inset-0 grid cursor-pointer place-items-center focus-visible:outline-offset-[-3px]"
					aria-label={video.live ? 'Watch live' : 'Play video'}
				>
					<span
						class="grid size-16 place-items-center rounded-full pl-1 text-white shadow-brick-lift transition-transform motion-safe:hover:scale-110 {video.live
							? 'bg-live'
							: 'bg-brick-video'}"
						aria-hidden="true"
					>
						<Icon name="play" class="size-7" />
					</span>
				</button>
				{#if video.live}
					<span
						class="pointer-events-none absolute top-2 left-2 flex items-center gap-1.5 rounded-full bg-live px-2.5 py-0.5 text-[0.7rem] font-bold tracking-wide text-white uppercase"
					>
						<span class="size-1.5 rounded-full bg-white motion-safe:animate-pulse" aria-hidden="true"
						></span>
						live
					</span>
				{/if}
				{#if runtime}
					<span
						class="pointer-events-none absolute right-2 bottom-2 rounded-full bg-kiln/75 px-2 py-0.5 text-[0.7rem] font-bold text-chalk tabular-nums"
					>
						{runtime}
					</span>
				{/if}
			{/if}
		</div>
	</Sensitive>
	{#if video.title}
		<h2 class="font-display text-xl leading-snug font-bold">{video.title}</h2>
	{/if}
	<div class="flex flex-wrap items-center gap-x-3 gap-y-1 text-sm opacity-75">
		{#if video.live && video.viewerCount !== null}
			<span class="font-semibold text-live dark:text-live-bright">
				{video.viewerCount === 1 ? '1 watching' : `${video.viewerCount} watching`}
			</span>
		{/if}
		{#if video.activity}
			<span>{video.activity}</span>
		{/if}
		{#if stamp}
			<time datetime={video.createdAt}>{stamp}</time>
		{/if}
	</div>
	{@render sourceLink(
		clientUrl(video.url),
		`${video.live ? 'watch live' : 'watch'} on ${video.source === 'streamplace' ? 'Streamplace' : 'Bluesky'}`,
		'text-brick-video-ink dark:text-brick-video-bright'
	)}
{/snippet}

{#if brick}
	<!-- the scrim: dims the wall in light mode, veils it in dark, and swallows
	     every click so nothing behind can be touched while the reader is up.
	     Tapping it is a dismiss, in the same language as SwitchWall's. -->
	<button
		type="button"
		tabindex="-1"
		aria-label="Close the reader"
		onclick={() => reader.close()}
		class="fixed inset-0 z-40 animate-scrim-in cursor-default bg-ink/50 backdrop-blur-[2px] dark:bg-kiln-deep/70"
	></button>
	<!-- pointer-events-none, so the gutter around the panel is the scrim rather
	     than a dead zone: a click beside the panel lands on the button under it -->
	<div class="pointer-events-none fixed inset-0 z-50 grid place-items-center p-3 sm:p-6">
		<!-- tabindex -1: the panel takes focus from script and never from a Tab,
		     which is why it drops the focus ring. Chromium draws one on a
		     programmatically focused container even when the open came from a
		     mouse, and a ring that marks nothing the keyboard can reach is noise. -->
		<div
			bind:this={panel}
			role="dialog"
			aria-modal="true"
			aria-label={label}
			tabindex="-1"
			class="pointer-events-auto max-h-full w-full max-w-2xl animate-reader-in overflow-y-auto overscroll-contain rounded-card border border-ink/10 bg-chalk p-4 text-left shadow-brick-lift focus-visible:outline-none sm:p-6 dark:border-chalk/15 dark:bg-kiln"
		>
			<div class="flex items-start justify-between gap-3">
				<AuthorChip author={brick.author} avatarClass="size-10" />
				<button
					type="button"
					onclick={() => reader.close()}
					aria-label="Close the reader"
					class="-mt-1 -mr-1 inline-flex min-h-11 min-w-11 shrink-0 cursor-pointer items-center justify-center rounded-full transition-transform motion-safe:hover:scale-105 motion-safe:active:scale-95"
				>
					<Icon name="x" class="size-5" />
				</button>
			</div>
			<!-- one brick's own content, and nothing around it -->
			<div class="mt-4 flex flex-col gap-4">
				{#if brick.kind === 'post'}
					{@render postBody(brick)}
				{:else if brick.kind === 'blog'}
					{@render blogBody(brick)}
				{:else}
					{@render videoBody(brick)}
				{/if}
			</div>
			<!-- along the wall without leaving the reader. Both controls stop where
			     the laid wall stops: stepping never pages, because a page pulled in
			     from here would grow a wall this reader cannot see. -->
			<div
				class="mt-5 flex items-center justify-between gap-2 border-t border-ink/10 pt-3 dark:border-chalk/10"
			>
				<button
					type="button"
					onclick={() => step(-1)}
					disabled={!reader.canPrev}
					class="inline-flex min-h-11 cursor-pointer items-center gap-1 rounded-full px-3 text-sm font-semibold transition-colors hover:bg-ink/5 disabled:cursor-default disabled:opacity-35 disabled:hover:bg-transparent dark:hover:bg-chalk/10"
				>
					<Icon name="chevron-left" class="size-5" />
					previous brick
				</button>
				<!-- No count between the two controls. It read as progress through the
				     wall and was not: the pump keeps laying while the reader is up, so
				     the denominator grows underneath a number that looked fixed, and
				     "4 of 22" became "4 of 31" without the reader having moved. The
				     live region below still says where a step landed, which is a
				     different job: it announces that the panel swapped at all, to
				     somebody who cannot see that it did. -->
				<button
					type="button"
					onclick={() => step(1)}
					disabled={!reader.canNext}
					class="inline-flex min-h-11 cursor-pointer items-center gap-1 rounded-full px-3 text-sm font-semibold transition-colors hover:bg-ink/5 disabled:cursor-default disabled:opacity-35 disabled:hover:bg-transparent dark:hover:bg-chalk/10"
				>
					next brick
					<Icon name="chevron-right" class="size-5" />
				</button>
			</div>
			<!-- a step swaps the whole panel under someone who cannot see it, so
			     say where the reader has landed -->
			<p class="sr-only" aria-live="polite">
				{#if reader.index >= 0}brick {reader.index + 1} of {reader.total}{/if}
			</p>
		</div>
	</div>
{/if}
