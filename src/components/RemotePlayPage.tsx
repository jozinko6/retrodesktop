import {
  Cast,
  CheckCircle2,
  ExternalLink,
  Gamepad2,
  LoaderCircle,
  MonitorSmartphone,
  Play,
  RefreshCw,
  ShieldCheck,
  Smartphone,
  Users,
  Wifi,
} from "lucide-react";
import { useEffect, useState } from "react";
import {
  getRemotePlayStatus,
  openRemotePlayTarget,
  startRemotePlayHost,
} from "../lib/tauri";
import type { RemotePlayStatus } from "../types";

export function RemotePlayPage() {
  const [status, setStatus] = useState<RemotePlayStatus>();
  const [busy, setBusy] = useState(false);
  const [message, setMessage] = useState<string>();

  async function refresh() {
    setStatus(await getRemotePlayStatus());
  }

  useEffect(() => {
    void refresh();
  }, []);

  async function startHost() {
    setBusy(true);
    setMessage(undefined);
    try {
      const current = await startRemotePlayHost();
      setStatus(current);
      setMessage(current.running ? "Sunshine host je pripravený." : "Sunshine sa spustil, ale Web UI zatiaľ neodpovedá.");
    } catch (reason) {
      setMessage(reason instanceof Error ? reason.message : String(reason));
    } finally {
      setBusy(false);
    }
  }

  return (
    <main className="content-page remote-play-page">
      <header>
        <p>Streamovanie</p>
        <h1>Remote hranie</h1>
        <span>Streamuj RetroBox do Androidu, iPhonu, tabletu alebo TV cez Sunshine a Moonlight.</span>
      </header>

      <section className="remote-status-card">
        <div className={`remote-status-icon ${status?.running ? "ready" : ""}`}>
          {status?.running ? <CheckCircle2 /> : <Cast />}
        </div>
        <div>
          <p>Sunshine host</p>
          <h2>{status?.running ? "Pripravený na pripojenie" : status?.installed ? "Nainštalovaný, ale neaktívny" : "Nie je nainštalovaný"}</h2>
          <span>
            {status?.localIp
              ? `Mobil pridaj cez adresu ${status.localIp}.`
              : "PC aj mobil musia byť v rovnakej lokálnej sieti."}
          </span>
          {status?.executable ? <small>{status.executable}</small> : null}
        </div>
        <div className="remote-status-actions">
          {status?.installed && !status.running ? (
            <button className="primary" disabled={busy} onClick={() => void startHost()}>
              {busy ? <LoaderCircle className="spin" /> : <Play />}
              Spustiť host
            </button>
          ) : null}
          {status?.running ? (
            <button className="primary" onClick={() => void openRemotePlayTarget("web-ui")}>
              <MonitorSmartphone /> Otvoriť párovanie
            </button>
          ) : null}
          {!status?.installed ? (
            <button className="primary" onClick={() => void openRemotePlayTarget("sunshine-download")}>
              <ExternalLink /> Stiahnuť Sunshine
            </button>
          ) : null}
          <button className="secondary" onClick={() => void refresh()}>
            <RefreshCw /> Obnoviť stav
          </button>
        </div>
      </section>

      {message ? <p className="discovery-message" role="status">{message}</p> : null}

      <section className="remote-steps">
        <article>
          <span>1</span>
          <Smartphone />
          <h2>Nainštaluj Moonlight</h2>
          <p>Do mobilu alebo TV nainštaluj oficiálneho Moonlight klienta.</p>
          <button onClick={() => void openRemotePlayTarget("moonlight-download")}>
            <ExternalLink size={18} /> Moonlight klienti
          </button>
        </article>
        <article>
          <span>2</span>
          <Wifi />
          <h2>Pridaj tento počítač</h2>
          <p>V Moonlighte pridaj IP adresu {status?.localIp ?? "tohto PC"} a vyžiadaj PIN.</p>
        </article>
        <article>
          <span>3</span>
          <ShieldCheck />
          <h2>Potvrď PIN</h2>
          <p>V Sunshine Web UI otvor sekciu PIN a potvrď kód z mobilu.</p>
        </article>
        <article>
          <span>4</span>
          <Gamepad2 />
          <h2>Spusti Desktop</h2>
          <p>V Moonlighte otvor Desktop, potom v RetroBoxe vyber a spusti hru.</p>
        </article>
      </section>

      <section className="remote-multiplayer">
        <div>
          <Users />
          <h2>Remote multiplayer</h2>
        </div>
        <ul>
          <li><strong>Odporúčané:</strong> pripoj viac Bluetooth gamepadov k jednému mobilu, tabletu alebo TV.</li>
          <li>Hra musí sama podporovať lokálny split-screen alebo couch co-op.</li>
          <li>Viac samostatných telefónov môže fungovať cez súbežné Sunshine relácie, ale mapovanie hráčov závisí od hry a klientov.</li>
          <li>Pre hranie mimo domácej siete použi radšej VPN; RetroBox automaticky neotvára porty do internetu.</li>
        </ul>
      </section>
    </main>
  );
}
