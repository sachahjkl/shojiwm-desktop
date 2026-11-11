// desktoptool: the project's machinery, one program, whatever the shell.
//   odin run tools/desktoptool -collection:gt=ext/gui_toolkit/src -- <command>
//   odin run tools/desktoptool -collection:gt=ext/gui_toolkit/src -- help
// What it needs is on the path: git (only when it is not run from the checkout's root).
package desktoptool

import "core:fmt"
import "core:os"
import "core:strings"
import toolbox "gt:toolbox"

EXIT_USAGE :: 2
// a file only this checkout has at its root
ROOT_MARK  :: "crates/shell/Cargo.toml"

Command :: struct {
	name, about: string,
	run:         proc(args: []string) -> int,
}

COMMANDS :: [?]Command{
	{name = "changelog", about = "new <type> <text> | check | preview: the fragments of changelog.d, what CHANGELOG.md is made of", run = run_changelog},
	{name = "prepare", about = "<version>: the fragments into CHANGELOG.md with today's date, the version in Cargo.toml, Cargo.lock and flake.nix", run = run_prepare},
}

main :: proc() {
	if len(os.args) < 2 || os.args[1] == "help" || os.args[1] == "-h" || os.args[1] == "--help" { usage(); return }
	// (the flake's check runs it in a copy of the tree with no .git, from its root)
	if !os.exists(ROOT_MARK) {
		root, code := toolbox.capture({"git", "rev-parse", "--show-toplevel"})
		if code != 0 { fmt.eprintln("desktoptool: not in the repository"); os.exit(EXIT_USAGE) }
		if os.set_working_directory(strings.trim_space(root)) != nil { fmt.eprintfln("desktoptool: cannot work in %s", root); os.exit(EXIT_USAGE) }
	}
	commands := COMMANDS
	for c in commands {
		if c.name == os.args[1] { os.exit(c.run(os.args[2:])) }
	}
	fmt.eprintfln("desktoptool: no command %s", os.args[1])
	usage()
	os.exit(EXIT_USAGE)
}

usage :: proc() {
	fmt.println("desktoptool <command> [arguments]")
	commands := COMMANDS
	for c in commands { fmt.printfln("  %-9s %s", c.name, c.about) }
}
