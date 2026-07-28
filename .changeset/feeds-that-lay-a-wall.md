---
"mason": patch
---

the feed picker only offers feeds that actually lay a wall. every one of the top
fifty popular feeds was asked for a page logged out, and the eleven that answer
with an error or a handful of posts are no longer listed: a card that opens onto
three bricks reads as broken whether the feed is gated or just quiet.

it cuts the other way too. two feeds mason was hiding turned out to work fine
logged out, and they are back: the rule was keyed to their names, and the names
belong to several publishers. now anything that could plausibly work is measured
per publisher rather than assumed, and `pnpm feeds:audit` re-derives the whole
list against the live directory.
