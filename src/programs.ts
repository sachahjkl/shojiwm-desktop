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
