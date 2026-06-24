import { Gamepad2 } from "lucide-react";
import { useMemo } from "react";
import { systemDefinitions } from "../data/systems";
import type { Game, SystemId } from "../types";

export function SystemsPage({
  games,
  onOpenSystem,
}: {
  games: Game[];
  onOpenSystem: (id: SystemId) => void;
}) {
  const counts = useMemo(() => {
    const values = new Map<SystemId, number>();
    for (const game of games) {
      values.set(game.systemId, (values.get(game.systemId) ?? 0) + 1);
    }
    return values;
  }, [games]);

  return (
    <main className="content-page systems-page">
      <header>
        <p>Knižnica</p>
        <h1>Systémy</h1>
        <span>Vyber platformu, otvor jej knižnicu alebo pridaj vlastnú hru.</span>
      </header>
      <div className="system-grid">
        {systemDefinitions.map((system) => {
          const count = counts.get(system.id) ?? 0;
          return (
            <button key={system.id} onClick={() => onOpenSystem(system.id)}>
              <Gamepad2 size={30} />
              <small>{system.family}</small>
              <strong>{system.name}</strong>
              <span>{count} {count === 1 ? "hra" : "hier"}</span>
            </button>
          );
        })}
      </div>
    </main>
  );
}
