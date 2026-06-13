import { describe, it, expect, vi } from "vitest";
import { render, screen } from "@testing-library/react";
import { createElement } from "react";

// Mock FullCalendar (very memory-heavy in jsdom)
vi.mock("@fullcalendar/react", () => ({
  default: () => createElement("div"),
}));
vi.mock("@fullcalendar/daygrid", () => ({ default: {} }));
vi.mock("@fullcalendar/timegrid", () => ({ default: {} }));
vi.mock("@fullcalendar/interaction", () => ({ default: {} }));
vi.mock("@fullcalendar/rrule", () => ({ default: {} }));
vi.mock("@fullcalendar/core", () => ({}));

// Mock Tauri API
vi.mock("@tauri-apps/api/core", () => ({ invoke: vi.fn() }));

// Mock api.sync for nav-header sync indicator
vi.mock("@/lib/api", () => ({
  api: {
    sync: { status: () => Promise.resolve({ pending: 0, last_sync: null, configured: false }) },
    notes: { list: () => Promise.resolve([]) },
    todos: { list: () => Promise.resolve([]) },
    notebooks: { list: () => Promise.resolve([]) },
    calendar: { list: () => Promise.resolve([]) },
    tags: { list: () => Promise.resolve([]) },
  },
}));

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
  useCalendar: () => ({
    events: [
      { id: "e1", title: "Meeting", start_time: 1700000000, end_time: 1700003600, all_day: false, color: "#3b82f6", user_id: "u1", created_at: 1700000000, updated_at: 1700000000 },
    ],
    isLoading: false,
    create: { mutateAsync: vi.fn(), isPending: false },
    remove: { mutateAsync: vi.fn() },
  }),
}));

import CalendarPage from "@/app/calendar/page";

describe("Calendar page", () => {
  it("renders calendar heading and buttons", () => {
    render(createElement(CalendarPage));
    expect(screen.getAllByText("Calendar").length).toBeGreaterThanOrEqual(1);
    expect(screen.getByText("Today")).toBeDefined();
  });
});
