"use client";
import { useNotebooks, useNotes } from "@/hooks";
import { useState, useRef } from "react";

const defaultColors = ["#3b82f6", "#ef4444", "#22c55e", "#f59e0b", "#a855f7", "#ec4899"];

export function NotebookSidebar({ activeNotebook, onSelect }: { activeNotebook: string | null; onSelect: (id: string | null) => void }) {
  const { notebooks, isLoading, create, update, remove } = useNotebooks();
  const [showForm, setShowForm] = useState(false);
  const [name, setName] = useState("");
  const [color, setColor] = useState(defaultColors[0]);
  const [sidebarOpen, setSidebarOpen] = useState(false);
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
  };
  const handleDrop = (idx: number) => {
    if (dragIdx.current === null || dragIdx.current === idx) {
      dragIdx.current = null;
      return;
    }
    const items = [...notebooks];
    const [moved] = items.splice(dragIdx.current, 1);
    items.splice(idx, 0, moved);
    dragIdx.current = null;
    // Save the new order: fire all mutations at once
    items.forEach((nb, i) => {
      if (nb.sort_order !== i) update.mutate({ id: nb.id, sort_order: i });
    });
  };

  const sidebarContent = (
    <>
      <div className="p-3 border-b">
        <button onClick={() => { onSelect(null); setSidebarOpen(false); }} className={`w-full text-left px-3 py-2 rounded-md text-sm font-medium ${activeNotebook === null ? "bg-primary text-primary-foreground" : "hover:bg-accent"}`}>
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
            onDrop={() => handleDrop(idx)}
            className="flex items-center gap-1 group"
          >
            <button onClick={() => { onSelect(nb.id); setSidebarOpen(false); }} className={`flex-1 text-left px-3 py-1.5 rounded-md text-sm truncate ${activeNotebook === nb.id ? "bg-primary/10 text-primary font-medium" : "hover:bg-accent"}`}>
              <span className="inline-block w-2 h-2 rounded-full mr-2" style={{ backgroundColor: nb.color }} />
              {nb.name}
            </button>
            <button onClick={() => remove.mutate(nb.id)} className="opacity-0 group-hover:opacity-100 text-destructive text-xs hover:underline px-1">x</button>
          </div>
        ))}
      </div>
    </>
  );

  return (
    <>
      {/* Desktop sidebar */}
      <aside className="hidden md:flex w-56 border-r bg-card shrink-0 flex-col">
        {sidebarContent}
      </aside>
      {/* Mobile sidebar toggle button */}
      <button
        onClick={() => setSidebarOpen(true)}
        className="md:hidden fixed bottom-4 left-4 z-40 bg-primary text-primary-foreground p-3 rounded-full shadow-lg hover:bg-primary/90"
        aria-label="Open notebooks"
      >
        <svg className="w-5 h-5" fill="none" stroke="currentColor" viewBox="0 0 24 24">
          <path strokeLinecap="round" strokeLinejoin="round" strokeWidth={2} d="M4 6h16M4 12h16M4 18h16" />
        </svg>
      </button>
      {/* Mobile sidebar overlay */}
      {sidebarOpen && (
        <div className="md:hidden fixed inset-0 z-50">
          <div className="absolute inset-0 bg-black/50" onClick={() => setSidebarOpen(false)} />
          <aside className="absolute left-0 top-0 bottom-0 w-64 bg-card border-r shadow-xl flex flex-col">
            <div className="p-3 border-b flex items-center justify-between">
              <span className="text-xs font-semibold text-muted-foreground uppercase">Notebooks</span>
              <button onClick={() => setSidebarOpen(false)} className="p-1 rounded-md hover:bg-accent" aria-label="Close">
                <svg className="w-4 h-4" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                  <path strokeLinecap="round" strokeLinejoin="round" strokeWidth={2} d="M6 18L18 6M6 6l12 12" />
                </svg>
              </button>
            </div>
            <div className="flex-1 overflow-y-auto">
              {sidebarContent}
            </div>
          </aside>
        </div>
      )}
    </>
  );
}
