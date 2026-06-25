import { useEffect, useState } from "react";
import {
  AlertCircle,
  CheckCircle2,
  Download,
  ExternalLink,
  FolderOpen,
  LoaderCircle,
  Upload,
} from "lucide-react";
import {
  chooseBios,
  chooseExecutable,
  installManagedEmulator,
  listEmulators,
  openOfficialEmulatorPage,
} from "../lib/tauri";
import type { EmulatorStatus } from "../types";

export function EmulatorManager() {
  const [items, setItems] = useState<EmulatorStatus[]>([]);
  const [error, setError] = useState<string>();
  const [busyId, setBusyId] = useState<string>();

  useEffect(() => {
    void listEmulators().then(setItems);
  }, []);

  async function reload() {
    setItems(await listEmulators());
  }

  async function configure(id: string) {
    setError(undefined);
    try {
      const configured = await chooseExecutable(id);
      if (configured) {
        setItems((current) =>
          current.map((item) => (item.id === id ? configured : item)),
        );
      }
    } catch (reason) {
      setError(reason instanceof Error ? reason.message : String(reason));
    }
  }

  async function install(id: string) {
    setError(undefined);
    setBusyId(id);
    try {
      await installManagedEmulator(id);
      await reload();
    } catch (reason) {
      setError(reason instanceof Error ? reason.message : String(reason));
    } finally {
      setBusyId(undefined);
    }
  }

  async function importBios(id: string) {
    setError(undefined);
    setBusyId(id);
    try {
      if (await chooseBios(id)) await reload();
    } catch (reason) {
      setError(reason instanceof Error ? reason.message : String(reason));
    } finally {
      setBusyId(undefined);
    }
  }

  async function openOfficialPage(id: string) {
    setError(undefined);
    try {
      await openOfficialEmulatorPage(id);
    } catch (reason) {
      setError(reason instanceof Error ? reason.message : String(reason));
    }
  }

  return (
    <section className="content-page">
      <header>
        <p>Konfigurácia</p>
        <h1>Správca emulátorov</h1>
        <span>
          RetroBox môže stiahnuť podporované emulátory z ich oficiálnych
          vydaní. BIOS a firmware pridávaš iba zo svojich vlastných súborov.
        </span>
      </header>
      <div className="emulator-list">
        {items.map((item) => (
          <article className="emulator-row" key={item.id}>
            <div className="emulator-logo">
              {item.displayName.slice(0, 2).toUpperCase()}
            </div>
            <div>
              <h2>{item.displayName}</h2>
              <p>{item.supportedSystems.join(" · ")}</p>
            </div>
            <div className="emulator-state">
              <div className={`status ${item.state}`}>
                {item.state === "ready" ? (
                  <CheckCircle2 size={18} />
                ) : (
                  <AlertCircle size={18} />
                )}
                {item.state === "not-installed"
                  ? "Nenainštalované"
                  : item.state}
              </div>
              {item.version ? <small>verzia {item.version}</small> : null}
              {item.biosRequired ? (
                <small
                  className={item.biosConfigured ? "bios-ok" : "bios-missing"}
                >
                  {item.biosConfigured
                    ? "BIOS/firmware vložený"
                    : "BIOS/firmware chýba"}
                </small>
              ) : null}
            </div>
            <div className="emulator-actions">
              {item.canManagedInstall && item.state !== "ready" ? (
                <button
                  disabled={busyId === item.id}
                  onClick={() => void install(item.id)}
                >
                  {busyId === item.id ? (
                    <LoaderCircle className="spin" size={19} />
                  ) : (
                    <Download size={19} />
                  )}
                  {busyId === item.id
                    ? "Sťahujem a nastavujem…"
                    : "Nainštalovať automaticky"}
                </button>
              ) : null}
              {!item.canManagedInstall ? (
                <button onClick={() => void openOfficialPage(item.id)}>
                  <ExternalLink size={19} /> Oficiálna stránka
                </button>
              ) : null}
              {item.biosRequired ? (
                <button
                  disabled={busyId === item.id}
                  onClick={() => void importBios(item.id)}
                >
                  <Upload size={19} /> Vložiť BIOS
                </button>
              ) : null}
              <button
                aria-label={`Vybrať ${item.displayName} executable`}
                disabled={busyId === item.id}
                onClick={() => void configure(item.id)}
              >
                <FolderOpen size={19} /> Zmeniť executable
              </button>
            </div>
          </article>
        ))}
      </div>
      {error ? (
        <p className="inline-error" role="alert">
          {error}
        </p>
      ) : null}
    </section>
  );
}
