import { invoke } from "@tauri-apps/api/core";
import type { DetectionResult, EmulatorStatus, Game } from "../types";
import { demoGames } from "../data/catalog";

const inTauri = () => "__TAURI_INTERNALS__" in window;

export async function listGames(): Promise<Game[]> {
  return inTauri() ? invoke<Game[]>("list_games") : demoGames;
}

export async function scanDirectory(path: string): Promise<Game[]> {
  return inTauri() ? invoke<Game[]>("scan_directory", { path }) : demoGames;
}

export async function detectPlatform(path: string): Promise<DetectionResult> {
  return inTauri()
    ? invoke<DetectionResult>("detect_platform", { path })
    : { systemId: "unknown", confidence: 0.25, evidence: ["Ukážkový režim prehliadača"], candidates: [] };
}

export async function listEmulators(): Promise<EmulatorStatus[]> {
  return inTauri()
    ? invoke<EmulatorStatus[]>("list_emulators")
    : [
        { id: "retroarch", displayName: "RetroArch", state: "not-installed", supportedSystems: ["NES", "SNES", "GBA", "Arcade"] },
        { id: "pcsx2", displayName: "PCSX2", state: "not-installed", supportedSystems: ["PlayStation 2"] },
        { id: "dolphin", displayName: "Dolphin", state: "not-installed", supportedSystems: ["GameCube", "Wii"] }
      ];
}

export async function launchGame(gameId: string): Promise<void> {
  if (!inTauri()) throw new Error("Spustenie hier je dostupné v desktopovej aplikácii po konfigurácii emulátora.");
  await invoke("launch_game", { gameId });
}
