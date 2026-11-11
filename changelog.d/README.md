# changelog.d

One file per change, assembled into `CHANGELOG.md` when a version is published. A branch adds
its fragment here and never edits `CHANGELOG.md`: two branches never conflict on it.

A fragment is `changelog.d/<yyyymmdd-hhmmss>-<subject>.md` (subject: lowercase letters, digits
and `-`):

```
type: Added

**The thing**: what it does for the user, in one entry (no leading "- ").
```

- `type` is `Added`, `Changed`, `Deprecated`, `Removed`, `Fixed` or `Security`
  ([Keep a Changelog](https://keepachangelog.com/en/1.1.0/)): it picks the subsection.
- The text is one entry of the list, as it will read there. It may span lines; no blank line,
  no heading.
- One change, one fragment. A change that a user cannot see has no fragment.
