"use client";
import { create } from "zustand";
import { api } from "@/lib/api";
import type { User } from "@/types";

interface AuthState {
  user: User | null;
  isLoggedIn: boolean;
  isUnlocked: boolean;
  isHydrated: boolean;
  sessionMinutes: number;
  sessionElapsed: number;
  setUser: (user: User | null) => void;
  setLoggedIn: (loggedIn: boolean) => void;
  setUnlocked: (unlocked: boolean) => void;
  setHydrated: (hydrated: boolean) => void;
  setSessionMinutes: (m: number) => void;
  setSessionElapsed: (s: number) => void;
  logout: () => void;
  lock: () => Promise<void>;
  unlock: (password: string) => Promise<void>;
  setupMasterPassword: (password: string) => Promise<void>;
  bootstrap: () => Promise<void>;
}

export const useAuth = create<AuthState>((set, get) => ({
  user: null,
  isLoggedIn: false,
  isUnlocked: false,
  isHydrated: false,
  sessionMinutes: 15,
  sessionElapsed: 0,
  setUser: (user) => set({ user }),
  setLoggedIn: (isLoggedIn) => set({ isLoggedIn }),
  setUnlocked: (isUnlocked) => set({ isUnlocked }),
  setHydrated: (isHydrated) => set({ isHydrated }),
  setSessionMinutes: (sessionMinutes) => set({ sessionMinutes }),
  setSessionElapsed: (sessionElapsed) => set({ sessionElapsed }),
  logout: () => set({ user: null, isLoggedIn: false, isUnlocked: false }),
  lock: async () => {
    await api.auth.lock();
    set({ isUnlocked: false, sessionElapsed: 0 });
  },
  unlock: async (password: string) => {
    await api.auth.unlock(password);
    await api.auth.refreshSession();
    set({ isUnlocked: true, sessionElapsed: 0 });
  },
  setupMasterPassword: async (password: string) => {
    await api.encryption.setMasterPassword(password);
    await api.auth.refreshSession();
    set({ isUnlocked: true, isLoggedIn: true, sessionElapsed: 0 });
  },
  bootstrap: async () => {
    try {
      const session = await api.auth.checkSession();
      if (session.valid) {
        // Session is valid — user is logged in and unlocked
        set({
          isLoggedIn: true,
          isUnlocked: true,
          sessionElapsed: session.elapsed_seconds ?? 0,
          sessionMinutes: session.timeout_minutes ?? 15,
          isHydrated: true,
        });
        return;
      }

      // Session not valid — check if a master password has been set
      try {
        const salt = await api.encryption.getSalt();
        if (salt !== null) {
          // Master password exists but not unlocked — logged in
          set({
            isLoggedIn: true,
            isUnlocked: false,
            sessionElapsed: session.elapsed_seconds ?? 0,
            sessionMinutes: session.timeout_minutes ?? 15,
            isHydrated: true,
          });
        } else {
          // No master password — not set up yet
          set({ isLoggedIn: false, isUnlocked: false, isHydrated: true });
        }
      } catch {
        // Salt check failed — assume not set up
        set({ isLoggedIn: false, isUnlocked: false, isHydrated: true });
      }
    } catch {
      // Session check failed — backend might not be ready
      set({ isLoggedIn: false, isUnlocked: false, isHydrated: true });
    }
  },
}));
