import { describe, expect, it } from "vitest";
import { directionFromAxes } from "./useGamepadNavigation";

describe("directionFromAxes", () => {
  it("applies a deadzone", () => expect(directionFromAxes(0.2, -0.3)).toBeNull());
  it("chooses the dominant axis", () => expect(directionFromAxes(0.8, 0.6)).toBe("right"));
  it("detects up", () => expect(directionFromAxes(0, -0.9)).toBe("up"));
});
