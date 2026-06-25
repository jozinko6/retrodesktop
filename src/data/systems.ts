import type { SystemId } from "../types";

export interface SystemDefinition {
  id: SystemId;
  name: string;
  family: string;
}

export const systemDefinitions: SystemDefinition[] = [
  { id: "windows", name: "Windows", family: "PC" },
  { id: "ps1", name: "PlayStation 1", family: "Sony" },
  { id: "ps2", name: "PlayStation 2", family: "Sony" },
  { id: "ps3", name: "PlayStation 3", family: "Sony" },
  { id: "psp", name: "PSP", family: "Sony" },
  { id: "nes", name: "Nintendo NES", family: "Nintendo" },
  { id: "snes", name: "Super Nintendo", family: "Nintendo" },
  { id: "gb", name: "Game Boy", family: "Nintendo" },
  { id: "gbc", name: "Game Boy Color", family: "Nintendo" },
  { id: "gba", name: "Game Boy Advance", family: "Nintendo" },
  { id: "n64", name: "Nintendo 64", family: "Nintendo" },
  { id: "nds", name: "Nintendo DS", family: "Nintendo" },
  { id: "gamecube", name: "Nintendo GameCube", family: "Nintendo" },
  { id: "wii", name: "Nintendo Wii", family: "Nintendo" },
  { id: "wiiu", name: "Nintendo Wii U", family: "Nintendo" },
  { id: "mastersystem", name: "Sega Master System", family: "Sega" },
  { id: "genesis", name: "Sega Mega Drive", family: "Sega" },
  { id: "gamegear", name: "Sega Game Gear", family: "Sega" },
  { id: "segacd", name: "Sega CD", family: "Sega" },
  { id: "sega32x", name: "Sega 32X", family: "Sega" },
  { id: "saturn", name: "Sega Saturn", family: "Sega" },
  { id: "dreamcast", name: "Sega Dreamcast", family: "Sega" },
  { id: "atari2600", name: "Atari 2600", family: "Atari" },
  { id: "atari5200", name: "Atari 5200", family: "Atari" },
  { id: "atari7800", name: "Atari 7800", family: "Atari" },
  { id: "lynx", name: "Atari Lynx", family: "Atari" },
  { id: "pcengine", name: "PC Engine", family: "NEC" },
  { id: "neogeo", name: "Neo Geo", family: "SNK" },
  { id: "arcade", name: "Arcade", family: "Arcade" },
  { id: "dos", name: "DOS", family: "PC" },
  { id: "scummvm", name: "ScummVM", family: "PC" },
  { id: "c64", name: "Commodore 64", family: "Počítače" },
  { id: "amiga", name: "Amiga", family: "Počítače" },
  { id: "msx", name: "MSX", family: "Počítače" },
  { id: "unknown", name: "Ostatné", family: "Iné" },
];

export const systemName = (id: SystemId) =>
  systemDefinitions.find((system) => system.id === id)?.name ?? id.toUpperCase();
