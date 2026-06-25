import {
  CheckCircle2,
  Download,
  ExternalLink,
  FolderOpen,
  HardDriveDownload,
  LoaderCircle,
  ShieldCheck,
  X,
} from "lucide-react";
import { useEffect, useMemo, useState } from "react";
import {
  chooseDirectory,
  fetchCatalog,
  getDownloadDirectory,
  openCatalogTarget,
  requestCatalogDownload,
  subscribeCatalogDownloadProgress,
} from "../lib/tauri";
import type { CatalogDownloadProgress, CatalogGame, Game } from "../types";

interface DownloadsPageProps {
  onDownloaded?: (game: Game) => void;
}

const formatBytes = (bytes: number) =>
  new Intl.NumberFormat("sk-SK", { style: "unit", unit: "megabyte", maximumFractionDigits: 2 })
    .format(bytes / 1024 / 1024);

export function DownloadsPage({ onDownloaded }: DownloadsPageProps) {
  const [catalog, setCatalog] = useState<CatalogGame[]>([]);
  const [selected, setSelected] = useState<CatalogGame>();
  const [directory, setDirectory] = useState<string>();
  const [progress, setProgress] = useState<Record<string, CatalogDownloadProgress>>({});
  const [busy, setBusy] = useState(false);
  const [message, setMessage] = useState<string>();

  useEffect(() => {
    void Promise.all([fetchCatalog(), getDownloadDirectory()]).then(([items, savedDirectory]) => {
      setCatalog(items);
      setDirectory(savedDirectory ?? undefined);
    }).catch((reason) => setMessage(reason instanceof Error ? reason.message : String(reason)));
    let active = true;
    let unsubscribe: () => void = () => undefined;
    void subscribeCatalogDownloadProgress((item) => {
      if (active) setProgress((current) => ({ ...current, [item.gameId]: item }));
    }).then((stop) => { unsubscribe = stop; });
    return () => {
      active = false;
      unsubscribe();
    };
  }, []);

  const activeProgress = useMemo(
    () => selected ? progress[selected.id] : undefined,
    [progress, selected],
  );

  async function selectDirectory() {
    const path = await chooseDirectory();
    if (path) setDirectory(path);
  }

  async function confirmDownload() {
    if (!selected) return;
    if (!directory) {
      await selectDirectory();
      return;
    }
    setBusy(true);
    setMessage(undefined);
    try {
      const result = await requestCatalogDownload(selected.id, directory);
      setMessage(`${result.game.title} bola overená a pridaná do knižnice.`);
      onDownloaded?.(result.game);
      setSelected(undefined);
    } catch (reason) {
      setMessage(reason instanceof Error ? reason.message : String(reason));
    } finally {
      setBusy(false);
    }
  }

  return (
    <main className="content-page downloads-page">
      <header>
        <p>Legálny katalóg</p>
        <h1>Sťahovania</h1>
        <span>Ručne overené public-domain a otvorene licencované homebrew hry.</span>
      </header>

      <section className="catalog-notice">
        <ShieldCheck />
        <div>
          <h2>Žiadny automatický scraping ROM webov</h2>
          <p>Každý súbor má pripnutý zdroj, licenciu, presnú veľkosť a SHA-256. Pred uložením sa kontroluje aj skutočný formát ROM.</p>
        </div>
        <button className="secondary" onClick={() => void selectDirectory()}>
          <FolderOpen size={18} /> {directory ? "Zmeniť priečinok" : "Vybrať priečinok"}
        </button>
      </section>

      {directory ? <p className="download-directory"><HardDriveDownload size={17} /> {directory}</p> : null}
      {message ? <p className="discovery-message" role="status">{message}</p> : null}

      <section className="catalog-grid" aria-label="Katalóg hier">
        {catalog.map((game) => {
          const current = progress[game.id];
          const percent = current ? Math.min(100, Math.round(current.bytesDownloaded / current.totalBytes * 100)) : 0;
          return (
            <article className="catalog-card" key={game.id}>
              <div className="catalog-art" aria-hidden="true">
                <span>{game.systemId.toUpperCase()}</span>
                <strong>{game.title.slice(0, 2).toUpperCase()}</strong>
              </div>
              <div className="catalog-card-copy">
                <div className="catalog-tags"><span>{game.license}</span><span>{formatBytes(game.fileSize)}</span></div>
                <h2>{game.title}</h2>
                <p>{game.description}</p>
                <small>{game.developer} · {game.genre}</small>
                {current?.status === "downloading" ? (
                  <div className="download-progress" aria-label={`Sťahovanie ${percent} %`}>
                    <span style={{ width: `${percent}%` }} />
                  </div>
                ) : null}
                <button className="primary" disabled={current?.status === "downloading"} onClick={() => setSelected(game)}>
                  {current?.status === "downloading" ? <LoaderCircle className="spin" /> : current?.status === "completed" ? <CheckCircle2 /> : <Download />}
                  {current?.status === "downloading" ? `${percent} %` : "Skontrolovať a stiahnuť"}
                </button>
              </div>
            </article>
          );
        })}
      </section>

      {selected ? (
        <div className="download-confirm-backdrop" role="presentation">
          <section className="download-confirm" role="dialog" aria-modal="true" aria-labelledby="download-confirm-title">
            <button className="dialog-close" aria-label="Zavrieť" disabled={busy} onClick={() => setSelected(undefined)}><X /></button>
            <p>Potvrdenie sťahovania</p>
            <h2 id="download-confirm-title">{selected.title}</h2>
            <dl>
              <div><dt>Licencia</dt><dd>{selected.license}</dd></div>
              <div><dt>Zdroj</dt><dd>{selected.sourceUrl}</dd></div>
              <div><dt>Veľkosť</dt><dd>{formatBytes(selected.fileSize)}</dd></div>
              <div><dt>SHA-256</dt><dd className="hash-value">{selected.sha256}</dd></div>
              <div><dt>Cieľ</dt><dd>{directory ?? "Priečinok ešte nie je vybraný"}</dd></div>
            </dl>
            {activeProgress?.status === "downloading" ? (
              <div className="download-progress"><span style={{ width: `${Math.round(activeProgress.bytesDownloaded / activeProgress.totalBytes * 100)}%` }} /></div>
            ) : null}
            <p className="license-confirmation">
              Pokračovaním potvrdzuješ stiahnutie tejto konkrétnej licencovanej položky z uvedeného zdroja.
            </p>
            <div className="download-confirm-actions">
              <button className="secondary" onClick={() => void openCatalogTarget(selected.id, "license")}><ExternalLink size={17} /> Licencia</button>
              <button className="secondary" disabled={busy} onClick={() => void selectDirectory()}><FolderOpen size={17} /> Vybrať cieľ</button>
              <button className="primary" disabled={busy} onClick={() => void confirmDownload()}>
                {busy ? <LoaderCircle className="spin" /> : <Download />} {directory ? "Potvrdiť a stiahnuť" : "Vybrať priečinok"}
              </button>
            </div>
          </section>
        </div>
      ) : null}
    </main>
  );
}
