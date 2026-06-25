import "@testing-library/jest-dom/vitest";
import { cleanup, render, screen, within } from "@testing-library/react";
import userEvent from "@testing-library/user-event";
import { afterEach, describe, expect, it, vi } from "vitest";
import type { Game } from "../types";
import { LibraryPage } from "./LibraryPage";

const windowsGame: Game = {
  id: "doom-64",
  title: "DOOM 64",
  systemId: "windows",
  primaryFile: "C:\\Program Files\\Epic Games\\DOOM64",
  description: "",
  totalPlayTimeSeconds: 0,
  favorite: false,
  accent: "#15d6ff",
};

afterEach(cleanup);

describe("LibraryPage", () => {
  it("opens and launches a Windows game from its card", async () => {
    const onPlay = vi.fn().mockResolvedValue(undefined);
    render(
      <LibraryPage
        games={[windowsGame]}
        selectedId=""
        onSelect={vi.fn()}
        onAddFolder={vi.fn()}
        onWindowsScan={vi.fn()}
        onPlay={onPlay}
        onRefreshMetadata={vi.fn()}
      />,
    );

    await userEvent.click(screen.getByRole("button", { name: /DOOM 64/ }));
    const dialog = screen.getByRole("dialog");
    expect(within(dialog).getByText("Lokálne nainštalovaná Windows hra.")).toBeInTheDocument();

    await userEvent.click(within(dialog).getByRole("button", { name: "Spustiť hru" }));
    expect(onPlay).toHaveBeenCalledWith("doom-64");
  });
});
