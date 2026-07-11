<script lang="ts">
	// Click-to-play HLS player for both Bluesky video and Steam trailers.
	// This component only ever mounts from an explicit user gesture — videos
	// on the Wall never start on their own.
	import type Hls from 'hls.js';
	import { player } from '$lib/state/player.svelte';

	let {
		id,
		playlist,
		poster,
		aspectRatio
	}: {
		id: string;
		playlist: string;
		poster: string | null;
		aspectRatio: string;
	} = $props();

	let video = $state<HTMLVideoElement | null>(null);
	let hls: Hls | null = null;
	let failed = $state(false);

	$effect(() => {
		if (!video) return;
		const el = video;
		player.claim(id);

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
			// inside the user's click gesture chain — allowed with sound
			el.play().catch(() => {});
		})();

		return () => {
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
		<p class="p-4 text-center text-sm font-semibold">this video refused to be a brick 🧱💔</p>
	</div>
{:else}
	<!-- svelte-ignore a11y_media_has_caption -->
	<video
		bind:this={video}
		controls
		playsinline
		{poster}
		class="w-full bg-kiln"
		style:aspect-ratio={aspectRatio}
	></video>
{/if}
