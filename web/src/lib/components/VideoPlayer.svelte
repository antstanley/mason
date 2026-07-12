<script lang="ts">
	// Click-to-play HLS player: Bluesky clips, and Streamplace streams both
	// live and archived. All three are m3u8; hls.js does not need to know
	// which is which.
	// This component only ever mounts from an explicit user gesture; videos
	// on the Wall never start on their own. It pauses itself when scrolled
	// out of view, and reports buffering so slow connections never see a
	// silent black box.
	import type Hls from 'hls.js';
	import { player } from '$lib/state/player.svelte';

	let {
		id,
		playlist,
		poster,
		aspectRatio,
		live = false
	}: {
		id: string;
		playlist: string;
		poster: string | null;
		aspectRatio: string;
		/** a live stream can end between the wall being laid and the click */
		live?: boolean;
	} = $props();

	let video = $state<HTMLVideoElement | null>(null);
	let hls: Hls | null = null;
	let failed = $state(false);
	let buffering = $state(true);

	$effect(() => {
		if (!video) return;
		const el = video;
		player.claim(id);

		const onPlaying = () => (buffering = false);
		const onWaiting = () => (buffering = true);
		el.addEventListener('playing', onPlaying);
		el.addEventListener('waiting', onWaiting);

		void (async () => {
			if (el.canPlayType('application/vnd.apple.mpegurl')) {
				el.src = playlist;
			} else {
				const { default: HlsCtor } = await import('hls.js');
				if (!HlsCtor.isSupported()) {
					failed = true;
					return;
				}
				hls = new HlsCtor();
				hls.on(HlsCtor.Events.ERROR, (_evt, data) => {
					if (data.fatal) failed = true;
				});
				hls.loadSource(playlist);
				hls.attachMedia(el);
			}
			// inside the user's click gesture chain; allowed with sound
			el.play().catch(() => {});
		})();

		// a brick that leaves the wall goes quiet; no off-screen audio
		const io = new IntersectionObserver(
			(entries) => {
				if (!entries[0].isIntersecting && !el.paused) el.pause();
			},
			{ threshold: 0 }
		);
		io.observe(el);

		return () => {
			io.disconnect();
			el.removeEventListener('playing', onPlaying);
			el.removeEventListener('waiting', onWaiting);
			hls?.destroy();
			hls = null;
			el.pause();
			player.release(id);
		};
	});

	// another card claimed the slot → this player yields
	$effect(() => {
		if (player.activeId !== id && video && !video.paused) video.pause();
	});
</script>

{#if failed}
	<div class="grid w-full place-items-center bg-kiln text-chalk" style:aspect-ratio={aspectRatio}>
		<p class="p-4 text-center text-sm font-semibold">
			{#if live}
				this stream has ended.
			{:else}
				this video would not play here. open it at the source instead.
			{/if}
		</p>
	</div>
{:else}
	<div class="relative">
		<!-- svelte-ignore a11y_media_has_caption -->
		<video
			bind:this={video}
			controls
			playsinline
			{poster}
			class="w-full bg-kiln"
			style:aspect-ratio={aspectRatio}
		></video>
		{#if buffering}
			<div
				class="pointer-events-none absolute inset-0 grid place-items-center bg-kiln/40"
				role="status"
			>
				<span
					class="motion-safe:animate-pulse rounded-full bg-kiln/80 px-4 py-1.5 text-sm font-semibold text-chalk"
				>
					laying the video…
				</span>
			</div>
		{/if}
	</div>
{/if}
