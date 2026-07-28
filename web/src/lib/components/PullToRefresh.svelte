<script lang="ts">
	// Pull the top of the wall down and let go: the second trigger for the one
	// refresh `RefreshWall` already asks for.
	//
	// This file listens and draws, and decides nothing. Which drags are pulls,
	// how far the band stretches and when it arms all live in
	// `state/pull.svelte.ts`, which is typechecked and unit tested; a `.svelte`
	// body is neither. What is left here is the one question that module cannot
	// answer (may a gesture start at all), the listeners, and the indicator.
	import { feed } from '$lib/state/feed.svelte';
	import { pull, PULL_THRESHOLD } from '$lib/state/pull.svelte';
	import { reader } from '$lib/state/reader.svelte';
	import Icon from './Icon.svelte';

	// Whether a screen is up over the wall. Passed in rather than re-derived
	// here, because the layout already owns that expression and one wrong copy
	// of it is a pull that lays the wall behind an open reader. It is a real
	// input and not belt-and-braces: `inert` on the layout's wrapper stops
	// clicks and focus, and stops neither of the window listeners below.
	let { blocked = false }: { blocked?: boolean } = $props();

	// The same two states the button refuses in, for the same reason: a refresh
	// is a hundred-author AppView fan-out spent from the reader's own budget in
	// local mode, and the trigger is the whole rate limit. `feed.refresh()`
	// refuses on its own too, but a gesture that started and travelled and did
	// nothing on release is worse than one that never took the finger.
	const busy = $derived(feed.loading || feed.warming);

	// The warm a pull started is over, so let the wall down off its shelf and put
	// the indicator away. `pull.laying` is set by the release below and cleared
	// only here, which makes the wall closing the signal that the refresh
	// finished: the reader watches the gap shut rather than reading a word.
	//
	// The effect reads `feed.warming` and writes `pull.laying`, and never reads
	// `pull.laying`: an effect that reads the state it writes is a cycle svelte
	// aborts the whole flush over, and an aborted flush leaves the DOM holding
	// the last thing written to it. That is not hypothetical, it is what the
	// first cut of this file did, and the symptom was the wall staying pulled
	// down after the finger had gone.
	$effect(() => {
		if (!feed.warming) pull.laying = false;
	});

	/** Did this touch land on the wall itself?
	 *
	 *  To pull the wall you have to have hold of the wall, and a window listener
	 *  hears every touch on the page: the header bar, an open switcher panel and
	 *  the scrim under it all sit over the wall and all bubble here. `blocked`
	 *  covers the three screens that own the whole viewport, and this covers
	 *  everything else, including the switcher, whose openness is local component
	 *  state that nothing outside it can read.
	 *
	 *  Asked of the element the touch STARTED on, which is the only honest
	 *  question: a gesture belongs to whatever it began on, and a finger that
	 *  wanders off the wall mid-drag is still pulling it. */
	function onTheWall(target: EventTarget | null): boolean {
		return target instanceof Element && target.closest('#wall') !== null;
	}

	/** May a gesture start here? Everything `PullState` cannot see, answered
	 *  once, at touch down.
	 *
	 *  `scrollY <= 0` and not `=== 0`: an iOS document that is already
	 *  rubber-banding reports a negative offset, and that reader is at the top
	 *  of the wall by any honest reading of where they are. */
	function ready(event: TouchEvent): boolean {
		return (
			event.touches.length === 1 &&
			!blocked &&
			!busy &&
			window.scrollY <= 0 &&
			onTheWall(event.target)
		);
	}

	function onStart(event: TouchEvent) {
		const touch = event.touches[0];
		if (!touch) return;
		pull.start({ x: touch.clientX, y: touch.clientY }, ready(event));
	}

	function onMove(event: TouchEvent) {
		// a second finger means a pinch or a zoom, which is not this gesture and
		// must not end in a fan-out
		if (event.touches.length > 1) {
			pull.cancel();
			return;
		}
		const touch = event.touches[0];
		if (!touch) return;
		if (!pull.move({ x: touch.clientX, y: touch.clientY })) return;
		// The move belongs to the wall, so take it from the browser: without this
		// the document would scroll (or bounce) under a wall that is already
		// moving, and the two would add up.
		//
		// `cancelable` is checked rather than assumed: once a scroll is under way
		// iOS hands over uncancelable moves, and calling preventDefault on one is
		// a console warning and no effect. The pull still tracks the finger
		// there, riding on the platform's own rubber band instead of replacing it.
		if (event.cancelable) event.preventDefault();
	}

	function onEnd() {
		if (!pull.release()) return;

		// CLOSE THE READER FIRST, the same obligation `RefreshWall` holds and for
		// the same reason: a refresh replaces `feed.items` wholesale and the
		// reader locates its open brick in that list by id, so a reader left up
		// would hold a brick that is no longer on the wall. Unlike the button,
		// this trigger is NOT inside the layout's inert wrapper (a window
		// listener has no wrapper), so the guard is a live path here rather than
		// a guarantee held for the next trigger. `blocked` is what keeps a
		// gesture from starting under an open reader; this is what keeps the
		// invariant if it ever does.
		if (reader.isOpen) reader.close();

		feed.refresh();
		// AFTER the call, and read back off the wall rather than assumed:
		// `refresh()` flips `warming` synchronously when it takes the ask and
		// refuses without side effects when it does not (a page already in
		// flight, a wall that started warming under the gesture). So this is
		// "the refresh took", and a wall propped open over a refresh nobody
		// started would have nothing to let it back down.
		pull.laying = feed.warming;
	}

	// addEventListener rather than `<svelte:window ontouchmove={...}>`, for one
	// reason that decides it: a touch listener on window is PASSIVE by default in
	// every browser that matters, a passive listener's preventDefault is ignored,
	// and svelte's event attributes cannot pass listener options. The wall would
	// move under a document that scrolled anyway.
	//
	// Attached once, with no reactive read in the effect body: the handlers read
	// `blocked` and `busy` when they run, so re-arming the listeners on every
	// state change would cost a teardown per brick laid and buy nothing.
	// named, not an inline arrow: removeEventListener matches on identity, and a
	// second `() => pull.cancel()` is a different function that removes nothing
	const onCancel = () => pull.cancel();

	$effect(() => {
		const passive = { passive: true } as const;
		window.addEventListener('touchstart', onStart, passive);
		window.addEventListener('touchmove', onMove, { passive: false });
		window.addEventListener('touchend', onEnd, passive);
		window.addEventListener('touchcancel', onCancel, passive);
		return () => {
			window.removeEventListener('touchstart', onStart);
			window.removeEventListener('touchmove', onMove);
			window.removeEventListener('touchend', onEnd);
			window.removeEventListener('touchcancel', onCancel);
		};
	});

	/** The pill's own height in px: 20px of line, 16px of padding, 2px of border.
	 *  A nominal figure rather than a measured one, because it positions an
	 *  affordance: a font that renders a pixel taller costs a pixel of air. */
	const PILL = 38;

	/** The air between the bottom of the pill and the top of the wall. It has to
	 *  be spent deliberately or it is not spent at all: the first version left
	 *  the two 2px apart, which is a gap that exists in the arithmetic and
	 *  nowhere on the screen. `PULL_LAYING` is this twice plus the pill, so the
	 *  shelf holds the same air above the pill as below it. */
	const PILL_GAP = 8;

	// The indicator hangs off the BOTTOM of the gap the wall has opened, clear of
	// its edge, at every stage and by one rule. Early in a pull that puts it
	// above the top edge, so it emerges from behind it as the gap grows; from the
	// threshold on it sits clear inside the gap; and while the wall rests on its
	// shelf it sits in that, which is the whole reason the shelf is there. One
	// expression, so the pill and the wall can never disagree about where the top
	// of the wall is.
	const offset = $derived(pull.offset - PILL - PILL_GAP);
	// fades in over the first half of the pull, so the affordance is legible
	// well before the threshold it is describing
	const opacity = $derived(pull.laying ? 1 : Math.min(1, pull.distance / (PULL_THRESHOLD / 2)));
