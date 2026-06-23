import { Gamepad2 } from "lucide-react";
import { useMemo } from "react";
import type { Game } from "../types";

const labels: Record<string, string> = {
  windows: "Windows",
  ps1: "PlayStation 1",
  ps2: "PlayStation 2",
  ps3: "PlayStation 3",
  psp: "PSP",
  nes: "Nintendo NES",
  snes: "Super Nintendo",
  gba: "Game Boy Advance",
  gamecube: "Nintendo GameCube",
  wii: "Nintendo Wii",
  genesis: "Sega Mega Drive",
  arcade: "Arcade",
  dos: "DOS",
  unknown: "Ostatné"
};

export function SystemsPage({ games, onOpenLibrary }: { games: Game[]; onOpenLibrary: () => void }) {
  const systems = useMemo(() => {
    const counts = new Map<string, number>();
    for (const game of games) counts.set(game.systemId, (counts.get(game.systemId) ?? 0) + 1);
    return [...counts.entries()].sort((a, b) => (labels[a[0]] ?? a[0]).localeCompare(labels[b[0]] ?? b[0], "sk"));
  }, [games]);

  return (
    <main className="content-page systems-page">
      <header><p>Knižnica</p><h1>Systémy</h1><span>Hry usporiadané podľa platformy.</span></header>
      {systems.length ? <div className="system-grid">
        {systems.map(([id, count]) => (
          <button key={id} onClick={onOpenLibrary}>
            <Gamepad2 size={30} />
            <strong>{labels[id] ?? id.toUpperCase()}</strong>
            <span>{count} {count === 1 ? "hra" : "hier"}</span>
          </button>
        ))}
      </div> : <div className="page-empty"><Gamepad2 size={38} /><h2>Zatiaľ tu nie sú žiadne systémy.</h2><p>Najprv pridaj hry do knižnice.</p></div>}
    </main>
  );
}
