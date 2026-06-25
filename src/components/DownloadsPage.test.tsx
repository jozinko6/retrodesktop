import { render, screen } from "@testing-library/react";
import userEvent from "@testing-library/user-event";
import { beforeEach, describe, expect, it, vi } from "vitest";
import { DownloadsPage } from "./DownloadsPage";
import {
  chooseDirectory,
  fetchCatalog,
  requestCatalogDownload,
} from "../lib/tauri";

vi.mock("../lib/tauri", () => ({
  fetchCatalog: vi.fn(),
  getDownloadDirectory: vi.fn().mockResolvedValue(null),
  chooseDirectory: vi.fn(),
  requestCatalogDownload: vi.fn(),
  subscribeCatalogDownloadProgress: vi.fn().mockResolvedValue(() => undefined),
  openCatalogTarget: vi.fn(),
}));

const game = {
  id: "gruniozerca",
  title: "Gruniożerca",
  description: "Public-domain arkádová hra.",
  systemId: "nes" as const,
  developer: "Autori",
  genre: "Arkádová",
  license: "Unlicense",
  licenseUrl: "https://github.com/example/license",
  sourceUrl: "https://github.com/example/game",
  fileName: "game.nes",
  fileSize: 40976,
  sha256: "a".repeat(64),
  thumbnailUrl: "",
};

describe("DownloadsPage", () => {
  beforeEach(() => {
    vi.mocked(fetchCatalog).mockResolvedValue([game]);
    vi.mocked(chooseDirectory).mockResolvedValue("C:\\Games");
  });

  it("requires license confirmation and imports a verified catalog game", async () => {
    const downloaded = {
      game: {
        id: "game-1",
        title: game.title,
        systemId: "nes" as const,
        primaryFile: "C:\\Games\\game.nes",
        description: game.description,
        totalPlayTimeSeconds: 0,
        favorite: false,
        accent: "#15d6ff",
      },
      sha256: game.sha256,
      storedPath: "C:\\Games\\game.nes",
    };
    vi.mocked(requestCatalogDownload).mockResolvedValue(downloaded);
    const onDownloaded = vi.fn();
    render(<DownloadsPage onDownloaded={onDownloaded} />);

    await userEvent.click(await screen.findByRole("button", { name: "Skontrolovať a stiahnuť" }));
    expect(screen.getByRole("dialog")).toHaveTextContent("Unlicense");
    await userEvent.click(screen.getByRole("button", { name: "Vybrať cieľ" }));
    await userEvent.click(screen.getByRole("button", { name: "Potvrdiť a stiahnuť" }));

    expect(requestCatalogDownload).toHaveBeenCalledWith("gruniozerca", "C:\\Games");
    expect(onDownloaded).toHaveBeenCalledWith(downloaded.game);
  });
});
