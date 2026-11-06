// Link sanitising (SPEC F-08, F3-06): only http, https and mailto survive. The backend checks
// again before saving (`validate_content_json`), so a hand-made payload cannot skip this.

const ALLOWED_PROTOCOLS = new Set(["http:", "https:", "mailto:"]);
const HAS_SCHEME = /^[a-z][a-z0-9+.-]*:/i;
const EMAIL = /^[^\s@/:]+@[^\s@/:]+\.[^\s@/:]+$/;

/**
 * Returns the address to store, or `null` if it must be rejected. A bare address gets a
 * scheme: `ana@example.com` becomes `mailto:` and `example.com` becomes `https://`.
 * The decision uses the parsed URL, so tricks such as `java\tscript:` are rejected too.
 */
export function sanitizeLink(raw: string): string | null {
  const text = raw.trim();
  if (!text) return null;

  let candidate = text;
  if (!HAS_SCHEME.test(text)) {
    if (/\s/.test(text)) return null;
    candidate = EMAIL.test(text) ? `mailto:${text}` : `https://${text}`;
  }

  let url: URL;
  try {
    url = new URL(candidate);
  } catch {
    return null;
  }
  if (!ALLOWED_PROTOCOLS.has(url.protocol)) return null;
  if (url.protocol === "mailto:") return url.pathname ? url.href : null;
  return url.hostname ? url.href : null;
}
