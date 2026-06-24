import "@testing-library/jest-dom/vitest";
import { cleanup, render, screen, within } from "@testing-library/react";
import userEvent from "@testing-library/user-event";
import { afterEach, describe, expect, it, vi } from "vitest";
import type { Game } from "../types";
import { SystemLibraryPage } from "./SystemLibraryPage";

const game: Game = {
  id: "tekken-3",
  title: "Tekken 3",
  systemId: "ps1",
  primaryFile: "C:\\Games\\Tekken 3.cue",
  description: "",
  totalPlayTimeSeconds: 0,
  favorite: false,
  accent: "#15d6ff",
};

afterEach(cleanup);

describe("SystemLibraryPage", () => {
  it("opens a game detail and exposes the launch action", async () => {
    const onPlay = vi.fn().mockResolvedValue(undefined);
    render(
      <SystemLibraryPage
        systemId="ps1"
        games={[game]}
        selectedId={game.id}
        onSelect={vi.fn()}
        onBack={vi.fn()}
        onAddFile={vi.fn()}
        onAddFolder={vi.fn()}
        onRefreshMetadata={vi.fn()}
        onPlay={onPlay}
      />,
    );

    await userEvent.click(screen.getByRole("button", { name: /Tekken 3/ }));
    const dialog = screen.getByRole("dialog");
    expect(dialog).toBeInTheDocument();
    expect(within(dialog).getByRole("heading", { name: "Tekken 3" })).toBeInTheDocument();

    await userEvent.click(screen.getByRole("button", { name: "Spustiť hru" }));
    expect(onPlay).toHaveBeenCalledWith("tekken-3");
  });

  it("shows a launch error inside the open detail", async () => {
    render(
      <SystemLibraryPage
        systemId="ps1"
        games={[game]}
        selectedId={game.id}
        onSelect={vi.fn()}
        onBack={vi.fn()}
        onAddFile={vi.fn()}
        onAddFolder={vi.fn()}
        onRefreshMetadata={vi.fn()}
        onPlay={vi.fn().mockRejectedValue(new Error("DuckStation nie je pripravený."))}
      />,
    );

    await userEvent.click(screen.getByRole("button", { name: /Tekken 3/ }));
    await userEvent.click(screen.getByRole("button", { name: "Spustiť hru" }));

    expect(await screen.findByRole("alert")).toHaveTextContent(
      "DuckStation nie je pripravený.",
    );
  });
});
