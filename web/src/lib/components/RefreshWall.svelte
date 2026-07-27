<script lang="ts">
	// Lay this wall again, now. One control in the header, no confirmation and no
	// count: a wall re-lays because the reader asked for it, never on its own.
	//
	// It calls `feed.refresh()` and closes an open reader, and it does nothing
	// else at all. Every decision about what a refresh IS lives in
	// `state/feed.svelte.ts`, which is typechecked and unit tested; a `.svelte`
	// body is neither, so what is left here is one button, one disabled state and
	// the comments below.
	import { feed } from '$lib/state/feed.svelte';
	import { reader } from '$lib/state/reader.svelte';
	import Icon from './Icon.svelte';

	// The whole rate limit, and the reason `disabled` below is a real attribute
	// rather than a styled-off look: one refresh costs a hundred-author AppView
	// fan-out, spent from the reader's own budget in local mode, and there is no
	// server-side throttle by design (a client-side engine has nowhere to keep
	// per-viewer state, and a shared server should not keep it on an
	// unauthenticated read). A control that only LOOKED disabled would still take
	// the click, and it would tell a screen-reader reader the wall is ready to be
	// laid again while it is already being laid.
	//
	// The same two states `feed.refresh()` refuses in, said where the reader can
	// see it rather than a second rule: a refused refresh is silent by design, so
	// without this the control would swallow taps with no explanation. `done` is
	// deliberately not here either, for the same reason `refresh()` leaves it
	// out: a wall that ran out of bricks is exactly the one worth asking again.
	const busy = $derived(feed.loading || feed.warming);

	function layAgain() {
		// CLOSE THE READER FIRST. A refresh replaces `feed.items` wholesale and
		// the reader locates its open brick in that list by id, so a reader left
		// up would be holding a brick that is no longer on the wall, with both
		// step controls dead. Both merged change specs hand that obligation to
		// whichever of them lands second.
		//
		// It is discharged HERE, at the trigger, rather than inside `FeedState`:
		// `reader.svelte.ts` already imports `feed.svelte.ts`, so the reverse
		// import is a cycle between two singletons, it would drag
		// `$app/navigation` and `$app/state` into the feed module's graph (which
		// is what keeps that module runnable for real in node under vitest), and
		// `reader.close()` reaches `history.back()` one module away, where
		// feed.test.ts's "names no DOM global" grep could not see it.
		//
		// Today no click can reach this line with a reader up: an open reader
		// makes the layout's content wrapper `inert`, this control sits inside
		// that wrapper, and this control is the only trigger a refresh has. That
		// makes the call a guarantee for the NEXT trigger rather than a live path
		// (a keyboard shortcut, a pull gesture, an auto-refresh nobody has asked
		// for), and none of those would be inside the wrapper. It is not dead
		// code, it is the state's invariant held at the one place that can hold it.
		//
		// Guarded on `isOpen`, and the guard is load-bearing rather than tidiness:
		// `close()` pops history only for an entry IT pushed and clears that flag
		// only inside itself, while the back gesture shuts the reader without ever
		// calling it. So a `close()` on an already-shut reader pops a SECOND entry
		// and leaves the wall altogether. `BrickReader`'s own teardown declines to
		// call `close()` for exactly this reason.
		if (reader.isOpen) reader.close();

		// AND NO SCROLL, of any kind: no `scrollTo`, no `scrollIntoView`, no
		// anchor jump. That is a choice rather than an omission, and it is worth
		// saying which, because the mechanics used to make the choice for us.
		//
		// `refresh()` flips `warming` true synchronously; `FeedGrid`'s freeze
		// effect re-runs on a microtask and arms its `{passive, once}` scroll
		// listener; and the `scroll` event `window.scrollTo` queues is delivered
		// AFTER that microtask. `freeze()` turns a commit away when the wall is not
		// warming or a page is already in flight, and a refresh sets neither, so in
		// EITHER call order the wall committed on its own programmatic scroll, with
		// no reflow at all. The coupling was event delivery, not ordering, which is
		// why no arrangement of these two lines ever fixed it.
		//
		// `FeedState`'s in-flight marker removed that: `freeze()` returns early
		// while a refresh's own flagged cursorless request is out, whoever
		// triggered it, so a scroll during a refresh can no longer commit
		// anything. What is left is the reason to still not scroll: the outgoing
		// wall stays on screen and reflows into the new one, and that is a thing
		// the reader has to be looking at to see. Jumping them to the top is not
		// wrong, it just throws away the reflow they asked for. Not scrolling
		// removes the coupling rather than timing around it.
		//
		// Under `prefers-reduced-motion: reduce` there is no scroll event in it at
		// all: `FeedGrid` calls `feed.freeze()` the instant `warming` flips true,
		// with no listener attached and nothing to wait for. That freeze is held
		// rather than sent, because the in-flight marker is set while the refresh's
		// flagged cursorless preview is out, so exactly ONE cursorless request
		// goes out under a refresh; `#warm` then adopts that preview's cursor,
		// which carries the refreshing snapshot's seed, and freezes from there, so
		// the request that COMMITS lands on the refreshed wall. One refreshed
		// fan-out, and one reflow when the preview lands rather than a commit
		// before the wall has moved.
		feed.refresh();
	}

	// DECISION, so the next reader does not rediscover it: this control adds no
	// live region, and `FeedGrid` gets no refresh-aware branch either. The wall
	// has exactly ONE polite region, it already says "laying bricks" while
	// `warming`, and a refresh IS a warm, so it is already announcing the right
	// thing. A second region would talk over the first, and 08's accessibility
	// section states there is one region for the whole wall. The reflow's churn
	// is suppressed for free too: that region resets its count while warming, so
	// bricks re-laid in a new arrangement never read as new bricks.
</script>

<!-- A plain <button>, so the disabled state is the platform's rather than a
     look: a screen-reader reader is told the wall is already being laid instead
     of pressing a control that silently does nothing. The name is `sr-only`
     because the bar never wraps on mobile and every control has to earn its
     width at 375px; the icon carries it visually, and `min-h-11` keeps the touch
     target at 44px whatever the icon does. It is the narrowest control on the
     bar (px-2, so 36px) for the same reason: this was a full row before it
     arrived, and the pickers beside it are carrying labels a first-time visitor
     needs. -->
<button
	type="button"
	onclick={layAgain}
	disabled={busy}
	class="inline-flex min-h-11 shrink-0 cursor-pointer items-center justify-center rounded-full px-2 transition-colors hover:bg-ink/5 disabled:cursor-default disabled:opacity-35 disabled:hover:bg-transparent dark:hover:bg-chalk/10"
>
	<Icon name="rotate-cw" class="size-5" />
	<span class="sr-only">lay this wall again</span>
</button>
