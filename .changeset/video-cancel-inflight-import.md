---
"mason": patch
---

fix: scrolling a playing clip off the wall before it finishes loading no longer
leaves a phantom playing in the background. If a video card is torn down while
the player library is still loading, the player now bows out instead of building
itself on a detached brick and quietly fetching segments (and audio) with nothing
left to stop it.
