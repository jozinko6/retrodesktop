import type { Game } from "../types";

export const demoGames: Game[] = [
  {
    id: "neon-drift",
    title: "Neon Drift",
    systemId: "ps2",
    primaryFile: "",
    description: "Nočná arkádová jazda cez elektrické pobrežie. Pripoj vlastnú zálohu hry a vyber kompatibilný emulátor.",
    releaseYear: 2004,
    developer: "Fictional North",
    genre: "Preteky",
    totalPlayTimeSeconds: 9540,
    lastPlayedAt: "2026-06-22T20:18:00Z",
    favorite: true,
    accent: "#15d6ff"
  },
  {
    id: "orbital-farms",
    title: "Orbital Farms",
    systemId: "snes",
    primaryFile: "",
    description: "Pokojná sci-fi stratégia pre dlhé večery.",
    releaseYear: 1995,
    genre: "Stratégia",
    totalPlayTimeSeconds: 4020,
    favorite: true,
    accent: "#f3a93b"
  },
  {
    id: "castle-signal",
    title: "Castle Signal",
    systemId: "gba",
    primaryFile: "",
    description: "Kompaktné dobrodružstvo v ručne kreslenej pevnosti.",
    releaseYear: 2002,
    genre: "Dobrodružná",
    totalPlayTimeSeconds: 1860,
    favorite: false,
    accent: "#8b7cff"
  },
  {
    id: "deep-current",
    title: "Deep Current",
    systemId: "gamecube",
    primaryFile: "",
    description: "Prieskum neznámeho oceánu.",
    releaseYear: 2003,
    genre: "Prieskum",
    totalPlayTimeSeconds: 540,
    favorite: false,
    accent: "#28d2a0"
  }
];
