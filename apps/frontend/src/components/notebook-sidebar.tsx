"use client";
import { useNotebooks, useNotes } from "@/hooks";
import { useState, useRef } from "react";

const defaultColors = ["#3b82f6", "#ef4444", "#22c55e", "#f59e0b", "#a855f7", "#ec4899"];

export function NotebookSidebar({ activeNotebook, onSelect }: { activeNotebook: string | null; onSelect: (id: string | null) => void }) {
  const { notebooks, isLoading, create, update, remove } = useNotebooks();
  const [showForm, setShowForm] = useState(false);
  const [name, setName] = useState("");
  const [color, setColor] = useState(defaultColors[0]);
  const dragIdx = useRef<number | null>(null);

  const handleCreate = async () => {
    if (!name.trim()) return;
    await create.mutateAsync({ name: name.trim(), color });
    setName("");
    setShowForm(false);
  };

  const handleDragStart = (idx: number) => { dragIdx.current = idx; };
  const handleDragOver = (e: React.DragEvent, idx: number) => {
    e.preventDefault();
    if (dragIdx.current === null || dragIdx.current === idx) return;
    const items = [...notebooks];
    const [moved] = items.splice(dragIdx.current, 1);
    items.splice(idx, 0, moved);
    dragIdx.current = idx;
    items.forEach((nb, i) => {
      if (nb.sort_order !== i) update.mutate({ id: nb.id, sort_order: i });
    });
  };
  const handleDrop = () => { dragIdx.current = null; };

  return (
    <aside className="w-56 border-r bg-card shrink-0 flex flex-col">
      <div className="p-3 border-b">
        <button onClick={() => onSelect(null)} className={`w-full text-left px-3 py-2 rounded-md text-sm font-medium ${activeNotebook === null ? "bg-primary text-primary-foreground" : "hover:bg-accent"}`}>
          All Notes
        </button>
      </div>
      <div className="p-3 border-b flex items-center justify-between">
        <span className="text-xs font-semibold text-muted-foreground uppercase">Notebooks</span>
        <button onClick={() => setShowForm(true)} className="text-xs text-primary hover:underline">+ New</button>
      </div>
      {showForm && (
        <div className="px-3 pb-3 space-y-2">
          <input value={name} onChange={(e) => setName(e.target.value)} placeholder="Notebook name..." className="w-full p-1.5 text-xs border rounded-md bg-background" autoFocus onKeyDown={(e) => e.key === "Enter" && handleCreate()} />
          <div className="flex gap-1">
            {defaultColors.map((c) => (
              <button key={c} onClick={() => setColor(c)} className={`w-4 h-4 rounded-full ${color === c ? "ring-2 ring-offset-1 ring-primary" : ""}`} style={{ backgroundColor: c }} />
            ))}
          </div>
          <div className="flex gap-1">
            <button onClick={handleCreate} disabled={create.isPending} className="text-xs bg-primary text-primary-foreground px-2 py-1 rounded">Create</button>
            <button onClick={() => setShowForm(false)} className="text-xs hover:bg-accent px-2 py-1 rounded">Cancel</button>
          </div>
        </div>
      )}
      <div className="flex-1 overflow-y-auto p-2 space-y-1">
        {isLoading ? (
          <p className="text-xs text-muted-foreground px-3">Loading...</p>
        ) : notebooks.length === 0 ? (
          <p className="text-xs text-muted-foreground px-3">No notebooks yet</p>
        ) : notebooks.map((nb, idx) => (
          <div
            key={nb.id}
            draggable
            onDragStart={() => handleDragStart(idx)}
            onDragOver={(e) => handleDragOver(e, idx)}
            onDrop={handleDrop}
            className="flex items-center gap-1 group"
          >
            <button onClick={() => onSelect(nb.id)} className={`flex-1 text-left px-3 py-1.5 rounded-md text-sm truncate ${activeNotebook === nb.id ? "bg-primary/10 text-primary font-medium" : "hover:bg-accent"}`}>
              <span className="inline-block w-2 h-2 rounded-full mr-2" style={{ backgroundColor: nb.color }} />
              {nb.name}
            </button>
            <button onClick={() => remove.mutate(nb.id)} className="opacity-0 group-hover:opacity-100 text-destructive text-xs hover:underline px-1">x</button>
          </div>
        ))}
      </div>
    </aside>
  );
}
