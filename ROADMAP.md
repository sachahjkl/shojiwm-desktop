# Roadmap

## Now

- **New pictures for the site**: the gallery shows the desktop before the GTK themes, the screen-share dialog and the folding thumbnails. Take them again in both looks.
- **Screen sharing checked end to end**: share a screen and a window from a browser, through ShojiWM's portal and the desktop's own dialog, on a real session.
- **Num Lock checked on a real keyboard**: the state comes back at the next session and the keyboard's light follows it.

## Next

- **Libadwaita programs in the desktop's look**: GTK 4 programs that use libadwaita (Ghostty among them) ignore the GTK themes; give them the look through their own style sheet.
- **The upstream pull requests merged**: the portal's external picker (bea4dev/ShojiWM#238), Num Lock at session start (#239), and the other branches in the fork's `FORK.md`. Each one merged leaves the fork.
- **A tool that checks the changelog and the roadmap**: the format of `changelog.d/`, `CHANGELOG.md` and this file, as a check of the flake.

## Later

- **Window tabs for windows with their own title bar**: today they never become tabs. A tab strip that takes the place of their bar would let them join a group.
