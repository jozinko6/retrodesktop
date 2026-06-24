import { FolderPlus, MonitorPlay, Search } from "lucide-react";
import { useMemo, useState } from "react";
import type { Game } from "../types";
import { GameCard } from "./GameCard";
import { GameDetailDialog } from "./GameDetailDialog";

export function LibraryPage({
  games,
  selectedId,
  onSelect,
  onAddFolder,
  onWindowsScan,
  onPlay,
  onRefreshMetadata,
}: {
  games: Game[];
  selectedId: string;
  onSelect: (id: string) => void;
  onAddFolder: () => void;
  onWindowsScan: () => void;
  onPlay: (gameId: string) => Promise<void>;
  onRefreshMetadata: (gameId: string) => Promise<void>;
}) {
  const [query, setQuery] = useState("");
  const [openGameId, setOpenGameId] = useState<string>();
  const filtered = useMemo(() => {
    const normalized = query.trim().toLocaleLowerCase("sk");
    return normalized
      ? games.filter((game) => `${game.title} ${game.systemId} ${game.genre ?? ""}`.toLocaleLowerCase("sk").includes(normalized))
      : games;
  }, [games, query]);
  const openGame = games.find((game) => game.id === openGameId);

  return (
    <main className="content-page library-page">
      <header>
        <p>Knižnica</p>
        <h1>Všetky hry</h1>
        <span>{games.length} položiek v lokálnej knižnici.</span>
      </header>
      {games.length ? <label className="library-search">
          <Search size={20} />
          <input value={query} onChange={(event) => setQuery(event.target.value)} placeholder="Hľadať podľa názvu, systému alebo žánru" />
        </label> : null}
      {filtered.length ? (
        <div className="library-grid">
          {filtered.map((game) => <GameCard key={game.id} game={game} selected={game.id === selectedId} onSelect={() => { onSelect(game.id); setOpenGameId(game.id); }} />)}
        </div>
      ) : games.length ? <div className="page-empty"><Search size={36} /><h2>Nenašla sa žiadna hra.</h2><p>Skús zmeniť hľadaný výraz.</p></div> : (
        <div className="page-empty">
          <LibraryEmptyIcon />
          <h2>Knižnica je prázdna.</h2>
          <p>Pridaj priečinok s hernými zálohami alebo vyhľadaj nainštalované Windows hry.</p>
          <div className="empty-actions">
            <button className="primary" onClick={onAddFolder}><FolderPlus size={19} /> Pridať priečinok</button>
            <button className="secondary" onClick={onWindowsScan}><MonitorPlay size={19} /> Windows hry</button>
          </div>
        </div>
      )}
      {openGame ? (
        <GameDetailDialog
          game={openGame}
          onClose={() => setOpenGameId(undefined)}
          onPlay={onPlay}
          onRefreshMetadata={onRefreshMetadata}
        />
      ) : null}
    </main>
  );
}

function LibraryEmptyIcon() {
  return <FolderPlus size={38} />;
}
