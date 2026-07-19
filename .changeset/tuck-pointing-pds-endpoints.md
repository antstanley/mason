---
"mason": patch
---

server mode now checks where a PDS pointer actually leads before it knocks: a
DID document can no longer steer mortar at loopback, private, link-local, or
cloud-metadata addresses, and only https endpoints are followed. hostile bricks
stay outside the wall.
