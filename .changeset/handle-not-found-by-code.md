---
"mason": patch
---

fix: a valid handle no longer gets told it does not exist. mason now only shows
the handle-not-found message when the feed engine actually reports the actor is
missing, rather than on any 404. In local mode a request that slips past the
service worker onto the static host used to 404 and wrongly accuse a real
handle; that case now reads as a plain feed-unavailable hiccup.
