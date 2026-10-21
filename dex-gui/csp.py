"""trunk post_build hook: add a Content-Security-Policy to a release build's index.html.

GitHub Pages cannot send headers, so the policy is a <meta> tag, and the inline module script that
trunk injects to load the wasm is allowed by its SHA-256 hash, computed here after trunk wrote it.
Debug builds (`trunk serve`) are left alone, so trunk's autoreload script keeps working.
"""

import base64
import hashlib
import os
import re
import sys

if os.environ.get("TRUNK_PROFILE") != "release":
    sys.exit(0)

path = os.path.join(os.environ["TRUNK_STAGING_DIR"], "index.html")
with open(path, encoding="utf-8") as f:
    html = f.read()

inline = re.findall(r"<script(?![^>]*\bsrc=)[^>]*>(.*?)</script>", html, re.S)
if not inline:
    sys.exit("csp.py: no inline script found; trunk's output changed")
hashes = " ".join(
    "'sha256-" + base64.b64encode(hashlib.sha256(body.encode("utf-8")).digest()).decode() + "'"
    for body in inline
)
policy = "; ".join(
    [
        "default-src 'none'",
        f"script-src 'self' 'wasm-unsafe-eval' {hashes}",
        "style-src 'self'",
        # Bootstrap's form controls draw their icons from data: SVGs.
        "img-src 'self' data:",
        # init() fetches the wasm module.
        "connect-src 'self'",
        "base-uri 'none'",
        "form-action 'none'",
    ]
)
meta = f'<meta http-equiv="Content-Security-Policy" content="{policy}" />'
# First in <head>, so the policy covers every element after it.
html, found = re.subn(r'<meta charset="utf-8"\s*/?>', lambda m: m.group(0) + "\n  " + meta, html)
if found != 1:
    sys.exit(f"csp.py: expected one <meta charset=\"utf-8\"> in {path}, found {found}")
with open(path, "w", encoding="utf-8") as f:
    f.write(html)
