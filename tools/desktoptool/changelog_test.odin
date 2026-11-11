package desktoptool

import "core:strings"
import "core:testing"

CARGO_SAMPLE :: "[package]\nname = \"shojiwm-shell\"\nversion = \"0.1.0\"\nedition = \"2024\"\n\n[dependencies]\nfoo = { version = \"9.9.9\" }\n"
LOCK_SAMPLE :: "[[package]]\nname = \"serde\"\nversion = \"1.0.0\"\n\n[[package]]\nname = \"shojiwm-shell\"\nversion = \"0.1.0\"\ndependencies = [\n \"serde\",\n]\n\n[[package]]\nname = \"zed\"\nversion = \"2.0.0\"\n"
FLAKE_SAMPLE :: "{\n  outputs = {self}: let\n    odinVersion = \"dev-2026-11\";\n    version = \"0.1.0\";\n  in {\n    inherit version;\n    other = { version = \"3.0.0\"; };\n  };\n}\n"

@(test)
the_versions_are_read :: proc(t: ^testing.T) {
	testing.expect_value(t, version_in(CARGO_SAMPLE, cargo_version_line), "0.1.0")
	testing.expect_value(t, version_in(LOCK_SAMPLE, lock_version_line), "0.1.0")
	testing.expect_value(t, version_in(FLAKE_SAMPLE, flake_version_line), "0.1.0")
}

@(test)
prepare_writes_the_version_in_the_three_files :: proc(t: ^testing.T) {
	cargo, ok := with_version_in(CARGO_SAMPLE, "0.2.0", cargo_version_line)
	testing.expect(t, ok)
	testing.expect_value(t, cargo, "[package]\nname = \"shojiwm-shell\"\nversion = \"0.2.0\"\nedition = \"2024\"\n\n[dependencies]\nfoo = { version = \"9.9.9\" }\n")
	lock: string
	lock, ok = with_version_in(LOCK_SAMPLE, "0.2.0", lock_version_line)
	testing.expect(t, ok)
	testing.expect(t, strings.contains(lock, "name = \"shojiwm-shell\"\nversion = \"0.2.0\"\n"), "the shell's entry")
	testing.expect(t, strings.contains(lock, "name = \"serde\"\nversion = \"1.0.0\"\n"), "not the entry before")
	testing.expect(t, strings.contains(lock, "name = \"zed\"\nversion = \"2.0.0\"\n"), "not the entry after")
	flake: string
	flake, ok = with_version_in(FLAKE_SAMPLE, "0.2.0", flake_version_line)
	testing.expect(t, ok)
	testing.expect(t, strings.contains(flake, "\n    version = \"0.2.0\";\n"), "the indentation and the ';' stay")
	testing.expect(t, strings.contains(flake, "odinVersion = \"dev-2026-11\";"), "not another name's version")
	testing.expect(t, strings.contains(flake, "{ version = \"3.0.0\"; }"), "only the first")
}

@(test)
prepare_keeps_a_crlf_file :: proc(t: ^testing.T) {
	crlf, _ := strings.replace_all(CARGO_SAMPLE, "\n", "\r\n", context.temp_allocator)
	testing.expect_value(t, version_in(crlf, cargo_version_line), "0.1.0")
	text, ok := with_version_in(crlf, "1.0.0", cargo_version_line)
	testing.expect(t, ok)
	testing.expect(t, strings.contains(text, "version = \"1.0.0\"\r\nedition"), "the line keeps its end")
	testing.expect_value(t, strings.count(text, "\r\n"), strings.count(crlf, "\r\n"))
}

@(test)
a_file_without_a_version_says_so :: proc(t: ^testing.T) {
	lock := "[[package]]\nname = \"serde\"\nversion = \"1.0.0\"\n"
	testing.expect_value(t, version_in(lock, lock_version_line), "")
	text, ok := with_version_in(lock, "1.0.0", lock_version_line)
	testing.expect(t, !ok, "no shell entry, nothing written")
	testing.expect_value(t, text, lock)
	cargo := "[workspace]\nversion = \"1.0.0\"\n"
	testing.expect_value(t, version_in(cargo, cargo_version_line), "")
}
