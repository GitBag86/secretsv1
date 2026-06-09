import { describe, it, expect, vi, beforeEach } from "vitest";
import { renderHook, waitFor } from "@testing-library/react";
import { QueryClient, QueryClientProvider } from "@tanstack/react-query";
import { createElement, ReactNode } from "react";
import { useTags, useNoteTags } from "@/hooks/useTags";

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

describe("useTags hook", () => {
  it("returns empty tags initially", async () => {
    mockInvoke.mockResolvedValue([]);
    const { result } = renderHook(() => useTags(), { wrapper: createWrapper() });
    await waitFor(() => expect(result.current.isLoading).toBe(false));
    expect(result.current.tags).toEqual([]);
  });

  it("fetches tags from API", async () => {
    const mockTags = [
      { id: "t1", user_id: "u1", name: "important", color: "#ef4444", created_at: 1700000000 },
      { id: "t2", user_id: "u1", name: "work", color: "#3b82f6", created_at: 1700000000 },
    ];
    mockInvoke.mockResolvedValue(mockTags);
    const { result } = renderHook(() => useTags(), { wrapper: createWrapper() });
    await waitFor(() => expect(result.current.isLoading).toBe(false));
    expect(result.current.tags).toHaveLength(2);
    expect(result.current.tags[0].name).toBe("important");
  });

  it("create mutation calls invoke", async () => {
    mockInvoke.mockResolvedValue({ id: "new", name: "urgent", color: "#ef4444" });
    const { result } = renderHook(() => useTags(), { wrapper: createWrapper() });
    await waitFor(() => expect(result.current.isLoading).toBe(false));

    await result.current.create.mutateAsync({ name: "urgent" });
    expect(mockInvoke).toHaveBeenCalledWith("create_tag", { name: "urgent" });
  });

  it("create mutation with color passes color", async () => {
    mockInvoke.mockResolvedValue({ id: "new", name: "urgent", color: "#ff0000" });
    const { result } = renderHook(() => useTags(), { wrapper: createWrapper() });
    await waitFor(() => expect(result.current.isLoading).toBe(false));

    await result.current.create.mutateAsync({ name: "urgent", color: "#ff0000" });
    expect(mockInvoke).toHaveBeenCalledWith("create_tag", { name: "urgent", color: "#ff0000" });
  });

  it("update mutation calls invoke", async () => {
    mockInvoke.mockResolvedValue({ id: "t1", name: "renamed", color: "#6b7280" });
    const { result } = renderHook(() => useTags(), { wrapper: createWrapper() });
    await waitFor(() => expect(result.current.isLoading).toBe(false));

    await result.current.update.mutateAsync({ id: "t1", name: "renamed" });
    expect(mockInvoke).toHaveBeenCalledWith("update_tag", { id: "t1", name: "renamed" });
  });

  it("update mutation with color calls invoke", async () => {
    mockInvoke.mockResolvedValue({ id: "t1", name: "test", color: "#00ff00" });
    const { result } = renderHook(() => useTags(), { wrapper: createWrapper() });
    await waitFor(() => expect(result.current.isLoading).toBe(false));

    await result.current.update.mutateAsync({ id: "t1", color: "#00ff00" });
    expect(mockInvoke).toHaveBeenCalledWith("update_tag", { id: "t1", color: "#00ff00" });
  });

  it("remove mutation calls invoke", async () => {
    mockInvoke.mockResolvedValue(undefined);
    const { result } = renderHook(() => useTags(), { wrapper: createWrapper() });
    await waitFor(() => expect(result.current.isLoading).toBe(false));

    await result.current.remove.mutateAsync("tag-1");
    expect(mockInvoke).toHaveBeenCalledWith("delete_tag", { id: "tag-1" });
  });
});

describe("useNoteTags hook", () => {
  it("returns empty note tags when noteId is null", () => {
    const { result } = renderHook(() => useNoteTags(null), { wrapper: createWrapper() });
    expect(result.current.noteTags).toEqual([]);
    expect(result.current.isLoading).toBe(false);
  });

  it("fetches note tags when noteId is provided", async () => {
    const mockTags = [
      { id: "t1", user_id: "u1", name: "important", color: "#ef4444", created_at: 1700000000 },
    ];
    mockInvoke.mockResolvedValue(mockTags);
    const { result } = renderHook(() => useNoteTags("note-1"), { wrapper: createWrapper() });
    await waitFor(() => expect(result.current.isLoading).toBe(false));
    expect(result.current.noteTags).toHaveLength(1);
    expect(mockInvoke).toHaveBeenCalledWith("get_note_tags", { noteId: "note-1" });
  });

  it("setNoteTags mutation calls invoke", async () => {
    mockInvoke.mockResolvedValue(undefined);
    const { result } = renderHook(() => useNoteTags("note-1"), { wrapper: createWrapper() });
    await waitFor(() => expect(result.current.isLoading).toBe(false));

    await result.current.setNoteTags.mutateAsync(["t1", "t2"]);
    expect(mockInvoke).toHaveBeenCalledWith("set_note_tags", { noteId: "note-1", tagIds: ["t1", "t2"] });
  });
});
