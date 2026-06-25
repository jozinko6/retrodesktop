import { render, screen } from "@testing-library/react";
import userEvent from "@testing-library/user-event";
import { beforeEach, describe, expect, it, vi } from "vitest";
import { WindowsDiscovery } from "./WindowsDiscovery";

const mocks = vi.hoisted(() => ({
  list: vi.fn(),
  scan: vi.fn(),
  confirm: vi.fn(),
  reject: vi.fn(),
  chooseRoot: vi.fn()
}));

vi.mock("../lib/tauri", () => ({
  listWindowsCandidates: mocks.list,
  scanWindowsGames: mocks.scan,
  confirmWindowsGames: mocks.confirm,
  rejectWindowsGames: mocks.reject,
  choosePortableScanRoot: mocks.chooseRoot
}));

describe("WindowsDiscovery", () => {
  beforeEach(() => {
    mocks.list.mockResolvedValue([]);
    mocks.scan.mockResolvedValue({
      candidates: [{
        id: "steam-1",
        source: "steam",
        sourceId: "123",
        title: "Synthetic Windows Game",
        launchKind: "uri",
        launchTarget: "steam://rungameid/123",
        launchArguments: [],
        confidence: 1,
        evidence: ["Steam appmanifest"]
      }],
      scannedSources: ["Steam"],
      warnings: []
    });
    mocks.confirm.mockResolvedValue([]);
  });

  it("scans, preselects high confidence matches, and confirms them", async () => {
    render(<WindowsDiscovery onImported={vi.fn()} />);
    await userEvent.click(screen.getByRole("button", { name: "Skenovať Windows hry" }));
    expect(await screen.findByText("Synthetic Windows Game")).toBeInTheDocument();
    expect(screen.getByText("100 %")).toBeInTheDocument();
    await userEvent.click(screen.getByRole("button", { name: "Pridať do knižnice" }));
    expect(mocks.confirm).toHaveBeenCalledWith(["steam-1"]);
  });
});
