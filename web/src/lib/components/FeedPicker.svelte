<script lang="ts">
	// mason's second front door: a screen for finding a feed, standing beside the
	// handle box as a peer rather than as another field on it.
	//
	// Mounted once in +layout.svelte as a SIBLING of the content wrapper, never
	// inside it, for the same reason BrickReader is: that wrapper goes `inert`
	// while either overlay is up and `inert` covers every descendant, so a picker
	// nested in the thing it freezes would open unfocusable and invisible to
	// assistive tech.
	//
	// Every decision it makes lives elsewhere on purpose. Whether it is open, the
	// recents list, the three questions and the hidden-tier filter are in
	// `state/feeds.svelte.ts`; which question one input is asking is in
	// `lib/feedref.ts`. Both are typechecked and unit tested and a `.svelte` body
	// is neither, so what is left here is markup, focus, the scroll lock and one
	// key listener. In particular the history push that holds the picker open is
	// NOT here: it carries the picker's key and nothing else, which is what shuts
	// the brick reader, and half of a rule about two overlays kept in a file
	// nothing can read is half a rule.
	import { tick, untrack } from 'svelte';
	import { goto } from '$app/navigation';
	import { askedFor } from '$lib/feedref';
	import { feeds } from '$lib/state/feeds.svelte';
	import FeedCard from './FeedCard.svelte';
	import Icon from './Icon.svelte';

	// How many skeletons stand in for a page in flight. Six is the picker's own
	// two columns three rows deep, which is enough to read as a list arriving and
	// short enough that it never pushes the recents row off a phone screen.
	const PICKER_SKELETON_COUNT = 6;

	// Whether the picker is up. `page.state.picker` is the whole of it and the
	// rune is the one place that key is read, so the address bar keeps showing
	// whatever is behind the picker and the back gesture closes it. The layout
	// hangs the wrapper's `inert` on the same predicate: a page frozen on a wider
	// condition than the overlay renders on is a page frozen under nothing.
	const isOpen = $derived(feeds.isOpen);

	let input = $state<HTMLInputElement | null>(null);
	let value = $state('');
	// A pasted value that will not parse, said in place. It is a flag rather than
	// a message because there is only one thing to say: what the reader typed
	// reads as a link and mason cannot lay it.
	let unparseable = $state(false);
	// The current question has no further pages. Inferred rather than read, since
	// the cursor is the rune's own business: see `more` below.
	let atEnd = $state(false);

	// ids rather than an aria-label, so the dialog and its sections are named by
	// the same headings a sighted reader sees. One generated base with four
	// suffixes, because `$props.id()` may be called only once per component.
	const uid = $props.id();
	const titleId = `${uid}-title`;
	const errorId = `${uid}-error`;
	const recentId = `${uid}-recent`;
	const resultsId = `${uid}-results`;

	// What the results are an answer to. It is what keeps "no feeds by that name"
	// away from somebody who searched nothing: the empty states below branch on
	// the same value.
	const resultsHeading = $derived(
		feeds.question === 'search'
			? `feeds called "${feeds.term}"`
			: feeds.question === 'creator'
				? `feeds by @${feeds.term}`
				: 'popular feeds'
	);

	function close() {
		feeds.closePicker();
	}

	/** A list's own count, said the way it reads. It is what a screen reader is
	 *  told a list holds, so "1 recent feeds" is not good enough. */
	function tally(n: number, noun: string): string {
		return `${n} ${noun}${n === 1 ? '' : 's'}`;
	}

	/** Ask whatever the one input turns out to be asking. */
	function submit(event: SubmitEvent) {
		event.preventDefault();
		const asked = askedFor(value);
		// a new question is a new list, so neither the last answer's complaint nor
		// its end-of-list flag survives it
		unparseable = false;
		atEnd = false;
		if (asked.kind === 'feed') {
			// a real navigation and not a shallow push: this is a wall. The picker's
			// own history entry stays behind it, so back from the feed returns here
			// rather than out of mason.
			void goto(`/?feed=${encodeURIComponent(asked.ref)}`);
			return;
		}
		if (asked.kind === 'unparseable') {
			// nothing navigates, and the input says why. Laying the wall and letting
			// mortar reject it would answer the same question one page later, with
			// the picker gone and the value with it.
			unparseable = true;
			return;
		}
		if (asked.kind === 'creator') {
			void feeds.byCreator(asked.handle);
			return;
		}
		if (asked.kind === 'search') {
			void feeds.search(asked.term);
			return;
		}
		void feeds.browse();
	}

	/** The next page of whatever is showing. */
	async function more() {
		const before = feeds.results.length;
		await feeds.more();
		// `more()` is a no-op once a question has no cursor left, so a page that
		// added nothing IS the end of the list. The cursor is private to the rune
		// and this is the honest way to ask; without it the control would sit there
		// offering a page that never comes.
		atEnd = feeds.results.length === before;
	}

	// Escape closes, in the same language as the reader's. Bound to the document
	// so it works wherever focus has landed inside the picker, and attached only
	// while the picker is up (below), so it can never reach one already shut.
	const onKey = (event: KeyboardEvent) => {
		if (event.key === 'Escape') close();
	};

	// The resting state: what the network itself ranks, asked for once as the
	// picker opens. `browse()` returns early when popular results are already in
	// hand, so reopening the picker spends no round trip.
	//
	// untrack, because `browse()` reads `feeds.question` and `feeds.results`
	// synchronously to decide that, and an effect that tracked those reads would
	// re-run on the empty list `#ask` starts with and ask again forever.
	$effect(() => {
		if (!isOpen) return;
		// and the end-of-list flag goes with the question it was about. This
		// component is mounted once in +layout.svelte and never unmounts, so every
		// piece of its own state outlives the dialog: `atEnd` set on a cursorless
		// search was still set the next time the picker opened, over a fresh
		// popular list that DOES have a cursor, and "more feeds" stayed hidden for
		// the rest of the session. Reopening is a new question the same way
		// submitting one is, which is what the spec means by paging being forgotten
		// the moment the picker closes.
		atEnd = false;
		untrack(() => void feeds.browse());
	});

	// Everything the picker has to undo, keyed on `feeds.isOpen` and nothing
	// else: the back gesture shuts the picker without ever calling close(), so
	// the teardown hangs off the state rather than off a control.
	$effect(() => {
		if (!isOpen) return;
		// captured BEFORE anything below moves focus, which is why the capture and
		// the focus live in one effect rather than two: two would run in whatever
		// order they were created, and one that focused the input first would leave
		// this holding the input as the element to hand focus back to.
		const active = document.activeElement;
		const opener = active instanceof HTMLElement ? active : null;
		const root = document.documentElement;
		const restore = root.style.overflow;
		// the page must not scroll behind the picker; the panel does its own
		// scrolling
		root.style.overflow = 'hidden';
		document.addEventListener('keydown', onKey);
		// into the input, which is what the picker is for: a keyboard reader lands
		// in the field rather than at the top of the inert page behind it. After a
		// tick, because the binding is set as the DOM updates and this effect is
		// not guaranteed to run after that on the first flush.
		void tick().then(() => input?.focus());
		return () => {
			root.style.overflow = restore;
			document.removeEventListener('keydown', onKey);
			// back to the control that opened the picker. Both entry points make
			// sure that control still exists while the picker is up, so this lands
			// somewhere a reader can carry on from.
			opener?.focus();
		};
	});
