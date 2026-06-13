import { describe, it, expect, vi } from "vitest";
import { render, screen } from "@testing-library/react";
import { createElement } from "react";
import { QueryClient, QueryClientProvider } from "@tanstack/react-query";
import { useQuery } from "@tanstack/react-query";

// Mock api for nav-header sync indicator
vi.mock("@/lib/api", () => ({
  api: {
    sync: { status: () => Promise.resolve({ pending: 0, last_sync: null, configured: false }) },
    notes: { list: () => Promise.resolve([]) },
    todos: { list: () => Promise.resolve([]) },
    notebooks: { list: () => Promise.resolve([]) },
    calendar: { list: () => Promise.resolve([]) },
    tags: {
      list: () => Promise.resolve([]),
      listAllTodoTags: () => Promise.resolve([]),
      listAllNoteTags: () => Promise.resolve([]),
    },
    recurringTodos: { list: () => Promise.resolve([]) },
  },
}));

const queryClient = new QueryClient({ defaultOptions: { queries: { retry: false } } });

vi.mock("@tanstack/react-query", async () => {
  const actual = await vi.importActual("@tanstack/react-query");
  return { ...actual, useQuery: vi.fn(() => ({ data: [] })) };
});

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
  useNotes: () => ({
    notes: [],
    isLoading: false,
    create: { mutateAsync: vi.fn(), isPending: false },
    update: { mutateAsync: vi.fn(), isPending: false },
    remove: { mutateAsync: vi.fn() },
  }),
  useNotebooks: () => ({
    notebooks: [],
    isLoading: false,
    create: { mutateAsync: vi.fn(), isPending: false },
    update: { mutateAsync: vi.fn(), isPending: false },
    remove: { mutateAsync: vi.fn() },
  }),
  useTags: () => ({
    tags: [],
    isLoading: false,
    create: { mutateAsync: vi.fn(), isPending: false },
    update: { mutateAsync: vi.fn(), isPending: false },
    remove: { mutateAsync: vi.fn(), isPending: false },
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
}));

import TodosPage from "@/app/todos/page";

describe("Todos page", () => {
  it("renders todos heading", () => {
    render(createElement(QueryClientProvider, { client: queryClient }, createElement(TodosPage)));
    expect(screen.getAllByText("Todos").length).toBeGreaterThanOrEqual(1);
  });

  it("displays todo items", () => {
    render(createElement(QueryClientProvider, { client: queryClient }, createElement(TodosPage)));
    expect(screen.getByText("Buy milk")).toBeDefined();
    expect(screen.getByText("Done task")).toBeDefined();
  });

  it("shows priority badges", () => {
    render(createElement(QueryClientProvider, { client: queryClient }, createElement(TodosPage)));
    expect(screen.getByText("high")).toBeDefined();
    expect(screen.getByText("low")).toBeDefined();
  });

  it("has note selector for todo-note linking", () => {
    render(createElement(QueryClientProvider, { client: queryClient }, createElement(TodosPage)));
    const selectors = screen.getAllByRole("combobox");
    expect(selectors.length).toBeGreaterThanOrEqual(1);
  });
});
