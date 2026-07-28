---
"mason": patch
---

the session keeps the last twelve walls, not every wall you ever opened.

mason remembers a laid wall so stepping back to it returns the same arrangement
instead of rolling a new one. it remembered every wall, for as long as the tab
stayed open, and an entry is a whole wall: every brick, its text, its image
urls, and a record of every id it has already laid.

that was fine when a wall was somewhere you arrived. the feed picker made
hopping between them a one tap habit, and nothing ever took an entry away. the
last twelve distinct walls are kept now, which is the length of the picker's
recents row, so every wall that row can offer you in one tap is one that comes
straight back. least recently used is the first out, so returning to a wall
keeps it and the one you glanced at once is the one that goes. stepping back and
forth between two walls costs two entries however long it goes on.
