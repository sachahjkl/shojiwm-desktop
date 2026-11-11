package desktoptool

// CHANGELOG.md is made of fragments (changelog.d/README.md), never edited by hand: the toolkit's changelog package does the work,
// this file says what shojiwm-desktop is to it and writes the version where the project keeps it.
//   desktoptool changelog new <type> <text>   one fragment, named from the text's bold summary
//   desktoptool changelog check               every fragment well formed, the version the last released, the same in its three files
//   desktoptool changelog preview             what the next version brings
//   desktoptool prepare <version>             the release's files: the section, its link, the version; a commit and the tag v<version> then publish them

import "core:fmt"
import "core:os"
import "core:strings"
import tk_changelog "gt:changelog"

// No GitHub release is made: a version's link is the tree of its tag v<version>, pushed after prepare.
CHANGELOG_LINK :: "https://github.com/sachahjkl/shojiwm-desktop/tree/v%s"

SHELL_PACKAGE :: "shojiwm-shell"

// A file that keeps the project's version, and the line of it.
Version_File :: struct {
	path: string,
	line: proc(lines: []string) -> int,
}

// The three must say the same version: the first is the one the changelog package reads.
VERSION_FILES :: [?]Version_File{
	{path = "crates/shell/Cargo.toml", line = cargo_version_line},
	{path = "crates/shell/Cargo.lock", line = lock_version_line},
	{path = "flake.nix", line = flake_version_line},
}

// What shojiwm-desktop is to the toolkit's changelog: this checkout, its versions' link, where it keeps its version.
changelog_options :: proc() -> tk_changelog.Options {
	return {root = ".", link = CHANGELOG_LINK, version = changelog_version, set_version = changelog_set_version}
}

@(private = "file")
read_text :: proc(path: string) -> string {
	data, err := os.read_entire_file(path, context.temp_allocator)
	return err == nil ? string(data) : ""
}

// The version of a line `version = "x"` (Cargo's) or `version = "x";` (Nix's). "": it is not one.
version_of_line :: proc(line: string) -> string {
	PREFIX :: `version = "`
	l := strings.trim_space(line)
	if !strings.has_prefix(l, PREFIX) { return "" }
	rest := l[len(PREFIX):]
	end := strings.index_byte(rest, '"')
	if end <= 0 { return "" }
	tail := rest[end + 1:]
	if tail != "" && tail != ";" { return "" }
	return rest[:end]
}

// The line with its version replaced: its indentation, its ';' and its end ("\r" on a checkout with CRLF) stay.
@(private = "file")
with_line_version :: proc(line, version: string) -> string {
	PREFIX :: `version = "`
	at := strings.index(line, PREFIX) + len(PREFIX)
	end := at + strings.index_byte(line[at:], '"')
	return fmt.tprintf("%s%s%s", line[:at], version, line[end:])
}

// The version line of Cargo.toml's [package] (-1: none).
cargo_version_line :: proc(lines: []string) -> int {
	in_package := false
	for l, i in lines {
		t := strings.trim_space(l)
		if strings.has_prefix(t, "[") { in_package = t == "[package]"; continue }
		if in_package && version_of_line(l) != "" { return i }
	}
	return -1
}

// The version line of the shell's [[package]] entry in Cargo.lock (-1: none).
lock_version_line :: proc(lines: []string) -> int {
	name := fmt.tprintf(`name = "%s"`, SHELL_PACKAGE)
	in_shell := false
	for l, i in lines {
		t := strings.trim_space(l)
		if strings.has_prefix(t, "[") { in_shell = false; continue }
		if t == name { in_shell = true; continue }
		if in_shell && version_of_line(l) != "" { return i }
	}
	return -1
}

// The first `version = "x";` of flake.nix: the one its packages inherit (-1: none).
flake_version_line :: proc(lines: []string) -> int {
	for l, i in lines {
		if strings.has_suffix(strings.trim_space(l), ";") && version_of_line(l) != "" { return i }
	}
	return -1
}

// The version a file's text keeps ("": none).
version_in :: proc(text: string, line: proc(lines: []string) -> int) -> string {
	lines := strings.split(text, "\n", context.temp_allocator)
	i := line(lines)
	return i >= 0 ? version_of_line(lines[i]) : ""
}

// A file's text with its version set to `version`. ok = false: it keeps none.
with_version_in :: proc(text, version: string, line: proc(lines: []string) -> int) -> (out: string, ok: bool) {
	// (split on "\n" only: a line's "\r" stays with it)
	lines := strings.split(text, "\n", context.temp_allocator)
	i := line(lines)
	if i < 0 { return text, false }
	lines[i] = with_line_version(lines[i], version)
	return strings.join(lines, "\n", context.temp_allocator), true
}

