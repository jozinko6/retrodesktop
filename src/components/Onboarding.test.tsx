import { render, screen } from "@testing-library/react";
import userEvent from "@testing-library/user-event";
import { describe, expect, it, vi } from "vitest";
import { Onboarding } from "./Onboarding";

describe("Onboarding", () => {
  it("advances through setup", async () => {
    render(<Onboarding onFinish={vi.fn()} />);
    await userEvent.click(screen.getByRole("button", { name: /pokračovať/i }));
    expect(screen.getByRole("heading", { name: "Dátový priečinok" })).toBeInTheDocument();
  });
});
