"use client";
import { create } from "zustand";
import { api } from "@/lib/api";
import type { User } from "@/types";

interface AuthState {
  user: User | null;
  isUnlocked: boolean;
  isHydrated: boolean;
  sessionMinutes: number;
  sessionElapsed: number;
  setUser: (user: User | null) => void;
  setUnlocked: (unlocked: boolean) => void;
  setHydrated: (hydrated: boolean) => void;
  setSessionMinutes: (m: number) => void;
  setSessionElapsed: (s: number) => void;
  logout: () => void;
  lock: () => Promise<void>;
  unlock: (password: string) => Promise<void>;
  setupMasterPassword: (password: string) => Promise<void>;
}

export const useAuth = create<AuthState>((set) => ({
  user: null,
  isUnlocked: false,
  isHydrated: false,
  sessionMinutes: 15,
  sessionElapsed: 0,
  setUser: (user) => set({ user }),
  setUnlocked: (isUnlocked) => set({ isUnlocked }),
  setHydrated: (isHydrated) => set({ isHydrated }),
  setSessionMinutes: (sessionMinutes) => set({ sessionMinutes }),
  setSessionElapsed: (sessionElapsed) => set({ sessionElapsed }),
  logout: () => set({ user: null, isUnlocked: false }),
  lock: async () => {
    await api.auth.lock();
    set({ isUnlocked: false });
  },
  unlock: async (password: string) => {
    await api.auth.unlock(password);
    await api.auth.refreshSession();
    set({ isUnlocked: true, sessionElapsed: 0 });
  },
  setupMasterPassword: async (password: string) => {
    await api.encryption.setMasterPassword(password);
    await api.auth.refreshSession();
    set({ isUnlocked: true, sessionElapsed: 0 });
  },
}));
