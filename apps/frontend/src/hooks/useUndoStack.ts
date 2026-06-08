"use client";
import { create } from "zustand";

interface UndoEntry {
  id: string;
  type: "todo" | "note" | "event";
  action: "create" | "update" | "delete";
  before: Record<string, any> | null;
  after: Record<string, any> | null;
  label: string;
}

interface UndoState {
  undoStack: UndoEntry[];
  redoStack: UndoEntry[];
  push: (entry: UndoEntry) => void;
  popUndo: () => UndoEntry | undefined;
  popRedo: () => UndoEntry | undefined;
  clear: () => void;
}

export const useUndoStack = create<UndoState>((set, get) => ({
  undoStack: [],
  redoStack: [],
  push: (entry) => set((s) => ({ undoStack: [...s.undoStack.slice(-49), entry], redoStack: [] })),
  popUndo: () => {
    const entry = get().undoStack.at(-1);
    if (entry) set((s) => ({ undoStack: s.undoStack.slice(0, -1), redoStack: [...s.redoStack, entry] }));
    return entry;
  },
  popRedo: () => {
    const entry = get().redoStack.at(-1);
    if (entry) set((s) => ({ redoStack: s.redoStack.slice(0, -1), undoStack: [...s.undoStack, entry] }));
    return entry;
  },
  clear: () => set({ undoStack: [], redoStack: [] }),
}));
