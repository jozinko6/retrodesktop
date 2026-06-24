import { LoaderCircle, Play, RefreshCw, X } from "lucide-react";
import { useEffect, useState } from "react";
import { systemName } from "../data/systems";
import { localAssetUrl } from "../lib/tauri";
import type { Game } from "../types";

export function GameDetailDialog({
  game,
  onClose,
  onPlay,
  onRefreshMetadata,
}: {
  game: Game;
  onClose: () => void;
  onPlay: (gameId: string) => Promise<void>;
  onRefreshMetadata?: (gameId: string) => Promise<void>;
}) {
  const [busy, setBusy] = useState<"launch" | "metadata">();
  const [error, setError] = useState<string>();

  useEffect(() => {
    const closeOnEscape = (event: KeyboardEvent) => {
      if (event.key === "Escape") onClose();
    };
    window.addEventListener("keydown", closeOnEscape);
    return () => window.removeEventListener("keydown", closeOnEscape);
  }, [onClose]);

  async function launch() {
    setBusy("launch");
    setError(undefined);
    try {
      await onPlay(game.id);
    } catch (reason) {
      setError(reason instanceof Error ? reason.message : String(reason));
    } finally {
      setBusy(undefined);
    }
  }

  async function refresh() {
    if (!onRefreshMetadata) return;
    setBusy("metadata");
    setError(undefined);
    try {
      await onRefreshMetadata(game.id);
    } catch (reason) {
      setError(reason instanceof Error ? reason.message : String(reason));
    } finally {
      setBusy(undefined);
    }
  }

  return (
    <div className="game-detail-backdrop" role="presentation" onClick={onClose}>
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
          onClick={onClose}
        >
          <X />
        </button>
        <div
          className="game-detail-art"
          style={{ "--game-accent": game.accent } as React.CSSProperties}
        >
          {game.coverPath ? (
            <img src={localAssetUrl(game.coverPath)} alt="" />
          ) : (
            <span>{game.title}</span>
          )}
        </div>
        <div className="game-detail-copy">
          <p>{systemName(game.systemId)}</p>
          <h2 id="game-detail-title">{game.title}</h2>
          <div className="metadata">
            <span>{game.releaseYear ?? "Rok neznámy"}</span>
            <span>{game.genre ?? "Hra"}</span>
          </div>
          <p>
            {game.description ||
              (game.systemId === "windows"
                ? "Lokálne nainštalovaná Windows hra."
                : "Metadata sa nenašli. Hru môžeš spustiť alebo skúsiť obnoviť metadata.")}
          </p>
          <div className="game-detail-actions">
            <button
              className="primary"
              disabled={Boolean(busy)}
              onClick={() => void launch()}
            >
              {busy === "launch" ? (
                <LoaderCircle className="spin" size={20} />
              ) : (
                <Play size={20} fill="currentColor" />
              )}
              {busy === "launch" ? "Spúšťam…" : "Spustiť hru"}
            </button>
            {onRefreshMetadata ? (
              <button
                className="secondary"
                disabled={Boolean(busy)}
                onClick={() => void refresh()}
              >
                {busy === "metadata" ? (
                  <LoaderCircle className="spin" size={18} />
                ) : (
                  <RefreshCw size={18} />
                )}
                Obnoviť metadata
              </button>
            ) : null}
          </div>
          {error ? (
            <p className="game-launch-error" role="alert">
              {error}
            </p>
          ) : null}
        </div>
      </section>
    </div>
  );
}
