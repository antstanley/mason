<script lang="ts">
	// Whoever's wall this is, their face doubles as a switcher. Clicking it does
	// NOT leave the current wall; it drops a small form below the button. Nothing
	// re-renders until a DIFFERENT handle is submitted, so opening the panel and
	// thinking better of it leaves the wall exactly where it was.
	import { tick } from 'svelte';
	import { goto } from '$app/navigation';
	import { feedInfo } from '$lib/state/feedinfo.svelte';
	import { feeds } from '$lib/state/feeds.svelte';
	import { cleanHandle, lastHandle } from '$lib/state/handle.svelte';
	import { profile } from '$lib/state/profile.svelte';
	import Icon from './Icon.svelte';

	// Both spellings of a wall, and neither is required: the header takes what
	// the URL has. A graph wall shows the owner's face, a feed wall shows the
	// generator's, and with both parameters present the feed is what is laid,
	// so the feed is what the button names.
	let { actor, feed }: { actor: string | null; feed: string | null } = $props();

	/** How many recents this panel lists. The picker itself keeps twelve, which is
	 *  a screen of cards there and a list past the fold here: this panel opens
	 *  upward from a fixed bar on a phone, so every row it adds is a row nearer
	 *  the top of the viewport. Five is what fits at 375px without the panel
	 *  needing to scroll, and the picker is one tap away for the rest.
	 *
	 *  The panel scrolls if it ever has to (see its `max-h`), which is the
	 *  backstop rather than the plan: the door is the first thing in it now, so a
	 *  panel too tall for its screen would clip the primary action, and the
	 *  bottom-anchored panel grows upward. */
	const PANEL_RECENT_FEEDS = 5;

	const panelRecent = $derived(feeds.recent.slice(0, PANEL_RECENT_FEEDS));

	let open = $state(false);
	let root = $state<HTMLElement | null>(null);
	let trigger = $state<HTMLButtonElement | null>(null);
	let door = $state<HTMLButtonElement | null>(null);
	let value = $state('');

	const onFeed = $derived(!!feed);

	// the face on the button, from whichever identity this wall has. Neither
	// load blocks anything: both leave a fallback in place and fill it in if and
	// when the AppView answers.
	$effect(() => {
		if (feed) {
			feedInfo.load(feed);
		} else {
			profile.load(actor ?? '');
		}
	});

	// what the button says it is showing, and the initial it falls back to when
	// there is no face yet
	const wallName = $derived(onFeed ? feedInfo.name : `@${actor ?? ''}`);
	const wallFace = $derived(onFeed ? feedInfo.avatar : profile.avatar);
	// A feed's own name is not unique (two "Discover" feeds are ordinary), so the
	// creator's handle is what tells a screen-reader reader which one this is.
	// Before that lands, wallName is the reference's rkey, which is at least
	// something the reader can match against the link they followed.
	const switchLabel = $derived(
		onFeed
			? `Switch wall, currently viewing the ${wallName} feed${feedInfo.creator ? ` by @${feedInfo.creator}` : ''}`
			: `Switch wall, currently viewing ${wallName}`
	);

	function openPanel() {
		open = true;
		// prefill the reader's own remembered handle, so switching back to your
		// own wall is still a single tap once they reach the field
		value = lastHandle.value;
		// FOCUS THE DOOR TO THE PICKER, not the handle field.
		//
		// This panel is opened by somebody who wants a different wall, and a feed
		// is the likelier one: a reader has one handle graph and thousands of
		// feeds, and the recents below are feeds too. Focusing the field also
		// opened the keyboard on every phone that opened this panel, over a panel
		// whose top half is a list you were probably reaching for, and it put the
		// caret in the one control that answers a question most opens are not
		// asking.
		//
		// Focus has to land INSIDE the dialog either way, so this is a choice
		// about which control, not whether. The field is one Tab away.
		void tick().then(() => door?.focus());
	}

	function closePanel(returnFocus = true) {
		open = false;
		if (returnFocus) trigger?.focus();
	}

	// A link inside the panel has been taken, so the panel comes down with it.
	//
	// The panel's openness is local state and a client-side navigation does not
	// touch it: without this, the dialog and its full-viewport scrim stay mounted
	// over the wall that has just laid, dimming it and swallowing every click,
	// and a screen-reader reader is left inside a dialog called "Switch wall"
	// over a wall that already switched. The submit path never had that problem
	// because it closes before it navigates, and the door to the picker closes
	// first for its own reason; these links are the two that did not.
	//
	// Nothing here calls preventDefault. They stay real links, so a middle click
	// or a cmd click still opens a wall in a new tab, and that is exactly why a
	// MODIFIED click leaves the panel up: it opened a wall somewhere else and
	// this one has not moved, so closing would shut the switcher on somebody who
	// is reaching for a second feed. Same rule, same spelling as the reader's own
	// activation (state/reader.svelte.ts).
	//
	// A middle click never reaches here at all, because no browser dispatches
	// `click` for a non-primary button: it fires auxclick, which only the recents
	// link listens for and only to remember the feed. So the panel stays up for
	// that one through absence rather than through the button clause, and the
	// clause is kept anyway so this predicate is the reader's rule spelled once
	// rather than a subset of it.
	//
	// And it closes without taking focus back. The trigger is where focus belongs
	// after a DISMISSAL, which changes nothing behind the panel, but this is a
	// navigation: SvelteKit resets focus to the top of the page it has just laid,
	// so grabbing the trigger first is a move undone a moment later, announcing a
	// control the reader is leaving rather than the wall they asked for.
	function closeOnNavigation(event: MouseEvent) {
		if (event.metaKey || event.ctrlKey || event.shiftKey || event.altKey || event.button !== 0) {
			return;
		}
		closePanel(false);
	}

	// Tab out of the dialog and it closes, so focus never walks into the dimmed
	// wall behind (mirrors the Escape and click-away dismissals). Only a move that
	// lands outside the whole switcher counts; hops between the fields do not.
	function onFocusOut(event: FocusEvent) {
		const next = event.relatedTarget as Node | null;
		if (next && root && !root.contains(next)) closePanel(false);
	}

	function submit(event: SubmitEvent) {
		event.preventDefault();
		const handle = cleanHandle(value);
		if (!handle) return;
		open = false;
		// same wall: close and change nothing, so the page never re-renders on a
		// no-op switch
		if (handle === actor) return;
		lastHandle.remember(handle);
		void goto(`/?actor=${encodeURIComponent(handle)}`);
	}

	$effect(() => {
		if (!open) return;
		const onDown = (event: PointerEvent) => {
			if (root && !root.contains(event.target as Node)) closePanel(false);
		};
		const onKey = (event: KeyboardEvent) => {
			if (event.key === 'Escape') closePanel();
		};
		document.addEventListener('pointerdown', onDown);
		document.addEventListener('keydown', onKey);
		return () => {
			document.removeEventListener('pointerdown', onDown);
			document.removeEventListener('keydown', onKey);
		};
	});
