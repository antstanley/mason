---
"mason": patch
---

brick links are scrubbed before they reach the wall: a blog or stream record
that smuggles a `javascript:`, `data:`, or other non-http(s) url in its link
field now lands without a link at all instead of arming the card, and the client
picker refuses to rewrite anything but a real http(s) address.
