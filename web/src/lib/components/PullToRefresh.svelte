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
	import { pull, PULL_SETTLE_MS, PULL_THRESHOLD } from '$lib/state/pull.svelte';
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

	/** Is this pointer over a screen that sits on top of the wall?
	 *
	 *  The WHEEL's version of the question above, and it is deliberately the
	 *  looser one, because the two inputs mean different things by "where". A
	 *  finger grabs the thing under it, so a pull must start on the wall. A wheel
	 *  only points: the cursor sits wherever it was last left, very often over the
	 *  bar, which on a desktop is stuck to the top of the wall and scrolls
	 *  nothing. Requiring the pointer to be over the wall there would make the
	 *  gesture fail for a reason no reader could see.
	 *
	 *  What a wheel must not do is pull the wall out from under a screen that has
	 *  its own scroll. `blocked` covers the three that own the viewport; this
	 *  covers the switcher's panel and the scrim under it, which are a dialog and
	 *  a labelled button rather than anything `blocked` can see. */
	function overAnOverlay(target: EventTarget | null): boolean {
		return (
			target instanceof Element &&
			target.closest('[role="dialog"], [aria-label="Close switch panel"]') !== null
		);
	}

	/** May a gesture start here? Everything `PullState` cannot see, answered once,
	 *  when a gesture begins. Both inputs ask the same question, because "the wall
	 *  is at its top, nobody is laying it, nothing is over it, and the reader is
	 *  on the wall" is not a claim about fingers.
	 *
	 *  `scrollY <= 0` and not `=== 0`: an iOS document that is already
	 *  rubber-banding reports a negative offset, and that reader is at the top
	 *  of the wall by any honest reading of where they are. */
	function ready(event: Event): boolean {
		// Where the top of the wall is, in the viewport, before it moves. Zero on a
		// phone, where the control bar is at the BOTTOM and the wall starts at the
		// top of the screen; the height of the bar on a desktop, where the bar is
		// sticky above the wall. The indicator hangs off this rather than off the
		// viewport, so it rides in the gap the wall opens instead of over the
		// controls, and one measurement answers for both layouts without either
		// naming a breakpoint or a bar height.
		//
		// Measured only when a gesture is not already running: mid-pull the wall is
		// translated and its own rect is exactly what this must not read.
		if (!pull.pulling) wallTop = document.querySelector('#wall')?.getBoundingClientRect().top ?? 0;
		return !blocked && !busy && window.scrollY <= 0;
	}

	function onStart(event: TouchEvent) {
		const touch = event.touches[0];
		if (!touch) return;
		// the one-finger rule is touch's alone: a second finger is a pinch, and a
		// wheel has no equivalent to be wrong about
		pull.start(
			{ x: touch.clientX, y: touch.clientY },
			event.touches.length === 1 && ready(event) && onTheWall(event.target),
		);
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

	/** Lay the wall again, whichever gesture asked. Both releases end here, so
	 *  there is one refresh with one set of obligations rather than two that have
	 *  to be kept in step. */
	function lay() {

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

	/** The finger lifted. */
	function onEnd() {
		if (pull.release()) lay();
	}

	/** A line of text, in px, for a wheel that reports lines rather than pixels.
	 *  Firefox does, and 16 is its own default line height; the exact number only
	 *  decides how many notches a pull takes there. */
	const LINE = 16;

	/** The wheel's delta in pixels, whatever unit it arrived in. Done here rather
	 *  than in the rune module because page-mode needs a viewport, which is a DOM
	 *  global that module deliberately does not name. */
	function wheelPixels(event: WheelEvent): number {
		if (event.deltaMode === 1) return event.deltaY * LINE;
		if (event.deltaMode === 2) return event.deltaY * window.innerHeight;
		return event.deltaY;
	}

	/** The wheel stopped, which is a desktop reader letting go. */
	let settleTimer: ReturnType<typeof setTimeout> | undefined;

	/** The viewport offset of the wall's own top edge; see `ready`. */
	let wallTop = $state(0);

	function onWheel(event: WheelEvent) {
		const allowed = ready(event) && !overAnOverlay(event.target);
		const claimed = pull.wheel(wheelPixels(event), event.timeStamp, allowed);
		// cleared on EVERY wheel event, claimed or not: the timer means "the wheel
		// has stopped", so any wheel event at all is evidence that it has not.
		clearTimeout(settleTimer);
		if (!claimed) return;
		settleTimer = setTimeout(() => {
			if (pull.settle()) lay();
		}, PULL_SETTLE_MS);
	}

	// addEventListener rather than `<svelte:window ontouchmove={...}>`, for one
	// reason that decides it: a touch listener on window is PASSIVE by default in
	// every browser that matters, a passive listener's preventDefault is ignored,
	// and svelte's event attributes cannot pass listener options. The wall would
	// move under a document that scrolled anyway.
	//
	// The wheel listener is PASSIVE on purpose, and it is the one place this file
	// declines to take the event from the browser. A non-passive wheel listener on
	// window makes every scroll on the wall wait for this handler before it
	// paints, on a surface whose whole job is scrolling. There is nothing to
	// prevent anyway: at the top of the wall an upward wheel scrolls nothing, and
	// the browser's own overscroll is already off (`overscroll-behavior-y` in
	// app.css). Where a platform still bounces the document under us, the pull
	// rides that bounce rather than replacing it, exactly as it does on iOS.
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
		window.addEventListener('wheel', onWheel, passive);
		return () => {
			window.removeEventListener('touchstart', onStart);
			window.removeEventListener('touchmove', onMove);
			window.removeEventListener('touchend', onEnd);
			window.removeEventListener('touchcancel', onCancel);
			window.removeEventListener('wheel', onWheel);
			clearTimeout(settleTimer);
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
		class="pointer-events-none fixed inset-x-0 z-30 flex justify-center"
		style="top: {wallTop}px; transform: translateY({offset}px); opacity: {opacity};"
	>
		<div
			class="flex items-center gap-2 rounded-full border border-ink/10 bg-chalk/95 px-4 py-2 font-display text-sm font-bold shadow-brick backdrop-blur-sm dark:border-chalk/15 dark:bg-kiln/95"
		>
			<!-- the same icon the header control carries, because it is the same
			     refresh; it only spins once the wall is actually being laid, and
			     never for a reader who asked for less motion -->
			<Icon name="rotate-cw" class="size-4 {pull.laying ? 'motion-safe:animate-spin' : ''}" />
			<!-- one word changes with the input, because the commitment does: a finger
			     lets go and a wheel stops. Saying "let go" to somebody holding no
			     mouse button is an instruction they cannot follow. -->
			<span>
				{#if pull.laying}
					laying bricks
				{:else if pull.armed}
					{pull.by === 'wheel' ? 'stop to lay again' : 'let go to lay again'}
				{:else}
					pull to lay again
				{/if}
			</span>
		</div>
	</div>
{/if}
