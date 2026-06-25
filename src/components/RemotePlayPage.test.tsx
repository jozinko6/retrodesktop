import "@testing-library/jest-dom/vitest";
import { render, screen } from "@testing-library/react";
import userEvent from "@testing-library/user-event";
import { describe, expect, it, vi } from "vitest";
import { RemotePlayPage } from "./RemotePlayPage";

const openRemotePlayTarget = vi.fn();

vi.mock("../lib/tauri", () => ({
  getRemotePlayStatus: vi.fn().mockResolvedValue({
    installed: true,
    running: true,
    localIp: "192.168.1.42",
    webUiUrl: "https://localhost:47990",
    port: 47990,
  }),
  startRemotePlayHost: vi.fn(),
  openRemotePlayTarget: (...args: unknown[]) => openRemotePlayTarget(...args),
}));

describe("RemotePlayPage", () => {
  it("shows pairing information and opens the Sunshine UI", async () => {
    render(<RemotePlayPage />);

    expect(await screen.findByText("Pripravený na pripojenie")).toBeInTheDocument();
    expect(screen.getByText("Mobil pridaj cez adresu 192.168.1.42.")).toBeInTheDocument();

    await userEvent.click(screen.getByRole("button", { name: "Otvoriť párovanie" }));
    expect(openRemotePlayTarget).toHaveBeenCalledWith("web-ui");
  });
});
