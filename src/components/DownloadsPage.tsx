import { Download } from "lucide-react";

export function DownloadsPage() {
  return (
    <main className="content-page">
      <header><p>Prenosy</p><h1>Sťahovania</h1><span>Priebeh importov z verejných priamych odkazov.</span></header>
      <div className="page-empty">
        <Download size={38} />
        <h2>Žiadne aktívne sťahovania.</h2>
        <p>Download pipeline ešte nie je zapnutá; obrazovka preto nepredstiera falošné úlohy.</p>
      </div>
    </main>
  );
}