</script>

<!-- the six standing in for a page in flight. Skeletons and not a spinner,
     because "nothing found" and "nothing yet" read identically and mean
     opposite things; aria-hidden, with the live region below doing the saying. -->
{#snippet skeletons()}
	<div class="mt-2 grid gap-2 sm:grid-cols-2" aria-hidden="true">
		{#each { length: PICKER_SKELETON_COUNT } as _, i (i)}
			<div
				class="flex animate-pulse items-start gap-3 rounded-card border-2 border-ink/10 bg-chalk p-3 dark:border-chalk/10 dark:bg-kiln"
			>
				<div class="size-10 shrink-0 rounded-lg bg-ink/10 dark:bg-chalk/10"></div>
				<div class="flex min-w-0 flex-1 flex-col gap-2">
					<div class="h-3.5 w-3/5 rounded bg-ink/10 dark:bg-chalk/10"></div>
					<div class="h-3 w-4/5 rounded bg-ink/10 dark:bg-chalk/10"></div>
				</div>
			</div>
		{/each}
	</div>
	<p class="sr-only" aria-live="polite">looking for feeds</p>
{/snippet}

<!-- the AppView would not answer. Quiet rather than fatal, and said in both the
     empty and the half-filled case: browsing rides an unspecced endpoint that
     carries no stability promise, while recents and the paste box are the
     load-bearing paths and neither goes through it. -->
{#snippet browsingOff()}
	<p class="mt-3 text-sm opacity-75">
		browsing is quiet right now. paste a feed link or an at:// uri and mason will still lay it.
	</p>
{/snippet}

{#if isOpen}
	<!-- the scrim: dims what is behind, and swallows every click so nothing back
	     there can be touched while the picker is up. Tapping it is a dismiss, in
	     the same language as the reader's. -->
	<button
		type="button"
		tabindex="-1"
		aria-label="Close the feed picker"
		onclick={close}
		class="fixed inset-0 z-40 animate-scrim-in cursor-default bg-ink/50 backdrop-blur-[2px] dark:bg-kiln-deep/70"
	></button>
	<!-- pointer-events-none, so the gutter around the panel is the scrim rather
	     than a dead zone: a click beside the panel lands on the button under it -->
	<div class="pointer-events-none fixed inset-0 z-50 grid place-items-center p-3 sm:p-6">
		<div
			role="dialog"
			aria-modal="true"
			aria-labelledby={titleId}
			class="pointer-events-auto max-h-full w-full max-w-2xl animate-reader-in overflow-y-auto overscroll-contain rounded-card border border-ink/10 bg-chalk p-4 text-left shadow-brick-lift sm:p-6 dark:border-chalk/15 dark:bg-kiln"
		>
			<div class="flex items-start justify-between gap-3">
				<div>
					<h2 id={titleId} class="font-display text-2xl leading-tight font-bold">pick a feed</h2>
					<p class="mt-1 text-sm opacity-75">
						any bluesky feed, laid as a mason wall. no login, nothing followed.
					</p>
				</div>
				<button
					type="button"
					onclick={close}
					aria-label="Close the feed picker"
					class="-mt-1 -mr-1 inline-flex min-h-11 min-w-11 shrink-0 cursor-pointer items-center justify-center rounded-full transition-transform motion-safe:hover:scale-105 motion-safe:active:scale-95"
				>
					<Icon name="x" class="size-5" />
				</button>
			</div>

			<!-- one field, three questions: what the reader typed decides which one
			     is asked (lib/feedref.ts). A handle here means "the feeds this person
			     made", which is the bridge between mason's two front doors. -->
			<form onsubmit={submit} class="mt-4 flex flex-col gap-2 sm:flex-row">
				<label class="sr-only" for="feed-query">
					Search feeds by name, by a creator's handle, or paste a feed link
				</label>
				<input
					id="feed-query"
					bind:this={input}
					bind:value
					oninput={() => (unparseable = false)}
					type="text"
					placeholder="a name, a handle, or a feed link"
					autocapitalize="none"
					autocorrect="off"
					spellcheck="false"
					aria-invalid={unparseable ? 'true' : undefined}
					aria-describedby={unparseable ? errorId : undefined}
					class="min-h-11 min-w-0 flex-1 rounded-full border-2 border-ink/20 bg-chalk px-5 py-2.5 font-semibold transition-colors focus:border-pop-pink dark:border-chalk/20 dark:bg-kiln"
				/>
				<button
					type="submit"
					class="min-h-11 shrink-0 cursor-pointer rounded-full bg-pop-pink-deep px-6 py-2.5 font-display font-bold text-white shadow-brick transition-transform max-sm:w-full motion-safe:hover:scale-105 motion-safe:active:scale-95"
				>
					find feeds
				</button>
			</form>
			{#if unparseable}
				<!-- in place, at the input, and nothing has navigated -->
				<p id={errorId} role="alert" class="mt-2 text-sm font-semibold text-live dark:text-live-bright">
					that is not a feed mason can lay. paste a bsky.app feed link, or an at:// uri ending in a
					feed generator.
				</p>
			{/if}

			{#if feeds.recent.length > 0}
				<!-- the one section that owes the network nothing, which is why it is
				     first: with the AppView unreachable this is still a wall away -->
				<section aria-labelledby={recentId} class="mt-6">
					<h3 id={recentId} class="text-xs font-bold tracking-wide uppercase opacity-60">recent</h3>
					<!-- keyed by uri and NOT by index, which is the opposite of the
					     results list below and has to stay that way. Taking a card
					     remembers its feed, and remembering moves it to the head of THIS
					     list; Svelte flushes in a microtask between one event listener
					     and the next, so with index keys the anchor the reader is
					     mid-click on is rebound to whichever feed now sits at that
					     position, href and all, before the browser resolves the
					     navigation. That laid the wrong wall for every card but the
					     first, silently, on a plain click, a keyboard Enter and a middle
					     click alike. A uri key MOVES the anchor instead of rebinding it,
					     which is what the switcher panel's copy of this list has always
					     done. `ordered()` keeps one entry per uri, so a duplicate key is
					     impossible here. -->
					<ul aria-label={tally(feeds.recent.length, 'recent feed')} class="mt-2 grid gap-2 sm:grid-cols-2">
						{#each feeds.recent as feed (feed.uri)}
							<li><FeedCard {feed} /></li>
						{/each}
					</ul>
				</section>
			{/if}

			<section aria-labelledby={resultsId} class="mt-6">
				<h3 id={resultsId} class="text-xs font-bold tracking-wide uppercase opacity-60">
					{resultsHeading}
				</h3>
				{#if feeds.loading && feeds.results.length === 0}
					{@render skeletons()}
				{:else if feeds.results.length > 0}
					<!-- keyed by index and NOT by uri: these are pages of somebody else's
					     directory concatenated, with no dedupe between them, and Svelte
					     answers a repeated key by THROWING each_key_duplicate as it
					     renders, which here would mean the picker going blank the moment
					     a second page repeated one feed.
					     THE RECENTS LIST ABOVE IS KEYED THE OTHER WAY, BY URI, and the
					     two are meant to disagree: do not "fix" the inconsistency. Each
					     list gets the only key that is safe for it. Recents are deduped
					     by `ordered()`, so a uri key cannot repeat there, and they
					     REORDER under their own cards, so an index key rebinds the
					     anchor being clicked and lays the wrong wall. Results are the
					     other way round: a uri can repeat, and nothing a results card
					     does touches `feeds.results`, so an index key costs nothing. -->
					<ul aria-label={tally(feeds.results.length, 'feed')} class="mt-2 grid gap-2 sm:grid-cols-2">
						{#each feeds.results as feed, i (i)}
							<li><FeedCard {feed} /></li>
						{/each}
					</ul>
					{#if feeds.browseUnavailable}
						{@render browsingOff()}
					{:else if !atEnd}
						<button
							type="button"
							onclick={more}
							disabled={feeds.loading}
							class="mt-3 inline-flex min-h-11 w-full cursor-pointer items-center justify-center gap-1 rounded-full border border-ink/15 px-4 text-sm font-semibold transition-colors hover:bg-ink/5 disabled:cursor-default disabled:opacity-50 dark:border-chalk/15 dark:hover:bg-chalk/10"
						>
							more feeds
							<Icon name="chevron-down" class="size-4" />
						</button>
					{/if}
				{:else if feeds.browseUnavailable}
					{@render browsingOff()}
				{:else if feeds.question === 'search'}
					<p class="mt-3 text-sm opacity-75">
						no feeds by that name. if you have the feed's link, paste it here and mason will lay it.
					</p>
				{:else if feeds.question === 'creator'}
					<p class="mt-3 text-sm opacity-75">that person has not made any feeds.</p>
					<!-- the other front door, offered rather than assumed: somebody who
					     typed a handle here still wants that person -->
					<a
						href="/?actor={encodeURIComponent(feeds.term)}"
						class="mt-1 inline-flex min-h-11 items-center font-semibold text-brick-post-ink hover:underline dark:text-brick-post"
					>
						lay @{feeds.term}'s wall instead
					</a>
				{:else}
					<p class="mt-3 text-sm opacity-75">
						no feeds to show. paste a feed link and mason will lay it.
					</p>
				{/if}
			</section>
		</div>
	</div>
{/if}
