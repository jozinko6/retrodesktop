import { useEffect, useState } from "react";
import { AlertCircle, CheckCircle2, FolderOpen, RefreshCw } from "lucide-react";
import { listEmulators } from "../lib/tauri";
import type { EmulatorStatus } from "../types";

export function EmulatorManager() {
  const [items, setItems] = useState<EmulatorStatus[]>([]);
  useEffect(() => { void listEmulators().then(setItems); }, []);
  return (
    <section className="content-page">
      <header><p>Konfigurácia</p><h1>Správca emulátorov</h1><span>RetroBox spúšťa iba overené executable cesty, nikdy ľubovoľný shell príkaz.</span></header>
      <div className="emulator-list">
        {items.map((item) => (
          <article className="emulator-row" key={item.id}>
            <div className="emulator-logo">{item.displayName.slice(0, 2).toUpperCase()}</div>
            <div><h2>{item.displayName}</h2><p>{item.supportedSystems.join(" · ")}</p></div>
            <div className={`status ${item.state}`}>
              {item.state === "ready" ? <CheckCircle2 size={18} /> : <AlertCircle size={18} />}
              {item.state === "not-installed" ? "Nenainštalované" : item.state}
            </div>
            <button aria-label={`Vybrať ${item.displayName} executable`}><FolderOpen size={19} /> Zmeniť executable</button>
            <button aria-label={`Obnoviť ${item.displayName}`}><RefreshCw size={19} /></button>
          </article>
        ))}
      </div>
    </section>
  );
}
