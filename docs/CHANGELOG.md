## 2026-06-08

### Internal Listing Public Mirrors

Modified:
- `webby ls` refreshes public-to-internal symlinks before listing the internal bag.
- Public mirror sync prunes stale public symlinks without clobbering real internal apps.

Why:
- The default internal listing should include public apps even when a host or checkout starts without the symlinks already in place.
