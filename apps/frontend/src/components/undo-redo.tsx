"use client";
import { useEffect, useState } from "react";
import { useUndoStack } from "@/hooks/useUndoStack";
import { api } from "@/lib/api";

export function UndoRedo() {
  const { undoStack, redoStack, popUndo, popRedo } = useUndoStack();
  const [msg, setMsg] = useState<string | null>(null);

  useEffect(() => {
    const handler = async (e: KeyboardEvent) => {
      if ((e.ctrlKey || e.metaKey) && e.key === "z") {
        if (e.shiftKey) {
          await doRedo();
        } else {
          await doUndo();
        }
      }
    };
    window.addEventListener("keydown", handler);
    return () => window.removeEventListener("keydown", handler);
  }, [undoStack, redoStack]);

  const doUndo = async () => {
    const entry = popUndo();
    if (!entry) return;
    try {
      if (entry.action === "create") {
        if (entry.type === "todo") await api.todos.delete(entry.id);
        else if (entry.type === "note") await api.notes.delete(entry.id);
      } else if (entry.action === "update") {
        if (entry.type === "todo") await api.todos.update(entry.id, entry.before!);
        else if (entry.type === "note") await api.notes.update(entry.id, entry.before!);
      } else if (entry.action === "delete") {
        if (entry.type === "todo") await api.todos.create(entry.before!);
        else if (entry.type === "note") await api.notes.create(entry.before!);
      }
      setMsg(`Undo: ${entry.label}`);
      setTimeout(() => setMsg(null), 2000);
    } catch { /* ignore */ }
  };

  const doRedo = async () => {
    const entry = popRedo();
    if (!entry) return;
    try {
      if (entry.action === "create") {
        if (entry.type === "todo") await api.todos.create(entry.after!);
        else if (entry.type === "note") await api.notes.create(entry.after!);
      } else if (entry.action === "update") {
        if (entry.type === "todo") await api.todos.update(entry.id, entry.after!);
        else if (entry.type === "note") await api.notes.update(entry.id, entry.after!);
      } else if (entry.action === "delete") {
        if (entry.type === "todo") await api.todos.delete(entry.id);
        else if (entry.type === "note") await api.notes.delete(entry.id);
      }
      setMsg(`Redo: ${entry.label}`);
      setTimeout(() => setMsg(null), 2000);
    } catch { /* ignore */ }
  };

  return (
    <>
      {msg && (
        <div className="fixed bottom-4 right-4 z-50 bg-card border shadow-lg rounded-md px-4 py-2 text-sm animate-in fade-in slide-in-from-bottom-2">
          {msg}
        </div>
      )}
      <div className="fixed bottom-4 left-4 z-50 flex gap-1">
        <button onClick={doUndo} disabled={undoStack.length === 0} title="Undo (Ctrl+Z)" className="bg-card border shadow-sm rounded-md px-2 py-1 text-xs hover:bg-accent disabled:opacity-30">
          ↩
        </button>
        <button onClick={doRedo} disabled={redoStack.length === 0} title="Redo (Ctrl+Shift+Z)" className="bg-card border shadow-sm rounded-md px-2 py-1 text-xs hover:bg-accent disabled:opacity-30">
          ↪
        </button>
      </div>
    </>
  );
}
