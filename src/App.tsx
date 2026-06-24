import { useEffect, useMemo, useState } from "react";
import { Download, Gamepad2, Grid2X2, Heart, Home, Library, MonitorPlay, Play, Search, Settings, SlidersHorizontal, UserRound, Wrench } from "lucide-react";
import { useGamepadNavigation } from "./hooks/useGamepadNavigation";
import { chooseDirectory, chooseGameFile, chooseRetroArchCore, importGameFile, launchGame, listGames, scanDirectory, scanDirectoryForSystem } from "./lib/tauri";
import type { Game, SystemId } from "./types";
import { DownloadsPage } from "./components/DownloadsPage";
import { EmulatorManager } from "./components/EmulatorManager";
import { EmptyLibrary } from "./components/EmptyLibrary";
import { GameCard } from "./components/GameCard";
import { IconButton } from "./components/IconButton";
import { LibraryPage } from "./components/LibraryPage";
import { Onboarding } from "./components/Onboarding";
import { SettingsPage } from "./components/SettingsPage";
import { SystemsPage } from "./components/SystemsPage";
import { SystemLibraryPage } from "./components/SystemLibraryPage";
import { WindowsDiscovery } from "./components/WindowsDiscovery";

type Page = "home" | "library" | "systems" | "system-library" | "windows" | "downloads" | "emulators" | "settings";

