import { render, screen } from "@testing-library/react";
import { describe, expect, it, vi } from "vitest";
import { demoGames } from "../data/catalog";
import { GameCard } from "./GameCard";

describe("GameCard", () => {
  it("exposes the title and selected state", () => {
    render(<GameCard game={demoGames[0]} selected onSelect={vi.fn()} />);
    expect(screen.getByRole("button", { name: /Neon Drift/i })).toHaveClass("selected");
  });
});
