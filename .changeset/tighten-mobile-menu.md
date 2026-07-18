---
"mason": patch
---

fix: the menu bar stays on one line now that Glaze is a third layout, and the
layout slider's thumb hugs each option. The segments size to their own content
and the thumb measures the selected label and matches its width and position, so
a short label like Glaze no longer leaves dead space inside the white highlight.
On mobile the slider is icon-only and the client picker drops to just its icon
(sized to match the layout icons; its label and chevron return at the sm
breakpoint), keeping the bar to one row.
