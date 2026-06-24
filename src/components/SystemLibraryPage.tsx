import { ArrowLeft, FilePlus2, FolderPlus, LoaderCircle } from "lucide-react";
import { useMemo, useState } from "react";
import { systemName } from "../data/systems";
import type { Game, SystemId } from "../types";
import { GameCard } from "./GameCard";

export function SystemLibraryPage({
  systemId,
  games,
  selectedId,
  onSelect,
  onBack,
  onAddFile,
  onAddFolder,
}: {
  systemId: SystemId;
  games: Game[];
  selectedId: string;
  onSelect: (id: string) => void;
  onBack: () => void;
  onAddFile: () => Promise<void>;
  onAddFolder: () => Promise<void>;
}) {
  const [busy, setBusy] = useState<"file" | "folder">();
  const filtered = useMemo(
    () => games.filter((game) => game.systemId === systemId),
    [games, systemId],
  );
  const selected =
    filtered.find((game) => game.id === selectedId) ?? filtered[0];

  async function run(kind: "file" | "folder", action: () => Promise<void>) {
    setBusy(kind);
    try {
      await action();
    } finally {
      setBusy(undefined);
    }
  }

  return (
    <main className="content-page system-library-page">
      <button className="back-link" onClick={onBack}>
        <ArrowLeft size={18} /> Všetky systémy
      </button>
      <header className="system-library-header">
        <div>
          <p>Systémová knižnica</p>
          <h1>{systemName(systemId)}</h1>
          <span>{filtered.length} hier v tejto platforme.</span>
        </div>
        <div className="system-library-actions">
          <button
            className="primary"
            disabled={Boolean(busy)}
            onClick={() => void run("file", onAddFile)}
          >
            {busy === "file" ? <LoaderCircle className="spin" /> : <FilePlus2 />}
            Nahrať hru
          </button>
          <button
            className="secondary"
            disabled={Boolean(busy)}
            onClick={() => void run("folder", onAddFolder)}
          >
            {busy === "folder" ? <LoaderCircle className="spin" /> : <FolderPlus />}
            Pridať priečinok
          </button>
        </div>
      </header>
      {selected ? (
        <section className="system-game-detail">
          <div>
            <p>Vybraná hra</p>
            <h2>{selected.title}</h2>
            <div className="metadata">
              <span>{selected.releaseYear ?? "Rok neznámy"}</span>
              <span>{selected.genre ?? systemName(systemId)}</span>
            </div>
            <p>{selected.description || "Metadata sa nenašli. Názov zostal odvodený zo súboru."}</p>
          </div>
          <aside>
            <strong>Krátky prehľad</strong>
            <p>{selected.shortReview || "Pre túto hru zatiaľ nie je dostupný dôveryhodný prehľad."}</p>
            {selected.metadataSource ? <small>Zdroj: {selected.metadataSource}</small> : null}
          </aside>
        </section>
      ) : null}
      {filtered.length ? (
        <div className="library-grid">
          {filtered.map((game) => (
            <GameCard
              key={game.id}
              game={game}
              selected={game.id === selected?.id}
              onSelect={() => onSelect(game.id)}
            />
          ))}
        </div>
      ) : (
        <div className="page-empty">
          <Gamepad2Empty />
          <h2>Táto knižnica je prázdna.</h2>
          <p>Nahraj jeden herný súbor alebo pridaj celý priečinok.</p>
        </div>
      )}
    </main>
  );
}

function Gamepad2Empty() {
  return <FilePlus2 size={38} />;
}
