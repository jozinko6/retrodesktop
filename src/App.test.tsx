import { render, screen } from "@testing-library/react";
import userEvent from "@testing-library/user-event";
import { beforeEach, describe, expect, it, vi } from "vitest";
import { App } from "./App";

vi.mock("./lib/tauri", () => ({
  listGames: vi.fn().mockResolvedValue([]),
  listEmulators: vi.fn().mockResolvedValue([]),
  listWindowsCandidates: vi.fn().mockResolvedValue([]),
  chooseDirectory: vi.fn(),
  scanDirectory: vi.fn(),
  chooseRetroArchCore: vi.fn(),
  launchGame: vi.fn(),
  scanWindowsGames: vi.fn(),
  choosePortableScanRoot: vi.fn(),
  confirmWindowsGames: vi.fn(),
  rejectWindowsGames: vi.fn(),
  getRemotePlayStatus: vi.fn().mockResolvedValue({ installed: false, running: false, webUiUrl: "https://localhost:47990", port: 47990 }),
  startRemotePlayHost: vi.fn(),
  openRemotePlayTarget: vi.fn()
}));

describe("App navigation", () => {
  beforeEach(() => localStorage.setItem("retrobox:onboarded", "true"));

  it("renders a distinct page for each main navigation item", async () => {
    render(<App />);
    await userEvent.click(screen.getByRole("button", { name: "Systémy" }));
    expect(screen.getByRole("heading", { name: "Systémy" })).toBeInTheDocument();
    await userEvent.click(screen.getByRole("button", { name: "Sťahovania" }));
    expect(screen.getByRole("heading", { name: "Sťahovania" })).toBeInTheDocument();
    await userEvent.click(screen.getByRole("button", { name: "Remote hranie" }));
    expect(screen.getByRole("heading", { name: "Remote hranie" })).toBeInTheDocument();
    await userEvent.click(screen.getByRole("button", { name: "Nastavenia" }));
    expect(screen.getByRole("heading", { name: "Nastavenia" })).toBeInTheDocument();
  });
});
