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
      { id: "1", title: "Test Note", content: "Content", word_count: 5, reading_time: 1, is_pinned: false, is_archived: false, user_id: "u1", created_at: 1700000000, updated_at: 1700000000 },
    ],
    isLoading: false,
    create: { mutateAsync: vi.fn(), isPending: false },
    update: { mutateAsync: vi.fn(), isPending: false },
    remove: { mutateAsync: vi.fn() },
  }),
  useTodos: () => ({
    todos: [
      { id: "1", title: "Buy milk", is_completed: false, priority: "high", created_at: 1700000000, updated_at: 1700000000 },
      { id: "2", title: "Done task", is_completed: true, priority: "low", created_at: 1700000000, updated_at: 1700000000 },
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
      { id: "e1", title: "Meeting", start_time: 1700000000, end_time: 1700003600, all_day: false, color: "#3b82f6", user_id: "u1", created_at: 1700000000, updated_at: 1700000000 },
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

// The dashboard uses direct useQuery calls rather than wrappers, so we need
// a QueryClientProvider with pre-populated data to prevent it from hanging.
function createDashboardWrapper() {
  const qc = new QueryClient({
    defaultOptions: { queries: { retry: false } },
  });
  // Pre-populate all queries the dashboard uses
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
    expect(screen.getByText("Notes")).toBeDefined();
    expect(screen.getByText("Events")).toBeDefined();
    // All stat cards show 0 since we seeded empty data
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
    const qc = new QueryClient({
      defaultOptions: { queries: { retry: false } },
    });
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
