import { Gamepad2, HardDrive, Maximize, RotateCcw, ShieldCheck } from "lucide-react";

export function SettingsPage({ onRestartOnboarding }: { onRestartOnboarding: () => void }) {
  return (
    <main className="content-page settings-page">
      <header><p>RetroBox</p><h1>Nastavenia</h1><span>Lokálne nastavenia aplikácie a knižnice.</span></header>
      <div className="settings-list">
        <article><Maximize /><div><h2>Zobrazenie</h2><p>Desktop a fullscreen konzolový režim.</p></div><span>Desktop</span></article>
        <article><HardDrive /><div><h2>Dáta</h2><p>SQLite, médiá, logy a zálohy v lokálnom RetroBox priečinku.</p></div><span>Lokálne</span></article>
        <article><Gamepad2 /><div><h2>Ovládanie</h2><p>Klávesnica, myš a gamepad navigácia.</p></div><span>Aktívne</span></article>
        <article><ShieldCheck /><div><h2>Bezpečnosť</h2><p>Žiadny všeobecný shell prístup z frontendu.</p></div><span>Chránené</span></article>
      </div>
      <button className="secondary restart-onboarding" onClick={onRestartOnboarding}><RotateCcw size={19} /> Spustiť onboarding znova</button>
    </main>
  );
}
