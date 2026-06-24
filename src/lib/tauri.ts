import { convertFileSrc, invoke } from "@tauri-apps/api/core";
import { open } from "@tauri-apps/plugin-dialog";
import type { BiosImportResult, DetectionResult, EmulatorStatus, Game, InstallResult, LaunchResult, WindowsDiscoveryResult, WindowsGameCandidate } from "../types";
import { demoGames } from "../data/catalog";

const inTauri = () => "__TAURI_INTERNALS__" in window;

export const localAssetUrl = (path?: string) =>
  path && inTauri() ? convertFileSrc(path) : undefined;

export async function listGames(): Promise<Game[]> {
  return inTauri() ? invoke<Game[]>("list_games") : import.meta.env.VITE_DEMO_DATA === "true" ? demoGames : [];
}

export async function scanDirectory(path: string): Promise<Game[]> {
  return inTauri() ? invoke<Game[]>("scan_directory", { path }) : demoGames;
}

export async function chooseGameFile(): Promise<string | null> {
  if (!inTauri()) return null;
  const result = await open({
    multiple: false,
    title: "Vyber herný súbor",
    filters: [{
      name: "Podporované hry",
      extensions: ["nes", "sfc", "smc", "gb", "gbc", "gba", "n64", "z64", "v64", "nds", "md", "gen", "sms", "gg", "a26", "a52", "a78", "lnx", "cue", "chd", "iso", "cso", "gcz", "rvz", "wbfs", "wad", "pbp", "zip", "7z", "rar", "jsdos", "adf", "d64", "exe", "bat", "com", "bin"]
    }]
  });
  return typeof result === "string" ? result : null;
}

export async function importGameFile(path: string, systemId: string): Promise<Game[]> {
  if (!inTauri()) return demoGames;
  return invoke<Game[]>("import_game_file", { path, systemId });
}

export async function scanDirectoryForSystem(path: string, systemId: string): Promise<Game[]> {
  if (!inTauri()) return demoGames;
  return invoke<Game[]>("scan_directory_for_system", { path, systemId });
}

export async function refreshGameMetadata(gameId: string): Promise<Game[]> {
  if (!inTauri()) return demoGames;
  return invoke<Game[]>("refresh_game_metadata", { gameId });
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
        { id: "retroarch", displayName: "RetroArch", state: "not-installed", supportedSystems: ["NES", "SNES", "GBA", "Arcade"], canManagedInstall: true, officialUrl: "https://www.retroarch.com/?page=platforms", biosRequired: true, biosConfigured: false },
        { id: "pcsx2", displayName: "PCSX2", state: "not-installed", supportedSystems: ["PlayStation 2"], canManagedInstall: true, officialUrl: "https://pcsx2.net/downloads/", biosRequired: true, biosConfigured: false },
        { id: "dolphin", displayName: "Dolphin", state: "not-installed", supportedSystems: ["GameCube", "Wii"], canManagedInstall: false, officialUrl: "https://dolphin-emu.org/download/", biosRequired: false, biosConfigured: false }
      ];
}

export async function installManagedEmulator(emulatorId: string): Promise<InstallResult> {
  if (!inTauri()) throw new Error("Automatická inštalácia je dostupná iba v desktopovej aplikácii.");
  return invoke<InstallResult>("install_managed_emulator", { emulatorId });
}

export async function chooseBios(emulatorId: string): Promise<BiosImportResult | null> {
  if (!inTauri()) return null;
  const firmware = emulatorId === "rpcs3";
  const result = await open({
    multiple: false,
    title: firmware ? "Vyber oficiálny PS3 firmware" : "Vyber BIOS súbor",
    filters: firmware
      ? [{ name: "PS3 firmware", extensions: ["pup"] }]
      : [{ name: "BIOS", extensions: ["bin", "rom", "mec", "nvm"] }]
  });
  if (typeof result !== "string") return null;
  return invoke<BiosImportResult>("import_bios", { emulatorId, sourcePath: result });
}

export async function openOfficialEmulatorPage(emulatorId: string): Promise<void> {
  if (!inTauri()) throw new Error("Oficiálnu stránku otvoríš v desktopovej aplikácii.");
  await invoke("open_official_emulator_page", { emulatorId });
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

export async function scanWindowsGames(roots: string[] = []): Promise<WindowsDiscoveryResult> {
  if (!inTauri()) return { candidates: [], scannedSources: ["Steam", "Epic Games", "GOG", "Windows skratky"], warnings: ["Discovery vyžaduje desktopovú aplikáciu."] };
  return invoke<WindowsDiscoveryResult>("scan_windows_games", { roots });
}

export async function listWindowsCandidates(): Promise<WindowsGameCandidate[]> {
  return inTauri() ? invoke<WindowsGameCandidate[]>("list_windows_candidates") : [];
}

export async function choosePortableScanRoot(): Promise<string | null> {
  if (!inTauri()) return null;
  const result = await open({ directory: true, multiple: false, title: "Vyber priečinok pre hlboký scan Windows hier" });
  return typeof result === "string" ? result : null;
}

export async function confirmWindowsGames(candidateIds: string[]): Promise<Game[]> {
  return inTauri() ? invoke<Game[]>("confirm_windows_games", { candidateIds }) : [];
}

export async function rejectWindowsGames(candidateIds: string[]): Promise<void> {
  if (inTauri()) await invoke("reject_windows_games", { candidateIds });
}
