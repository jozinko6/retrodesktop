import "@testing-library/jest-dom/vitest";
import { render, screen } from "@testing-library/react";
import userEvent from "@testing-library/user-event";
import { describe, expect, it, vi } from "vitest";
import { SystemsPage } from "./SystemsPage";

describe("SystemsPage", () => {
  it("shows empty supported systems and opens the selected library", async () => {
    const onOpenSystem = vi.fn();
    render(<SystemsPage games={[]} onOpenSystem={onOpenSystem} />);

    const playStation = screen.getByRole("button", {
      name: "Sony PlayStation 2 0 hier",
    });
    expect(playStation).toBeInTheDocument();

    await userEvent.click(playStation);
    expect(onOpenSystem).toHaveBeenCalledWith("ps2");
  });
});
