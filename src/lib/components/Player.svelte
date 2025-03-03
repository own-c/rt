<script>
    import "vidstack/bundle";

    import { stream } from "$lib/Stream.svelte";
</script>

{#key stream.url}
    <media-player
        autoplay
        stream-type="on-demand"
        style="--plyr-border-radius: 0px; max-height: calc(100vh - 2rem); max-width: calc(100vw - 2rem);"
    >
        <media-provider>
            <source src={stream.url} type="application/x-mpegurl" />
        </media-provider>

        <media-plyr-layout
            seek-time={5}
            display-duration={true}
            controls={[
                "play-large",
                "play",
                "progress",
                "current-time",
                "mute+volume",
                "settings",
                "pip",
                "airplay",
                "fullscreen",
            ]}
        ></media-plyr-layout>
    </media-player>
{/key}

<style>
    :global(media-player video) {
        max-height: 100%;
        max-width: 100%;
    }
</style>
