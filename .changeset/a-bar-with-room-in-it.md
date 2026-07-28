---
"mason": minor
---

the header bar has room in it again, and a settings screen behind a cog.

on a phone the layout picker was three segments laid side by side, and it was
wide enough that two of its own touch targets had been shaved under 44px to fit
a fourth control beside it. it is a dropdown there now, in the same language as
the client picker, and every control on the bar is back to a 44px target. from
sm up it is the slider it always was.

the bar reads left to right: layout, the wall switcher, refresh, settings. the
switcher's panel now lists the five feeds you opened most recently, as real
links, so going back to one is a tap rather than a trip through the picker.

which client a post opens in has moved to settings, because it is a choice you
make once and the bar is for what changes while you read. settings is a screen
held in history state like the reader and the picker: the address bar keeps
naming the wall behind it and the back gesture closes it.

two more clients to open posts in: twinkl and witchsky. twinkl spells its
profile routes differently from everyone else, so mason rewrites the path rather
than only swapping the host, which is the difference between a link that opens
and one that 404s.
