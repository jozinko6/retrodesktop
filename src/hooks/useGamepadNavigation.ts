import { useEffect, useRef } from "react";

const DEADZONE = 0.55;
const INITIAL_REPEAT_MS = 360;
const REPEAT_MS = 115;

export function directionFromAxes(x: number, y: number) {
  if (Math.abs(x) < DEADZONE && Math.abs(y) < DEADZONE) return null;
  if (Math.abs(x) > Math.abs(y)) return x > 0 ? "right" : "left";
  return y > 0 ? "down" : "up";
}

export function useGamepadNavigation(enabled = true) {
  const previous = useRef<Record<string, boolean>>({});
  const heldSince = useRef<Record<string, number>>({});
  const lastRepeat = useRef<Record<string, number>>({});

  useEffect(() => {
    if (!enabled) return;
    let frame = 0;
    const loop = (time: number) => {
      const gamepad = navigator.getGamepads?.()[0];
      if (gamepad) {
        const axisDirection = directionFromAxes(gamepad.axes[0] ?? 0, gamepad.axes[1] ?? 0);
        const states: Record<string, boolean> = {
          up: gamepad.buttons[12]?.pressed || axisDirection === "up",
          down: gamepad.buttons[13]?.pressed || axisDirection === "down",
          left: gamepad.buttons[14]?.pressed || axisDirection === "left",
          right: gamepad.buttons[15]?.pressed || axisDirection === "right",
          accept: gamepad.buttons[0]?.pressed,
          back: gamepad.buttons[1]?.pressed
        };
        for (const [action, pressed] of Object.entries(states)) {
          const edge = pressed && !previous.current[action];
          if (edge) {
            heldSince.current[action] = time;
            lastRepeat.current[action] = time;
            window.dispatchEvent(new CustomEvent("retrobox-gamepad", { detail: action }));
          } else if (
            pressed &&
            time - (heldSince.current[action] ?? time) >= INITIAL_REPEAT_MS &&
            time - (lastRepeat.current[action] ?? 0) >= REPEAT_MS &&
            action !== "accept" &&
            action !== "back"
          ) {
            lastRepeat.current[action] = time;
            window.dispatchEvent(new CustomEvent("retrobox-gamepad", { detail: action }));
          }
          previous.current[action] = pressed;
        }
      }
      frame = requestAnimationFrame(loop);
    };
    frame = requestAnimationFrame(loop);
    return () => cancelAnimationFrame(frame);
  }, [enabled]);
}
