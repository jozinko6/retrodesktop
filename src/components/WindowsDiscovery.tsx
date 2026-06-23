import { useEffect, useMemo, useState } from "react";
import { Check, FolderSearch2, MonitorPlay, RefreshCw, Search, X } from "lucide-react";
import { choosePortableScanRoot, confirmWindowsGames, listWindowsCandidates, rejectWindowsGames, scanWindowsGames } from "../lib/tauri";
import type { Game, WindowsGameCandidate } from "../types";

const sourceNames: Record<string, string> = {
  steam: "Steam",
  epic: "Epic Games",
  gog: "GOG",
  registry: "Windows Registry",
  shortcut: "Windows skratky",
  portable: "Portable hry"
};

function groupCandidates(items: WindowsGameCandidate[]) {
  const grouped = new Map<string, WindowsGameCandidate[]>();
  for (const item of items) {
    const group = grouped.get(item.source);
    if (group) group.push(item); else grouped.set(item.source, [item]);
  }
  return grouped;
}

export function WindowsDiscovery({ onImported }: { onImported: (games: Game[]) => void }) {
  const [items, setItems] = useState<WindowsGameCandidate[]>([]);
  const [selected, setSelected] = useState<Set<string>>(new Set());
  const [scanning, setScanning] = useState(false);
  const [message, setMessage] = useState<string>();
  const grouped = useMemo(() => groupCandidates(items), [items]);
  const allSelected = items.length > 0 && selected.size === items.length;

  useEffect(() => { void listWindowsCandidates().then(setItems); }, []);

  async function scan(deep = false) {
    setScanning(true);
    try {
      const root = deep ? await choosePortableScanRoot() : null;
      if (deep && !root) return;
      const result = await scanWindowsGames(root ? [root] : []);
      setItems(result.candidates);
      setSelected(new Set(result.candidates.filter((item) => item.confidence >= 0.9).map((item) => item.id)));
      setMessage(`Nájdené: ${result.candidates.length}. Zdroje: ${result.scannedSources.join(", ")}.${result.warnings.length ? ` Upozornenia: ${result.warnings.join(" ")}` : ""}`);
    } catch (error) {
      setMessage(error instanceof Error ? error.message : String(error));
    } finally {
      setScanning(false);
    }
  }

  function toggle(id: string) {
    setSelected((current) => {
      const next = new Set(current);
      if (next.has(id)) next.delete(id); else next.add(id);
      return next;
    });
  }

  async function confirm() {
    const ids = [...selected];
    if (!ids.length) return;
    const games = await confirmWindowsGames(ids);
    setItems((current) => current.filter((item) => !selected.has(item.id)));
    setSelected(new Set());
    onImported(games);
  }

  async function reject() {
    const ids = [...selected];
    if (!ids.length) return;
    await rejectWindowsGames(ids);
    setItems((current) => current.filter((item) => !selected.has(item.id)));
    setSelected(new Set());
  }

  return (
    <section className="content-page windows-discovery">
      <header><p>Lokálna knižnica</p><h1>Windows Game Discovery</h1><span>Steam, Epic, GOG, ponuka Štart a voliteľný heuristický scan. Neisté zhody vždy potvrdzuješ ty.</span></header>
      <div className="discovery-toolbar">
        <button className="primary" onClick={() => void scan(false)} disabled={scanning}>{scanning ? <RefreshCw className="spin" size={20} /> : <Search size={20} />} Skenovať Windows hry</button>
        <button className="secondary" onClick={() => void scan(true)} disabled={scanning}><FolderSearch2 size={20} /> Hlboký scan priečinka</button>
        {items.length ? <label className="select-all"><input type="checkbox" checked={allSelected} onChange={() => setSelected(allSelected ? new Set() : new Set(items.map((item) => item.id)))} /> Vybrať všetko</label> : null}
      </div>
      {message ? <p className="discovery-message" role="status">{message}</p> : null}
      {!items.length ? (
        <div className="discovery-empty"><MonitorPlay size={42} /><h2>Spusti scan a skontroluj nájdené hry.</h2><p>Portable scan je oddelený, aby RetroBox neprehľadával disky bez tvojho rozhodnutia.</p></div>
      ) : (
        <div className="candidate-groups">
          {[...grouped.entries()].map(([source, candidates]) => (
            <section key={source}>
              <h2>{sourceNames[source] ?? source} <span>{candidates.length}</span></h2>
              <div className="candidate-list">
                {candidates.map((item) => (
                  <label className={`candidate-row ${selected.has(item.id) ? "selected" : ""}`} key={item.id}>
                    <input type="checkbox" checked={selected.has(item.id)} onChange={() => toggle(item.id)} />
                    <span className="candidate-icon"><MonitorPlay size={24} /></span>
                    <span className="candidate-copy"><strong>{item.title}</strong><small>{item.installPath ?? item.launchTarget}</small><em>{item.evidence.join(" · ")}</em></span>
                    <span className={`confidence ${item.confidence >= 0.9 ? "high" : item.confidence >= 0.7 ? "medium" : "low"}`}>{Math.round(item.confidence * 100)} %</span>
                  </label>
                ))}
              </div>
            </section>
          ))}
        </div>
      )}
      {selected.size ? <div className="candidate-actions"><span>{selected.size} vybraných</span><button className="secondary" onClick={() => void reject()}><X size={19} /> Zamietnuť</button><button className="primary" onClick={() => void confirm()}><Check size={19} /> Pridať do knižnice</button></div> : null}
    </section>
  );
}
