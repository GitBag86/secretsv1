import { describe, it, expect, vi, beforeEach } from "vitest";
import { renderHook, waitFor } from "@testing-library/react";
import { QueryClient, QueryClientProvider } from "@tanstack/react-query";
import { createElement, ReactNode } from "react";
import { useCalendar } from "@/hooks/useCalendar";

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

describe("useCalendar hook", () => {
  it("returns empty events initially", async () => {
    mockInvoke.mockResolvedValue([]);
    const { result } = renderHook(() => useCalendar(), { wrapper: createWrapper() });
    await waitFor(() => expect(result.current.isLoading).toBe(false));
    expect(result.current.events).toEqual([]);
  });

  it("fetches events from API", async () => {
    const mockEvents = [
      { id: "e1", title: "Standup", start_time: 1700000000, end_time: 1700003600, all_day: false, color: "#3b82f6" },
    ];
    mockInvoke.mockResolvedValue(mockEvents);
    const { result } = renderHook(() => useCalendar(), { wrapper: createWrapper() });
    await waitFor(() => expect(result.current.isLoading).toBe(false));
    expect(result.current.events).toHaveLength(1);
    expect(result.current.events[0].title).toBe("Standup");
  });

  it("create mutation calls correct invoke", async () => {
    mockInvoke.mockResolvedValue({ id: "new" });
    const { result } = renderHook(() => useCalendar(), { wrapper: createWrapper() });
    await waitFor(() => expect(result.current.isLoading).toBe(false));

    await result.current.create.mutateAsync({ title: "Lunch", start_time: 100, end_time: 200 });
    expect(mockInvoke).toHaveBeenCalledWith("create_calendar_event", { title: "Lunch", start_time: 100, end_time: 200 });
  });

  it("remove mutation works", async () => {
    mockInvoke.mockResolvedValue(undefined);
    const { result } = renderHook(() => useCalendar(), { wrapper: createWrapper() });
    await waitFor(() => expect(result.current.isLoading).toBe(false));

    await result.current.remove.mutateAsync("e-1");
    expect(mockInvoke).toHaveBeenCalledWith("delete_calendar_event", { id: "e-1" });
  });
});
