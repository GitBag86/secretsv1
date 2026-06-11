import { describe, it, expect } from "vitest";
import { render, screen } from "@testing-library/react";
import { createElement } from "react";
import { QueryClient, QueryClientProvider } from "@tanstack/react-query";

// Mock Tauri API
vi.mock("@tauri-apps/api/core", () => ({ invoke: vi.fn() }));

// Mock next/navigation for SearchPalette (rendered inside NavHeader)
vi.mock("next/navigation", () => ({
  useRouter: () => ({ push: vi.fn(), replace: vi.fn(), prefetch: vi.fn(), back: vi.fn(), forward: vi.fn(), refresh: vi.fn() }),
  usePathname: () => "/",
  useSearchParams: () => new URLSearchParams(),
}));

vi.mock("@/hooks", () => ({
  useAuth: () => ({
    user: null,
    isLoggedIn: true,
    isUnlocked: true,
    isHydrated: true,
    sessionElapsed: 0,
    sessionMinutes: 15,
    setUser: vi.fn(),
    setLoggedIn: vi.fn(),
    setUnlocked: vi.fn(),
    setHydrated: vi.fn(),
    setSessionElapsed: vi.fn(),
    setSessionMinutes: vi.fn(),
    logout: vi.fn(),
    lock: vi.fn(),
    unlock: vi.fn(),
    setupMasterPassword: vi.fn(),
    bootstrap: vi.fn(),
  }),
}));

import Home from "@/app/page";

function createDashboardWrapper() {
  const qc = new QueryClient({ defaultOptions: { queries: { retry: false } } });
  qc.setQueryData(["notes"], []);
  qc.setQueryData(["todos"], []);
  qc.setQueryData(["notebooks"], []);
  qc.setQueryData(["calendar"], []);
  qc.setQueryData(["tags"], []);
  qc.setQueryData(["all-todo-tags"], []);
  qc.setQueryData(["all-note-tags"], []);
  return function Wrapper({ children }: { children: React.ReactNode }) {
    return createElement(QueryClientProvider, { client: qc }, children);
  };
}

describe("Dashboard page", () => {
  it("renders dashboard heading", () => {
    render(createElement(createDashboardWrapper(), null, createElement(Home)));
    expect(screen.getByText("Dashboard")).toBeDefined();
  });

  it("renders navigation links", () => {
    render(createElement(createDashboardWrapper(), null, createElement(Home)));
    expect(screen.getAllByText("Notes").length).toBeGreaterThanOrEqual(1);
    expect(screen.getAllByText("Todos").length).toBeGreaterThanOrEqual(1);
    expect(screen.getAllByText("Calendar").length).toBeGreaterThanOrEqual(1);
  });

  it("renders stat cards", () => {
    render(createElement(createDashboardWrapper(), null, createElement(Home)));
    expect(screen.getAllByText("Notes").length).toBeGreaterThanOrEqual(1);
    expect(screen.getAllByText("Events").length).toBeGreaterThanOrEqual(1);
    expect(screen.getAllByText("0").length).toBeGreaterThanOrEqual(3);
  });

  it("renders empty states", () => {
    render(createElement(createDashboardWrapper(), null, createElement(Home)));
    expect(screen.getByText("No events in the next 7 days.")).toBeDefined();
    expect(screen.getByText("No notes yet.")).toBeDefined();
    expect(screen.getByText("No tags yet.")).toBeDefined();
  });

  it("renders section headings", () => {
    render(createElement(createDashboardWrapper(), null, createElement(Home)));
    expect(screen.getByText("Todo Progress")).toBeDefined();
    expect(screen.getByText("Upcoming Events")).toBeDefined();
    expect(screen.getByText("Recent Notes")).toBeDefined();
    expect(screen.getByText("Popular Tags")).toBeDefined();
  });

  it("shows todo progress when data is available", () => {
    const qc = new QueryClient({ defaultOptions: { queries: { retry: false } } });
    qc.setQueryData(["notes"], []);
    qc.setQueryData(["todos"], [
      { id: "1", title: "Task 1", is_completed: true, priority: "high", created_at: 1700000000, updated_at: 1700000000 },
      { id: "2", title: "Task 2", is_completed: false, priority: "low", created_at: 1700000000, updated_at: 1700000000 },
    ]);
    qc.setQueryData(["notebooks"], []);
    qc.setQueryData(["calendar"], []);
    qc.setQueryData(["tags"], []);
    qc.setQueryData(["all-todo-tags"], []);
    qc.setQueryData(["all-note-tags"], []);
    render(createElement(
      (props: any) => createElement(QueryClientProvider, { client: qc }, props.children),
      null,
      createElement(Home),
    ));
    expect(screen.getByText("1 of 2 tasks completed")).toBeDefined();
    expect(screen.getByText("50% complete")).toBeDefined();
  });
});