@(private = "file")
changelog_version :: proc(root: string) -> string {
	files := VERSION_FILES
	return version_in(read_text(fmt.tprintf("%s/%s", root, files[0].path)), files[0].line)
}

// The version in the three files: what assemble writes after CHANGELOG.md. Nothing is written when one of them keeps none.
@(private = "file")
changelog_set_version :: proc(root, version: string) -> bool {
	files := VERSION_FILES
	texts: [len(VERSION_FILES)]string
	for f, i in files {
		text, found := with_version_in(read_text(fmt.tprintf("%s/%s", root, f.path)), version, f.line)
		if !found { fmt.eprintfln("prepare: no version in %s", f.path); return false }
		texts[i] = text
	}
	for f, i in files {
		if os.write_entire_file(fmt.tprintf("%s/%s", root, f.path), transmute([]byte)texts[i]) != nil {
			fmt.eprintfln("prepare: %s cannot be written", f.path)
			return false
		}
	}
	return true
}

// What is wrong with the version in the three files: one keeps none, or they differ.
version_errors :: proc(root: string) -> []string {
	errors := make([dynamic]string, context.temp_allocator)
	files := VERSION_FILES
	said: [len(VERSION_FILES)]string
	for f, i in files {
		said[i] = version_in(read_text(fmt.tprintf("%s/%s", root, f.path)), f.line)
		if said[i] == "" { append(&errors, fmt.tprintf("no version found in %s", f.path)) }
	}
	if len(errors) > 0 { return errors[:] }
	for i in 1 ..< len(files) {
		if said[i] != said[0] {
			append(&errors, fmt.tprintf("%s says version %s, %s says %s: the three files keep the same version", files[i].path, said[i], files[0].path, said[0]))
		}
	}
	return errors[:]
}

// The errors said; whether there were none.
@(private = "file")
say_errors :: proc(what: string, errors: []string) -> bool {
	for e in errors { fmt.eprintfln("%s: %s", what, e) }
	return len(errors) == 0
}

// What `changelog check` and `prepare` run.
changelog_check :: proc(release := "") -> bool {
	errors, n := tk_changelog.check(changelog_options(), release)
	append(&errors, ..version_errors("."))
	if !say_errors("changelog", errors[:]) { return false }
	fmt.printfln("changelog: %d fragments, well formed", n)
	return true
}

run_changelog :: proc(args: []string) -> int {
	USAGE :: "changelog new <Added|Changed|Deprecated|Removed|Fixed|Security> <text> | check | preview"
	if len(args) == 0 { fmt.eprintln(USAGE); return EXIT_USAGE }
	switch args[0] {
	case "new":
		if len(args) < 3 { fmt.eprintln(USAGE); return EXIT_USAGE }
		path, why := tk_changelog.new_fragment(".", args[1], "", strings.join(args[2:], " ", context.temp_allocator))
		if why != "" { fmt.eprintfln("changelog: %s", why); return 1 }
		fmt.println(path)
		return 0
	case "check":
		return changelog_check() ? 0 : 1
	case "preview":
		text, errors := tk_changelog.preview(".")
		if !say_errors("changelog", errors[:]) { return 1 }
		fmt.print(text != "" ? text : "changelog: no fragment, nothing to publish\n")
		return 0
	}
	fmt.eprintln(USAGE)
	return EXIT_USAGE
}

// The files of a release: the fragments into CHANGELOG.md, the version in its three files. Nothing is committed nor tagged.
run_prepare :: proc(args: []string) -> int {
	if len(args) != 1 { fmt.eprintln("prepare <version>"); return EXIT_USAGE }
	version := args[0]
	if !tk_changelog.version_is_good(version) { fmt.eprintfln("prepare: %s is not a version (x.y.z)", version); return EXIT_USAGE }
	if !changelog_check() { return 1 }
	n, errors := tk_changelog.assemble(changelog_options(), version)   // (and set_version: Cargo.toml, Cargo.lock, flake.nix)
	if !say_errors("prepare", errors[:]) { return 1 }
	if !changelog_check(version) { return 1 }
	fmt.printfln("prepare: %s, %d entries in CHANGELOG.md, the version in Cargo.toml, Cargo.lock and flake.nix; commit them with the deletion of the fragments, then tag v%s", version, n, version)
	return 0
}