</script>

<!-- `relative` only from md. Below it the panel is positioned against the fixed
     bar instead (see its own comment), which is what keeps a 320px panel on
     screen when the trigger it hangs off is not the right-most control. -->
<div bind:this={root} class="md:relative">
	<button
		bind:this={trigger}
		type="button"
		onclick={() => (open ? closePanel(false) : openPanel())}
		aria-haspopup="dialog"
		aria-expanded={open}
		aria-label={switchLabel}
		class="inline-flex min-h-11 min-w-11 cursor-pointer items-center justify-center overflow-hidden rounded-full bg-pop-pink-quiet p-0.5 font-semibold text-white shadow-brick transition-transform motion-safe:hover:scale-105 motion-safe:active:scale-95 sm:min-h-9 sm:min-w-0 sm:justify-start sm:gap-2 sm:pr-4"
	>
		{#if wallFace}
			<img src={wallFace} alt="" class="size-8 shrink-0 rounded-full object-cover" />
		{:else}
			<span
				class="grid size-8 shrink-0 place-items-center rounded-full bg-white/20 text-base font-bold uppercase"
				aria-hidden="true"
			>
				{(onFeed ? feedInfo.name : (actor ?? '?')).slice(0, 1) || '?'}
			</span>
		{/if}
		<!-- the name is already spelled with its @ on a graph wall; a feed's is a
		     display name, and prefixing one would read as a handle -->
		<span class="hidden max-w-[10rem] truncate sm:inline">{wallName}</span>
		<Icon name="arrow-left-right" class="hidden size-4 shrink-0 opacity-80 sm:block" />
	</button>

	{#if open}
		<!-- a filter over the wall: dims it in light mode, veils it in dark, and
		     swallows every click so the content behind cannot be touched while the
		     switcher is up. tapping it is a dismiss. -->
		<button
			type="button"
			tabindex="-1"
			aria-label="Close switch panel"
			onclick={() => closePanel()}
			class="fixed inset-0 z-30 cursor-default bg-ink/35 backdrop-blur-[2px] dark:bg-chalk/15"
		></button>
		<!-- Below md this is positioned against the bar, which is `fixed` and so is
		     the nearest positioned ancestor once the wrapper stops being
		     `relative`: `inset-x-4` spans the viewport with a gutter either side,
		     whatever the trigger's own position is. Anchoring it to the trigger with
		     `right-0` only worked while the switcher was the right-most control; the
		     moment it moved left of refresh and settings, a 320px panel hung off the
		     left edge of a 375px screen and clipped its own input. From md the header
		     is not fixed, the wrapper is `relative` again, and the panel hangs off the
		     trigger as before. -->
		<div
			role="dialog"
			aria-modal="true"
			aria-label="Switch wall"
			onfocusout={onFocusOut}
			class="absolute inset-x-4 bottom-full z-40 mb-2 max-h-[70vh] overflow-y-auto overscroll-contain rounded-2xl border border-ink/10 bg-chalk p-5 text-left shadow-brick-lift md:inset-x-auto md:right-0 md:top-full md:bottom-auto md:mt-2 md:mb-0 md:w-80 dark:border-chalk/15 dark:bg-kiln"
		>
			<!-- THE FIRST THING IN THE PANEL, and the one filled control in it.
			     Somebody opening the switcher wants another wall, and a feed is the
			     likelier one: they have one follow graph and thousands of feeds, and
			     the recents under this are feeds as well. The handle box is still
			     here, below, where a reader who came for it will find it, and the
			     panel takes focus HERE rather than in the field (see openPanel).

			     The panel closes FIRST, which hands focus back to the switcher's own
			     button: this control unmounts with the panel, and the picker returns
			     focus to whatever held it, so that has to be something still on the
			     page. -->
			<button
				bind:this={door}
				type="button"
				onclick={() => {
					closePanel();
					feeds.openPicker();
				}}
				class="inline-flex min-h-11 w-full cursor-pointer items-center justify-center rounded-full bg-pop-pink-deep px-4 font-display text-sm font-bold text-white shadow-brick transition-transform motion-safe:hover:scale-105 motion-safe:active:scale-95"
			>
				pick a feed to lay
			</button>
			<!-- The feeds this reader has opened before, which is the fastest way back
			     to one and the reason the panel is worth opening at all on a phone.
			     They sit directly under the door because they answer the same
			     question it does, one tap sooner. Capped at PANEL_RECENT_FEEDS rather
			     than showing the whole dozen: this panel grows UPWARD from a fixed bar
			     on a small screen, so every row it adds is a row nearer the top of the
			     viewport, and the door at the top is what a panel too tall for its
			     screen would take with it.

			     Real links with real hrefs, in the same language as a brick's and as
			     FeedCard's, so a middle click or a cmd click opens a wall in a new tab
			     instead of being swallowed. Remembering the feed moves it back to the
			     front of the list, exactly as choosing one in the picker does; without
			     it, opening the same feed from here would leave the order it was in.
			     It runs on EVERY activation the browser reports, modified or not, for
			     the same reason FeedCard's does: a feed opened in a background tab is
			     still a feed this reader opened. That takes TWO listeners rather than
			     one, because a middle click dispatches auxclick and no click at all;
			     `feeds.rememberFromLink` holds the rule for which buttons count, so
			     this link and the picker's cards cannot drift apart on it.

			     Taking the panel down is the half that is only right for the click
			     which replaces this wall, so that half lives in closeOnNavigation and
			     hangs off the click alone: a middle click opened the feed elsewhere
			     and this wall has not moved. -->
			{#if panelRecent.length > 0}
				<div class="mt-4 border-t border-ink/10 pt-4 dark:border-chalk/10">
					<h2 class="text-xs font-semibold opacity-75">recent feeds</h2>
					<ul class="mt-2 flex flex-col">
						{#each panelRecent as feed (feed.uri)}
							<li>
								<a
									href="/?feed={encodeURIComponent(feed.uri)}"
									onclick={(event) => {
										feeds.rememberFromLink(event, feed);
										closeOnNavigation(event);
									}}
									onauxclick={(event) => feeds.rememberFromLink(event, feed)}
									class="flex min-h-11 items-center gap-2 rounded-xl px-2 text-sm font-semibold transition-colors hover:bg-ink/5 dark:hover:bg-chalk/10"
								>
									{#if feed.avatar}
										<img
											src={feed.avatar}
											alt=""
											class="size-6 shrink-0 rounded-md object-cover"
										/>
									{:else}
										<span
											aria-hidden="true"
											class="size-6 shrink-0 rounded-md bg-ink/10 dark:bg-chalk/15"
										></span>
									{/if}
									<span class="truncate">{feed.name}</span>
								</a>
							</li>
						{/each}
					</ul>
				</div>
			{/if}

			<!-- The other spelling of a wall, kept and demoted rather than moved out:
			     a handle is still how you reach a person's wall, it is just not what
			     most opens of this panel are asking for. Quieter label, outline
			     submit, no autofocus, so the panel carries exactly one filled control
			     and it is the one at the top. -->
			<form onsubmit={submit} class="mt-4 flex flex-col gap-3 border-t border-ink/10 pt-4 dark:border-chalk/10">
				<label class="text-xs font-semibold opacity-75" for="switch-handle">
					or switch to a handle
				</label>
				<input
					id="switch-handle"
					bind:value
					type="text"
					placeholder="your.handle.bsky.social"
					autocapitalize="none"
					autocorrect="off"
					spellcheck="false"
					class="min-w-0 rounded-full border border-ink/15 bg-chalk px-4 py-2.5 text-sm font-semibold transition-colors focus:border-pop-pink focus-visible:outline-2 focus-visible:outline-offset-0 dark:border-chalk/15 dark:bg-kiln"
				/>
				<button
					type="submit"
					class="inline-flex min-h-11 cursor-pointer items-center justify-center rounded-full border border-ink/15 px-4 font-display text-sm font-bold transition-colors hover:bg-ink/5 dark:border-chalk/15 dark:hover:bg-chalk/10"
				>
					lay bricks
				</button>
				{#if actor !== 'demo'}
					<a
						href="/?actor=demo"
						onclick={closeOnNavigation}
						class="text-center text-xs font-semibold text-brick-post-ink hover:underline dark:text-brick-post"
					>
						or wander the demo wall
					</a>
				{/if}
			</form>
		</div>
	{/if}
</div>
