/**
 * Global shortcuts for other programs, through the GlobalShortcuts portal.
 *
 * Programs such as Hex bind their shortcuts through the portal service
 * (`portal/`, xdg-desktop-portal-shojiwm-win7). The service sends the
 * triggers of every bound shortcut here, over the shell socket:
 *
 *   { "method": "shortcuts", "params": { "triggers": ["Alt+space"] } }
 *
 * Each trigger becomes a key binding. A press sends the event
 * `shortcut` `{ trigger, state: "pressed" }` back to the service. An input
 * grab then watches the keyboard: the first release of any key, in any
 * order, sends `{ trigger, state: "released" }`. A press of another key or
 * a mouse button also ends the shortcut.
 */
import {
  COMPOSITOR,
  type InputGrab,
  type IpcClient,
  type IpcServer,
} from "shoji_wm";

/**
 * A key that no keyboard produces. ShojiWM cannot remove a binding, so a
 * withdrawn trigger is bound to this key instead.
 */
const NO_KEY = "VoidSymbol";

/** Modifiers, then one keysym name: `Ctrl+Alt+v`, `Super+F1`, `space`. */
const TRIGGER = /^((Ctrl|Alt|Shift|Super)\+)*[A-Za-z0-9_]+$/;

export function serveShortcuts(server: IpcServer): void {
  /** The service's connection, which gets the events. */
  let portal: IpcClient | null = null;
  /** The triggers bound now. */
  let bound = new Set<string>();
  /** The shortcut held down, and the grab that waits for its release. */
  let held: { trigger: string; grab: InputGrab } | null = null;

  function release(): void {
    if (held === null) {
      return;
    }
    const { trigger, grab } = held;
    held = null;
    grab.release();
    portal?.send("shortcut", { trigger, state: "released" });
  }

  function press(trigger: string): void {
    release();
    portal?.send("shortcut", { trigger, state: "pressed" });
    held = {
      trigger,
      grab: COMPOSITOR.input.grab({
        onKey: release,
        onPointerButton: release,
        onCancel: release,
      }),
    };
  }

  function bind(trigger: string): void {
    COMPOSITOR.key.bind(`portal:${trigger}`, trigger, () => press(trigger));
    // The release can come before the grab is in place.
    COMPOSITOR.key.bind(`portal-release:${trigger}`, trigger, release, {
      on: "release",
    });
  }

  function unbind(trigger: string): void {
    const nothing = (): void => undefined;
    COMPOSITOR.key.bind(`portal:${trigger}`, NO_KEY, nothing);
    COMPOSITOR.key.bind(`portal-release:${trigger}`, NO_KEY, nothing, {
      on: "release",
    });
  }

  server.handle("shortcuts", (params, client) => {
    portal = client;
    const list =
      typeof params === "object" && params !== null
        ? (params as { triggers?: unknown }).triggers
        : undefined;
    const wanted = new Set(
      (Array.isArray(list) ? list : []).filter(
        (trigger): trigger is string =>
          typeof trigger === "string" && TRIGGER.test(trigger),
      ),
    );
    for (const trigger of bound) {
      if (!wanted.has(trigger)) {
        unbind(trigger);
      }
    }
    for (const trigger of wanted) {
      if (!bound.has(trigger)) {
        bind(trigger);
      }
    }
    if (held !== null && !wanted.has(held.trigger)) {
      release();
    }
    bound = wanted;
    return [...wanted];
  });
}
