import { ArrowLeft, FilePlus2, FolderPlus, LoaderCircle, Play, RefreshCw, X } from "lucide-react";
import { useEffect, useMemo, useState } from "react";
import { systemName } from "../data/systems";
import { localAssetUrl } from "../lib/tauri";
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
  onRefreshMetadata,
  onPlay,
}: {
  systemId: SystemId;
  games: Game[];
  selectedId: string;
  onSelect: (id: string) => void;
  onBack: () => void;
  onAddFile: () => Promise<void>;
  onAddFolder: () => Promise<void>;
  onRefreshMetadata: (gameId: string) => Promise<void>;
  onPlay: (gameId: string) => Promise<void>;
}) {
  const [busy, setBusy] = useState<"file" | "folder" | "metadata" | "launch">();
  const [openGameId, setOpenGameId] = useState<string>();
  const [launchError, setLaunchError] = useState<string>();
  const filtered = useMemo(
    () => games.filter((game) => game.systemId === systemId),
    [games, systemId],
  );
  const selected =
    filtered.find((game) => game.id === selectedId) ?? filtered[0];
  const openGame = filtered.find((game) => game.id === openGameId);

  useEffect(() => {
    if (!openGameId) return;
    const closeOnEscape = (event: KeyboardEvent) => {
      if (event.key === "Escape") setOpenGameId(undefined);
    };
    window.addEventListener("keydown", closeOnEscape);
    return () => window.removeEventListener("keydown", closeOnEscape);
  }, [openGameId]);

  async function run(kind: "file" | "folder" | "metadata" | "launch", action: () => Promise<void>) {
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
          {selected.coverPath ? (
            <img
              className="system-detail-cover"
              src={localAssetUrl(selected.coverPath)}
              alt={`Obal hry ${selected.title}`}
            />
          ) : null}
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
            <button
              className="metadata-refresh"
              disabled={Boolean(busy)}
              onClick={() => void run("metadata", () => onRefreshMetadata(selected.id))}
            >
              {busy === "metadata" ? <LoaderCircle className="spin" size={17} /> : <RefreshCw size={17} />}
              Obnoviť metadata a obrázok
            </button>
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
              onSelect={() => {
                onSelect(game.id);
                setLaunchError(undefined);
                setOpenGameId(game.id);
              }}
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
      {openGame ? (
        <div
          className="game-detail-backdrop"
          role="presentation"
          onClick={() => setOpenGameId(undefined)}
        >
          <section
            className="game-detail-dialog"
            role="dialog"
            aria-modal="true"
            aria-labelledby="game-detail-title"
            onClick={(event) => event.stopPropagation()}
          >
            <button
              className="game-detail-close"
              aria-label="Zavrieť detail hry"
              onClick={() => setOpenGameId(undefined)}
            >
              <X />
            </button>
            <div
              className="game-detail-art"
              style={{ "--game-accent": openGame.accent } as React.CSSProperties}
            >
              {openGame.coverPath ? (
                <img src={localAssetUrl(openGame.coverPath)} alt="" />
              ) : (
                <span>{openGame.title}</span>
              )}
            </div>
            <div className="game-detail-copy">
              <p>{systemName(openGame.systemId)}</p>
              <h2 id="game-detail-title">{openGame.title}</h2>
              <div className="metadata">
                <span>{openGame.releaseYear ?? "Rok neznámy"}</span>
                <span>{openGame.genre ?? "Hra"}</span>
              </div>
              <p>
                {openGame.description ||
                  "Metadata sa nenašli. Hru môžeš spustiť alebo skúsiť obnoviť metadata."}
              </p>
              <div className="game-detail-actions">
                <button
                  className="primary"
                  disabled={busy === "launch"}
                  onClick={() => {
                    setLaunchError(undefined);
                    void run("launch", () => onPlay(openGame.id)).catch((reason) => {
                      setLaunchError(reason instanceof Error ? reason.message : String(reason));
                    });
                  }}
                >
                  {busy === "launch" ? (
                    <LoaderCircle className="spin" size={20} />
                  ) : (
                    <Play size={20} fill="currentColor" />
                  )}
                  {busy === "launch" ? "Spúšťam…" : "Spustiť hru"}
                </button>
                <button
                  className="secondary"
                  disabled={Boolean(busy)}
                  onClick={() =>
                    void run("metadata", () => onRefreshMetadata(openGame.id))
                  }
                >
                  {busy === "metadata" ? (
                    <LoaderCircle className="spin" size={18} />
                  ) : (
                    <RefreshCw size={18} />
                  )}
                  Obnoviť metadata
                </button>
              </div>
              {launchError ? (
                <p className="game-launch-error" role="alert">{launchError}</p>
              ) : null}
            </div>
          </section>
        </div>
      ) : null}
    </main>
  );
}

function Gamepad2Empty() {
  return <FilePlus2 size={38} />;
}
