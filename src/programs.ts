/**
 * The programs the desktop's keys start: the terminal (Super+Enter), the file
 * manager (Super+E), the task manager (Ctrl+Shift+Escape), the clipboard
 * history (Super+V) and the screenshot (Print Screen). Each is a command
 * line, run by a shell.
 *
 * The Control Panel chooses them (Default Programs); the shell sends its
 * choices over its socket when it connects and when one changes. Until then,
 * and for a program left "as the session sets it", the session's environment
 * gives it (SHOJIWM_DESKTOP_TERMINAL, ..._FILE_MANAGER, ..._TASK_MANAGER,
 * ..._CLIPBOARD, ..._SCREENSHOT), then the defaults below.
 */
import { COMPOSITOR } from "shoji_wm";

export type ProgramKey =
  | "terminal"
  | "fileManager"
  | "taskManager"
  | "clipboard"
  | "screenshot";

const SESSION: Record<ProgramKey, string> = {
  terminal: COMPOSITOR.env.get("SHOJIWM_DESKTOP_TERMINAL") ?? "kitty",
  fileManager: COMPOSITOR.env.get("SHOJIWM_DESKTOP_FILE_MANAGER") ?? "thunar",
  taskManager:
    COMPOSITOR.env.get("SHOJIWM_DESKTOP_TASK_MANAGER") ?? "inspecteur",
  // No clipboard history by default: Super+V then does nothing.
  clipboard: COMPOSITOR.env.get("SHOJIWM_DESKTOP_CLIPBOARD") ?? "",
  // The whole screen to the clipboard, as Windows 7 does.
  screenshot:
    COMPOSITOR.env.get("SHOJIWM_DESKTOP_SCREENSHOT") ??
    "grim - | wl-copy --type image/png",
};

const chosen: Partial<Record<ProgramKey, string>> = {};

/** The command of a program now: the Control Panel's choice, else the session's. */
export function program(key: ProgramKey): string {
  const command = chosen[key];
  return command !== undefined && command.length > 0 ? command : SESSION[key];
}

/**
 * The nice value of a program started from the desktop. The compositor and
 * the shell stay at nice 0, so they get the CPU first: a build in a terminal
 * no longer makes the taskbar, the menus and the windows stutter.
 */
const APP_NICE = 5;

/**
 * uwsm's slice for graphical applications. Each program gets its own scope in
 * it, so a build in a terminal is accounted to that terminal, not to the
 * compositor's unit.
 */
const APP_SLICE = "app-graphical.slice";

/** The longest program name in a unit name. */
const NAME_LIMIT = 64;

let started = 0;

/**
 * The program's name for its unit: the file name of the first word, after any
 * VAR=value and exec, with what systemd does not allow in a unit name
 * ([a-zA-Z0-9:_.-]) escaped as \xNN, as systemd-escape writes it.
 */
function unitWord(command: string | string[]): string {
  const words =
    typeof command === "string" ? command.trim().split(/\s+/) : command;
  const first =
    words.find((word) => !word.includes("=") && word !== "exec") ?? "";
  const name =
    first
      .replace(/^['"]|['"]$/g, "")
      .split("/")
      .pop() || "program";
  let out = "";
  for (const char of name) {
    const allowed =
      /^[A-Za-z0-9:_-]$/.test(char) || (char === "." && out.length > 0);
    const piece = allowed ? char : escapeChar(char);
    if (out.length + piece.length > NAME_LIMIT) {
      break;
    }
    out += piece;
  }
  return out;
}

/** A character as systemd-escape writes it: each UTF-8 byte as \xNN. */
function escapeChar(char: string): string {
  const encoded = encodeURIComponent(char);
  const bytes = encoded.startsWith("%")
    ? encoded.slice(1).split("%")
    : [char.charCodeAt(0).toString(16).padStart(2, "0")];
  return bytes.map((byte) => `\\x${byte.toLowerCase()}`).join("");
}

/**
 * Starts a program the user works in (a terminal, the file manager, the
 * browser, the task manager) in a systemd scope of its own, in a new session,
 * a little below the desktop. A string is a command line, run by `sh -lc` as
 * COMPOSITOR.process.spawn runs one; an array is the program and its
 * arguments. The shell's commands and short helpers (the screenshot, the media
 * keys) do not come here.
 */
export function startProgram(command: string | string[]): void {
  const argv =
    typeof command === "string" ? ["/bin/sh", "-lc", command] : command;
  if (argv.length === 0 || (typeof command === "string" && !command.trim())) {
    return;
  }
  started += 1;
  const unit = `app-${unitWord(command)}-${Date.now()}-${started}`;
  COMPOSITOR.process.spawn({
    command: [
      "setsid",
      "-f",
      "nice",
      "-n",
      String(APP_NICE),
      "systemd-run",
      "--user",
      "--scope",
      `--slice=${APP_SLICE}`,
      "--collect",
      "--quiet",
      `--unit=${unit}`,
      "--",
      ...argv,
    ],
  });
}

/**
 * The shell's choices: an object of command lines by key. A key missing or
 * empty goes back to the session's program.
 */
export function setPrograms(params: unknown): void {
  const given =
    typeof params === "object" && params !== null
      ? (params as Record<string, unknown>)
      : {};
  for (const key of Object.keys(SESSION) as ProgramKey[]) {
    const value = given[key];
    chosen[key] = typeof value === "string" ? value : "";
  }
}