export function App() {
  const [onboarded, setOnboarded] = useState(() => localStorage.getItem("retrobox:onboarded") === "true");
  const [page, setPage] = useState<Page>("home");
  const [games, setGames] = useState<Game[]>([]);
  const [selectedId, setSelectedId] = useState("");
  const [message, setMessage] = useState<string>();
  const [selectedSystem, setSelectedSystem] = useState<SystemId>("nes");
  useGamepadNavigation(onboarded);

  useEffect(() => {
    void listGames().then((items) => {
      setGames(items);
      setSelectedId(items[0]?.id ?? "");
    });
  }, []);

  const selected = useMemo(() => games.find((game) => game.id === selectedId) ?? games[0], [games, selectedId]);

  useEffect(() => {
    const handler = (event: Event) => {
      if (!games.length) return;
      const action = (event as CustomEvent<string>).detail;
      const index = games.findIndex((game) => game.id === selectedId);
      if (action === "right" || action === "down") setSelectedId(games[(index + 1) % games.length].id);
      if (action === "left" || action === "up") setSelectedId(games[(index - 1 + games.length) % games.length].id);
      if (action === "accept") void play();
    };
    window.addEventListener("retrobox-gamepad", handler);
    return () => window.removeEventListener("retrobox-gamepad", handler);
  });

  async function play() {
    if (!selected) return;
    try {
      await launchGame(selected.id);
    } catch (error) {
      setMessage(error instanceof Error ? error.message : String(error));
    }
  }

  async function addFolder() {
    const path = await chooseDirectory();
    if (!path) return;
    try {
      const scanned = await scanDirectory(path);
      const updated = await listGames();
      setGames(updated);
      setSelectedId(updated[0]?.id ?? "");
      setMessage(`Scan dokončený: ${scanned.length} hier.`);
    } catch (error) {
      setMessage(error instanceof Error ? error.message : String(error));
    }
  }

  async function configureCore() {
    if (!selected) return;
    try {
      const configured = await chooseRetroArchCore(selected.id);
      if (configured) setMessage("RetroArch core bol priradený k hre.");
    } catch (error) {
      setMessage(error instanceof Error ? error.message : String(error));
    }
  }

  async function addGameFile(systemId: SystemId) {
    const path = await chooseGameFile();
    if (!path) return;
    try {
      const updated = await importGameFile(path, systemId);
      setGames(updated);
      const imported = updated.find((game) => game.primaryFile === path);
      setSelectedId(imported?.id ?? updated[0]?.id ?? "");
      setMessage(imported?.metadataSource
        ? `Hra bola pridaná a metadata sa našli: ${imported.title}.`
        : "Hra bola pridaná. Pre tento názov sa nenašla bezpečná zhoda metadát.");
    } catch (error) {
      setMessage(error instanceof Error ? error.message : String(error));
    }
  }

  async function addSystemFolder(systemId: SystemId) {
    const path = await chooseDirectory();
    if (!path) return;
    try {
      const updated = await scanDirectoryForSystem(path, systemId);
      setGames(updated);
      const first = updated.find((game) => game.systemId === systemId);
      setSelectedId(first?.id ?? "");
      setMessage(`Priečinok bol pridaný do knižnice ${systemId.toUpperCase()}.`);
    } catch (error) {
      setMessage(error instanceof Error ? error.message : String(error));
    }
  }

  function restartOnboarding() {
    localStorage.removeItem("retrobox:onboarded");
    setOnboarded(false);
  }

  function renderHome() {
    if (!games.length) {
      return <EmptyLibrary onAddFolder={() => void addFolder()} onWindowsScan={() => setPage("windows")} />;
    }
    return (
      <main>
        <section className="hero" style={{ "--hero-accent": selected?.accent ?? "#15d6ff" } as React.CSSProperties}>
          <div className="hero-art" aria-hidden="true"><span /><span /><span /></div>
          <div className="hero-copy">
            <p>Pokračovať v hraní</p>
            <h1>{selected?.title}</h1>
            <div className="metadata"><span>{selected?.systemId.toUpperCase()}</span><span>{selected?.releaseYear}</span><span>{selected?.genre}</span></div>
            <p className="description">{selected?.description}</p>
            <div className="hero-actions">
              <button className="primary" onClick={() => void play()}><Play size={21} fill="currentColor" /> Hrať</button>
              {selected?.systemId === "windows" ? null : <button className="secondary" onClick={() => void configureCore()}><SlidersHorizontal size={20} /> Vybrať core</button>}
            </div>
          </div>
        </section>
        <section className="rail">
          <div className="section-heading"><div><p>Knižnica</p><h2>Nedávno hrané</h2></div><span>{games.length} hier</span></div>
          <div className="game-row">
            {games.map((game) => <GameCard key={game.id} game={game} selected={game.id === selectedId} onSelect={() => setSelectedId(game.id)} />)}
          </div>
        </section>
        <section className="quick-links">
          <button onClick={() => setPage("library")}><Heart /> Obľúbené</button>
          <button onClick={() => setPage("systems")}><Grid2X2 /> Systémy</button>
          <button onClick={() => setPage("library")}><Library /> Všetky hry</button>
        </section>
      </main>
    );
  }

  function renderPage() {
    switch (page) {
      case "home":
        return renderHome();
      case "library":
        return <LibraryPage games={games} selectedId={selectedId} onSelect={setSelectedId} onAddFolder={() => void addFolder()} onWindowsScan={() => setPage("windows")} />;
      case "systems":
        return <SystemsPage games={games} onOpenSystem={(systemId) => { setSelectedSystem(systemId); setSelectedId(games.find((game) => game.systemId === systemId)?.id ?? ""); setPage("system-library"); }} />;
      case "system-library":
        return <SystemLibraryPage systemId={selectedSystem} games={games} selectedId={selectedId} onSelect={setSelectedId} onBack={() => setPage("systems")} onAddFile={() => addGameFile(selectedSystem)} onAddFolder={() => addSystemFolder(selectedSystem)} />;
      case "windows":
        return <WindowsDiscovery onImported={(items) => { setGames(items); setSelectedId(items[0]?.id ?? ""); setPage("library"); }} />;
      case "downloads":
        return <DownloadsPage />;
      case "emulators":
        return <EmulatorManager />;
      case "settings":
        return <SettingsPage onRestartOnboarding={restartOnboarding} />;
    }
  }

  if (!onboarded) {
    return <Onboarding onFinish={() => { localStorage.setItem("retrobox:onboarded", "true"); setOnboarded(true); }} />;
  }

  return (
    <div className="app-shell">
      <aside className="sidebar">
        <div className="brand"><Gamepad2 /><span>RetroBox</span></div>
        <nav>
          <IconButton label="Domov" active={page === "home"} onClick={() => setPage("home")}><Home /></IconButton>
          <IconButton label="Všetky hry" active={page === "library"} onClick={() => setPage("library")}><Library /></IconButton>
          <IconButton label="Systémy" active={page === "systems" || page === "system-library"} onClick={() => setPage("systems")}><Grid2X2 /></IconButton>
          <IconButton label="Windows hry" active={page === "windows"} onClick={() => setPage("windows")}><MonitorPlay /></IconButton>
          <IconButton label="Sťahovania" active={page === "downloads"} onClick={() => setPage("downloads")}><Download /></IconButton>
          <IconButton label="Emulátory" active={page === "emulators"} onClick={() => setPage("emulators")}><Wrench /></IconButton>
        </nav>
        <IconButton label="Nastavenia" active={page === "settings"} onClick={() => setPage("settings")}><Settings /></IconButton>
      </aside>
      <div className="main-stage">
        <header className="topbar">
          <div><Search size={20} /><span>Hľadať hry</span><kbd>Y</kbd></div>
          <button className="profile"><UserRound size={20} /><span>Rodina</span></button>
        </header>
        {renderPage()}
      </div>
      {message ? <div className="toast" role="status" onClick={() => setMessage(undefined)}>{message}</div> : null}
    </div>
  );
}
