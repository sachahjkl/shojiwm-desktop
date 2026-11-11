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

The project's tool (`tools/desktoptool`) writes, checks and assembles the fragments. Run it
from the checkout, in the development shell (`nix develop`), which gives Odin and the path of
gui_toolkit in `$GUI_TOOLKIT_SRC`:

```sh
# a fragment, named from the text's bold summary
odin run tools/desktoptool -collection:gt=$GUI_TOOLKIT_SRC -- changelog new Added "**The thing**: what it does"
# every fragment well formed, and the version the same in its three files
odin run tools/desktoptool -collection:gt=$GUI_TOOLKIT_SRC -- changelog check
# what the next version brings
odin run tools/desktoptool -collection:gt=$GUI_TOOLKIT_SRC -- changelog preview
# a release: the fragments into a "## [x.y.z] - date" section, the fragments deleted
odin run tools/desktoptool -collection:gt=$GUI_TOOLKIT_SRC -- prepare 0.2.0
```

`prepare` also writes the version in `crates/shell/Cargo.toml`, `crates/shell/Cargo.lock` and
`flake.nix`. Commit the result, then push the tag `v<version>`: the section's link points at
its tree. `nix flake check` runs `changelog check` (the `changelog` check).
