import { FolderPlus, Link, MonitorPlay, Plus } from "lucide-react";

export function EmptyLibrary({ onAddFolder, onWindowsScan, onImportGame }: { onAddFolder: () => void; onWindowsScan: () => void; onImportGame: () => void }) {
  return (
    <main className="empty-library">
      <div className="empty-symbol"><FolderPlus size={44} /></div>
      <p>Knižnica</p>
      <h1>Zatiaľ nemáš pridané žiadne hry.</h1>
      <span>Vyber priečinok so svojimi vlastnými hernými zálohami. RetroBox ho bezpečne naskenuje a uloží ako sledovaný priečinok.</span>
      <div className="empty-actions">
        <button className="primary" onClick={onAddFolder}><FolderPlus size={20} /> Pridať priečinok</button>
        <button className="secondary" onClick={onWindowsScan}><MonitorPlay size={20} /> Skenovať Windows hry</button>
        <button className="secondary" onClick={onImportGame}><Plus size={20} /> Importovať hru</button>
        <button className="secondary" disabled><Link size={20} /> Pridať odkaz</button>
      </div>
      <small>Pri importe jedného súboru najprv vyberieš jeho systém. Download z odkazu bude dostupný v ďalšej fáze.</small>
    </main>
  );
}
