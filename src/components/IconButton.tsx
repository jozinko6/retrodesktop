import type { ReactNode } from "react";

export function IconButton({ label, children, active, onClick }: { label: string; children: ReactNode; active?: boolean; onClick: () => void }) {
  return (
    <button className={`icon-button ${active ? "active" : ""}`} aria-label={label} title={label} onClick={onClick}>
      {children}
      <span>{label}</span>
    </button>
  );
}
