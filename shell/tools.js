.pragma library

// The programs the shell runs for the system's own settings. The Nix package
// of this configuration writes their store paths here, so they need not be
// installed; without Nix they are looked up in PATH.

// Reads and writes the light or dark preference that programs follow
// (org.gnome.desktop.interface color-scheme, which xdg-desktop-portal-gtk
// hands to every program as org.freedesktop.appearance color-scheme).
var dconf = "dconf";

// Listens for logind's Lock signal, so `loginctl lock-session` locks.
var gdbus = "gdbus";

// Opens, launches and trashes the desktop's files: `gio open`, `gio launch`
// for a .desktop shortcut, `gio trash`.
var gio = "gio";
