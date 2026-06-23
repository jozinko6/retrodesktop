export type SystemId =
  | "nes" | "snes" | "gb" | "gbc" | "gba" | "n64" | "nds"
  | "mastersystem" | "genesis" | "gamegear" | "segacd" | "sega32x"
  | "saturn" | "dreamcast" | "ps1" | "ps2" | "ps3" | "psp"
  | "gamecube" | "wii" | "wiiu" | "arcade" | "dos" | "scummvm"
  | "c64" | "amiga" | "msx" | "atari2600" | "atari5200"
  | "atari7800" | "lynx" | "pcengine" | "neogeo" | "windows" | "unknown";

export interface Game {
  id: string;
  title: string;
  systemId: SystemId;
  primaryFile: string;
  description: string;
  releaseYear?: number;
  developer?: string;
  genre?: string;
  totalPlayTimeSeconds: number;
  lastPlayedAt?: string;
  favorite: boolean;
  accent: string;
}

export interface EmulatorStatus {
  id: string;
  displayName: string;
  state: "not-installed" | "detected" | "managed" | "configuration-required" | "missing-firmware" | "ready" | "error";
  version?: string;
  executable?: string;
  supportedSystems: string[];
}

export interface LaunchResult {
  sessionId: string;
  exitCode?: number;
  durationSeconds: number;
}

export interface DetectionResult {
  systemId: SystemId | null;
  confidence: number;
  evidence: string[];
  candidates: Array<{ systemId: SystemId; confidence: number }>;
}

export interface WindowsGameCandidate {
  id: string;
  source: string;
  sourceId?: string;
  title: string;
  installPath?: string;
  launchKind: "exe" | "uri";
  launchTarget: string;
  launchArguments: string[];
  workingDirectory?: string;
  confidence: number;
  evidence: string[];
}

export interface WindowsDiscoveryResult {
  candidates: WindowsGameCandidate[];
  scannedSources: string[];
  warnings: string[];
}
