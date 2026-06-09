import { describe, it, expect, beforeEach } from "vitest";
import { renderHook, act } from "@testing-library/react";
import { useAuth } from "@/hooks/useAuth";

beforeEach(() => {
  useAuth.setState({ user: null, isUnlocked: false, isHydrated: false });
});

describe("useAuth store", () => {
  it("has initial state", () => {
    const { result } = renderHook(() => useAuth());
    expect(result.current.user).toBeNull();
    expect(result.current.isUnlocked).toBe(false);
    expect(result.current.isHydrated).toBe(false);
  });

  it("setUser stores user", () => {
    const { result } = renderHook(() => useAuth());
    const user = { id: "1", email: "a@b.com", created_at: 0, updated_at: 0 };
    act(() => result.current.setUser(user));
    expect(result.current.user).toEqual(user);
  });

  it("setUnlocked toggles", () => {
    const { result } = renderHook(() => useAuth());
    act(() => result.current.setUnlocked(true));
    expect(result.current.isUnlocked).toBe(true);
    act(() => result.current.setUnlocked(false));
    expect(result.current.isUnlocked).toBe(false);
  });

  it("setHydrated toggles", () => {
    const { result } = renderHook(() => useAuth());
    act(() => result.current.setHydrated(true));
    expect(result.current.isHydrated).toBe(true);
  });

  it("logout clears user and unlock state", () => {
    const { result } = renderHook(() => useAuth());
    act(() => {
      result.current.setUser({ id: "1", email: "a@b.com", created_at: 0, updated_at: 0 });
      result.current.setUnlocked(true);
    });
    expect(result.current.user).not.toBeNull();
    expect(result.current.isUnlocked).toBe(true);

    act(() => result.current.logout());
    expect(result.current.user).toBeNull();
    expect(result.current.isUnlocked).toBe(false);
  });

  it("setUser with null clears user", () => {
    const { result } = renderHook(() => useAuth());
    act(() => result.current.setUser({ id: "1", email: "a@b.com", created_at: 0, updated_at: 0 }));
    act(() => result.current.setUser(null));
    expect(result.current.user).toBeNull();
  });
});
