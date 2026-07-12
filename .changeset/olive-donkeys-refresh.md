---
"mason": patch
---

Serve `site.webmanifest` with the right content type.

S3 was returning `application/octet-stream`, because blogwright had no entry for the extension. Fixed upstream in blogwright 0.3.1, which also grants the CI build role the `s3:PutObjectTagging` permission its object tagging needs, and which now creates the preview stack's wildcard DNS record and its CloudFront log delivery instead of asking for them by hand.

S3 writes object metadata only on a PUT, so a normal deploy skips content-identical files and a corrected header never reaches an object already live. Both workflows can now pass `--refresh` to re-upload everything: preview always does, production takes it as an input.
