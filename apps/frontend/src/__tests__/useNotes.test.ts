import { describe, it, expect, vi, beforeEach } from "vitest";
import { renderHook, waitFor } from "@testing-library/react";
import { QueryClient, QueryClientProvider } from "@tanstack/react-query";
import { createElement, ReactNode } from "react";
import { useNotes } from "@/hooks/useNotes";

vi.mock("@tauri-apps/api/core", () => ({
  invoke: vi.fn(),
}));

import { invoke } from "@tauri-apps/api/core";
const mockInvoke = vi.mocked(invoke);

function createWrapper() {
  const queryClient = new QueryClient({
    defaultOptions: { queries: { retry: false } },
  });
  return function Wrapper({ children }: { children: ReactNode }) {
    return createElement(QueryClientProvider, { client: queryClient }, children);
  };
}

beforeEach(() => {
  mockInvoke.mockReset();
});

describe("useNotes hook", () => {
  it("returns empty notes initially", async () => {
    mockInvoke.mockResolvedValue([]);
    const { result } = renderHook(() => useNotes(), { wrapper: createWrapper() });
    await waitFor(() => expect(result.current.isLoading).toBe(false));
    expect(result.current.notes).toEqual([]);
  });

  it("fetches notes from API", async () => {
    const mockNotes = [
      { id: "1", title: "Note 1", content: "C1", word_count: 10, reading_time: 1, is_pinned: false, is_archived: false },
      { id: "2", title: "Note 2", content: "C2", word_count: 20, reading_time: 2, is_pinned: true, is_archived: false },
    ];
    mockInvoke.mockResolvedValue(mockNotes);
    const { result } = renderHook(() => useNotes(), { wrapper: createWrapper() });
    await waitFor(() => expect(result.current.isLoading).toBe(false));
    expect(result.current.notes).toHaveLength(2);
    expect(result.current.notes[0].title).toBe("Note 1");
  });

  it("create mutation calls invoke", async () => {
    mockInvoke.mockResolvedValue({ id: "new", title: "New", content: "C" });
    const { result } = renderHook(() => useNotes(), { wrapper: createWrapper() });
    await waitFor(() => expect(result.current.isLoading).toBe(false));

    await result.current.create.mutateAsync({ title: "New", content: "C" });
    expect(mockInvoke).toHaveBeenCalledWith("create_note", { title: "New", content: "C" });
  });

  it("remove mutation calls delete", async () => {
    mockInvoke.mockResolvedValue(undefined);
    const { result } = renderHook(() => useNotes(), { wrapper: createWrapper() });
    await waitFor(() => expect(result.current.isLoading).toBe(false));

    await result.current.remove.mutateAsync("note-1");
    expect(mockInvoke).toHaveBeenCalledWith("delete_note", { id: "note-1" });
  });
});
