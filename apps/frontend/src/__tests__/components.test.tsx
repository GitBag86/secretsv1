import { describe, it, expect } from "vitest";
import { render, screen } from "@testing-library/react";
import { createElement } from "react";
import { QueryClient, QueryClientProvider } from "@tanstack/react-query";

const queryClient = new QueryClient({ defaultOptions: { queries: { retry: false } } });

function Wrapper({ children }: { children: React.ReactNode }) {
  return createElement(QueryClientProvider, { client: queryClient }, children);
}

function renderWithQ(cmp: React.ReactElement) {
  return render(createElement(Wrapper, null, cmp));
}

// Mock Tauri API
vi.mock("@tauri-apps/api/core", () => ({ invoke: vi.fn() }));

// We test the page components in isolation by mocking the hooks
vi.mock("@/hooks", () => ({
  useAuth: () => ({
    user: null,
    isUnlocked: true,
    lock: vi.fn(),
    sessionElapsed: 0,
    sessionMinutes: 15,
    setSessionElapsed: vi.fn(),
    setSessionMinutes: vi.fn(),
  }),
  useNotebooks: () => ({
    notebooks: [],
    isLoading: false,
    create: { mutateAsync: vi.fn(), isPending: false },
    update: { mutateAsync: vi.fn() },
    remove: { mutateAsync: vi.fn() },
  }),
  useNotes: () => ({
    notes: [
      { id: "1", title: "Test Note", content: "Content", word_count: 5, reading_time: 1, is_pinned: false, is_archived: false },
    ],
    isLoading: false,
    create: { mutateAsync: vi.fn(), isPending: false },
    update: { mutateAsync: vi.fn(), isPending: false },
    remove: { mutateAsync: vi.fn() },
  }),
  useTodos: () => ({
    todos: [
      { id: "1", title: "Buy milk", is_completed: false, priority: "high" },
      { id: "2", title: "Done task", is_completed: true, priority: "low" },
    ],
    isLoading: false,
    create: { mutateAsync: vi.fn(), isPending: false },
    update: { mutateAsync: vi.fn(), isPending: false },
    remove: { mutateAsync: vi.fn() },
    bulkUpdate: { mutate: vi.fn(), isPending: false },
    bulkDelete: { mutate: vi.fn(), isPending: false },
  }),
  useCalendar: () => ({
    events: [
      { id: "e1", title: "Meeting", start_time: 1700000000, end_time: 1700003600, all_day: false, color: "#3b82f6" },
    ],
    isLoading: false,
    create: { mutateAsync: vi.fn(), isPending: false },
    remove: { mutateAsync: vi.fn() },
  }),
}));

import Home from "@/app/page";
import NotesPage from "@/app/notes/page";
import TodosPage from "@/app/todos/page";
import CalendarPage from "@/app/calendar/page";

describe("Dashboard page", () => {
  it("renders dashboard heading", () => {
    render(createElement(Home));
    expect(screen.getByText("Dashboard")).toBeDefined();
  });

  it("renders navigation links", () => {
    render(createElement(Home));
    expect(screen.getAllByText("Notes").length).toBeGreaterThanOrEqual(1);
    expect(screen.getAllByText("Todos").length).toBeGreaterThanOrEqual(1);
    expect(screen.getAllByText("Calendar").length).toBeGreaterThanOrEqual(1);
  });

  it("renders feature cards", () => {
    render(createElement(Home));
    expect(screen.getByText("Create and organize your thoughts")).toBeDefined();
    expect(screen.getByText("Track your tasks and goals")).toBeDefined();
    expect(screen.getByText("Manage your schedule")).toBeDefined();
  });
});

describe("Notes page", () => {
  it("renders notes heading", () => {
    renderWithQ(createElement(NotesPage));
    expect(screen.getAllByText("Notes").length).toBeGreaterThanOrEqual(1);
  });

  it("displays note cards", () => {
    renderWithQ(createElement(NotesPage));
    expect(screen.getByText("Test Note")).toBeDefined();
  });

  it("shows word count and reading time", () => {
    renderWithQ(createElement(NotesPage));
    expect(screen.getByText("5 words · 1 min read")).toBeDefined();
  });
});

describe("Todos page", () => {
  it("renders todos heading", () => {
    render(createElement(TodosPage));
    expect(screen.getAllByText("Todos").length).toBeGreaterThanOrEqual(1);
  });

  it("displays todo items", () => {
    render(createElement(TodosPage));
    expect(screen.getByText("Buy milk")).toBeDefined();
    expect(screen.getByText("Done task")).toBeDefined();
  });

  it("shows priority badges", () => {
    render(createElement(TodosPage));
    expect(screen.getByText("high")).toBeDefined();
    expect(screen.getByText("low")).toBeDefined();
  });
});

describe("Calendar page", () => {
  it("renders calendar heading and buttons", () => {
    render(createElement(CalendarPage));
    expect(screen.getAllByText("Calendar").length).toBeGreaterThanOrEqual(1);
    expect(screen.getByText("Today")).toBeDefined();
  });
});