</script>

<!-- aria-hidden, and that is a decision rather than an oversight: the wall has
     exactly ONE polite live region, it already says "laying bricks" while
     warming, and a pull ends in a warm. A second region here would talk over
     the first, and this is a touch affordance narrating a touch gesture that a
     screen-reader reader is not making. The button remains their way to lay the
     wall again, named and disabled by the platform.
     pointer-events-none so it can never take a tap meant for a brick under it. -->
{#if pull.offset > 0}
	<div
		aria-hidden="true"
		class="pointer-events-none fixed inset-x-0 top-0 z-30 flex justify-center"
		style="transform: translateY({offset}px); opacity: {opacity};"
	>
		<div
			class="flex items-center gap-2 rounded-full border border-ink/10 bg-chalk/95 px-4 py-2 font-display text-sm font-bold shadow-brick backdrop-blur-sm dark:border-chalk/15 dark:bg-kiln/95"
		>
			<!-- the same icon the header control carries, because it is the same
			     refresh; it only spins once the wall is actually being laid, and
			     never for a reader who asked for less motion -->
			<Icon name="rotate-cw" class="size-4 {pull.laying ? 'motion-safe:animate-spin' : ''}" />
			<span>
				{#if pull.laying}
					laying bricks
				{:else if pull.armed}
					let go to lay again
				{:else}
					pull to lay again
				{/if}
			</span>
		</div>
	</div>
{/if}
