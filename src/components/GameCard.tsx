import { Heart } from "lucide-react";
import type { Game } from "../types";

export function GameCard({ game, selected, onSelect }: { game: Game; selected: boolean; onSelect: () => void }) {
  return (
    <button className={`game-card ${selected ? "selected" : ""}`} onClick={onSelect} style={{ "--game-accent": game.accent } as React.CSSProperties}>
      <span className="cover-art">
        <span className="cover-orbit" />
        <strong>{game.title}</strong>
        <small>{game.systemId.toUpperCase()}</small>
      </span>
      <span className="game-card-copy">
        <strong>{game.title}</strong>
        <span>{game.genre ?? "Hra"}</span>
      </span>
      {game.favorite ? <Heart className="favorite-mark" size={18} fill="currentColor" /> : null}
    </button>
  );
}
