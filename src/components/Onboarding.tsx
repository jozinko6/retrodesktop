import { useState } from "react";
import { Check, ChevronRight, Gamepad2, HardDrive, Sparkles } from "lucide-react";

const steps = ["Vitaj v RetroBoxe", "Dátový priečinok", "Gamepad", "Emulátory", "BIOS", "Scraping", "Priečinky s hrami", "Prvý scan"];

export function Onboarding({ onFinish }: { onFinish: () => void }) {
  const [step, setStep] = useState(0);
  const icons = [Sparkles, HardDrive, Gamepad2];
  const Icon = icons[Math.min(step, 2)] ?? Check;

  return (
    <main className="onboarding">
      <div className="onboarding-progress" aria-label={`Krok ${step + 1} z ${steps.length}`}>
        {steps.map((name, index) => <span key={name} className={index <= step ? "done" : ""} />)}
      </div>
      <div className="onboarding-icon"><Icon size={44} /></div>
      <p>Krok {step + 1} z {steps.length}</p>
      <h1>{steps[step]}</h1>
      <p className="onboarding-copy">
        {step === 0
          ? "Lokálna knižnica vašich vlastných herných záloh. Bez ROM katalógu, reklám a cloudového účtu."
          : "Tento krok môžete nakonfigurovať teraz alebo bezpečne preskočiť a vrátiť sa k nemu v nastaveniach."}
      </p>
      <div className="onboarding-actions">
        <button className="secondary" onClick={onFinish}>Preskočiť sprievodcu</button>
        <button className="primary" onClick={() => step === steps.length - 1 ? onFinish() : setStep((value) => value + 1)}>
          {step === steps.length - 1 ? "Dokončiť" : "Pokračovať"} <ChevronRight size={20} />
        </button>
      </div>
    </main>
  );
}
