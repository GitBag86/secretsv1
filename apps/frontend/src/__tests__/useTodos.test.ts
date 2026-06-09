import { describe, it, expect, vi, beforeEach } from "vitest";
import { renderHook, waitFor } from "@testing-library/react";
import { QueryClient, QueryClientProvider } from "@tanstack/react-query";
import { createElement, ReactNode } from "react";
import { useTodos } from "@/hooks/useTodos";

vi.mock("@tauri-apps/api/core", () => ({
  invoke: vi.fn(),
}));

import { invoke } from "@tauri-apps/api/core";
const mockInvoke = vi.mocked(invoke);

function createWrapper() {
  const queryClient = new QueryClient({ defaultOptions: { queries: { retry: false } } });
  return function Wrapper({ children }: { children: ReactNode }) {
    return createElement(QueryClientProvider, { client: queryClient }, children);
  };
}

beforeEach(() => { mockInvoke.mockReset(); });

describe("useTodos hook", () => {
  it("returns empty todos initially", async () => {
    mockInvoke.mockResolvedValue([]);
    const { result } = renderHook(() => useTodos(), { wrapper: createWrapper() });
    await waitFor(() => expect(result.current.isLoading).toBe(false));
    expect(result.current.todos).toEqual([]);
  });

  it("fetches todos from API", async () => {
    const mockTodos = [
      { id: "1", title: "Buy milk", is_completed: false, priority: "high", created_at: 0, updated_at: 0 },
      { id: "2", title: "Walk dog", is_completed: true, priority: "low", created_at: 0, updated_at: 0 },
    ];
    mockInvoke.mockResolvedValue(mockTodos);
    const { result } = renderHook(() => useTodos(), { wrapper: createWrapper() });
    await waitFor(() => expect(result.current.isLoading).toBe(false));
    expect(result.current.todos).toHaveLength(2);
  });

  it("create mutation works", async () => {
    mockInvoke.mockResolvedValue({ id: "new", title: "New", priority: "medium" });
    const { result } = renderHook(() => useTodos(), { wrapper: createWrapper() });
    await waitFor(() => expect(result.current.isLoading).toBe(false));

    await result.current.create.mutateAsync({ title: "New", priority: "medium" });
    expect(mockInvoke).toHaveBeenCalledWith("create_todo", { title: "New", priority: "medium" });
  });

  it("update mutation works", async () => {
    mockInvoke.mockResolvedValue({ id: "1", is_completed: true });
    const { result } = renderHook(() => useTodos(), { wrapper: createWrapper() });
    await waitFor(() => expect(result.current.isLoading).toBe(false));

    await result.current.update.mutateAsync({ id: "1", is_completed: true });
    expect(mockInvoke).toHaveBeenCalledWith("update_todo", { id: "1", is_completed: true });
  });

  it("remove mutation works", async () => {
    mockInvoke.mockResolvedValue(undefined);
    const { result } = renderHook(() => useTodos(), { wrapper: createWrapper() });
    await waitFor(() => expect(result.current.isLoading).toBe(false));

    await result.current.remove.mutateAsync("t-1");
    expect(mockInvoke).toHaveBeenCalledWith("delete_todo", { id: "t-1" });
  });
});
