import { invoke } from "@tauri-apps/api/core";
import { open } from "@tauri-apps/plugin-dialog";
import type { DetectionResult, EmulatorStatus, Game, LaunchResult } from "../types";
import { demoGames } from "../data/catalog";

const inTauri = () => "__TAURI_INTERNALS__" in window;

export async function listGames(): Promise<Game[]> {
  return inTauri() ? invoke<Game[]>("list_games") : import.meta.env.VITE_DEMO_DATA === "true" ? demoGames : [];
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

export async function launchGame(gameId: string): Promise<LaunchResult> {
  if (!inTauri()) throw new Error("Spustenie hier je dostupné v desktopovej aplikácii po konfigurácii emulátora.");
  return invoke<LaunchResult>("launch_game", { gameId });
}

export async function chooseDirectory(): Promise<string | null> {
  if (!inTauri()) return null;
  const result = await open({ directory: true, multiple: false, title: "Vyber priečinok s hrami" });
  return typeof result === "string" ? result : null;
}

export async function chooseExecutable(emulatorId: string): Promise<EmulatorStatus | null> {
  if (!inTauri()) return null;
  const result = await open({
    multiple: false,
    title: "Vyber executable emulátora",
    filters: [{ name: "Windows executable", extensions: ["exe"] }]
  });
  if (typeof result !== "string") return null;
  return invoke<EmulatorStatus>("configure_emulator", { emulatorId, executable: result });
}

export async function chooseRetroArchCore(gameId: string): Promise<boolean> {
  if (!inTauri()) return false;
  const result = await open({
    multiple: false,
    title: "Vyber RetroArch core",
    filters: [{ name: "Libretro core", extensions: ["dll"] }]
  });
  if (typeof result !== "string") return false;
  await invoke("configure_retroarch_game", { gameId, corePath: result });
  return true;
}
