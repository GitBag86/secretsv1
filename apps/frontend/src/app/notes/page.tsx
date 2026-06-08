"use client";
import { useNotes, useNotebooks } from "@/hooks";
import { NotebookSidebar } from "@/components/notebook-sidebar";
import { NavHeader } from "@/components/nav-header";
import { RichTextEditor } from "@/components/rich-text-editor";
import { useState, useMemo, useEffect } from "react";
import { useQuery } from "@tanstack/react-query";
import { api } from "@/lib/api";
import type { Note } from "@/types";
import TurndownService from "turndown";

const turndown = new TurndownService({ headingStyle: "atx", codeBlockStyle: "fenced" });

function exportNote(note: Note) {
  const md = turndown.turndown(note.content || "");
  const text = `# ${note.title}\n\n${md}`;
  const blob = new Blob([text], { type: "text/markdown" });
  const url = URL.createObjectURL(blob);
  const a = document.createElement("a");
  a.href = url;
  a.download = `${note.title.replace(/[^a-zA-Z0-9_-]/g, "_")}.md`;
  a.click();
  URL.revokeObjectURL(url);
}

function stripHtml(html: string): string {
  if (!html) return "";
  return html.replace(/<[^>]*>/g, "").replace(/&nbsp;/g, " ").replace(/\s+/g, " ").trim();
}

export default function NotesPage() {
  const { notes, isLoading: notesLoading, create, update, remove } = useNotes();
  const { notebooks } = useNotebooks();
  const [title, setTitle] = useState("");
  const [content, setContent] = useState("");
  const [showForm, setShowForm] = useState(false);
  const [activeNotebook, setActiveNotebook] = useState<string | null>(null);
  const [search, setSearch] = useState("");
  const [debouncedSearch, setDebouncedSearch] = useState("");
  const [editingNote, setEditingNote] = useState<Note | null>(null);
  const [editTitle, setEditTitle] = useState("");
  const [editContent, setEditContent] = useState("");

  useEffect(() => {
    const t = setTimeout(() => setDebouncedSearch(search), 300);
    return () => clearTimeout(t);
  }, [search]);

  const { data: searchResults, isLoading: searchLoading } = useQuery({
    queryKey: ["notes", "search", debouncedSearch],
    queryFn: () => api.notes.search(debouncedSearch),
    enabled: debouncedSearch.length > 1,
  });

  const filtered = useMemo(() => {
    let result = debouncedSearch.length > 1 ? (searchResults ?? notes) : notes;
    if (activeNotebook) result = result.filter((n) => n.notebook_id === activeNotebook);
    return result;
  }, [notes, searchResults, activeNotebook, debouncedSearch]);

  useEffect(() => {
    if (editingNote) {
      setEditTitle(editingNote.title);
      setEditContent(editingNote.content);
    }
  }, [editingNote]);

  const notebookName = activeNotebook ? notebooks.find((nb) => nb.id === activeNotebook)?.name : null;
  const isLoading = notesLoading || searchLoading;

  const handleCreate = async () => {
    if (!title.trim()) return;
    await create.mutateAsync({ title, content });
    setTitle("");
    setContent("");
    setShowForm(false);
  };

  const handleSaveEdit = async () => {
    if (!editingNote || !editTitle.trim()) return;
    await update.mutateAsync({ id: editingNote.id, title: editTitle, content: editContent });
    setEditingNote(null);
  };

  return (
    <div className="min-h-screen bg-background">
      <NavHeader />
      <div className="flex" style={{ height: "calc(100vh - 3.5rem)" }}>
        <NotebookSidebar activeNotebook={activeNotebook} onSelect={setActiveNotebook} />
        <main className="flex-1 overflow-y-auto p-6">
          <div className="flex items-center justify-between mb-6">
            <div>
              <h1 className="text-3xl font-bold">{notebookName || "All Notes"}</h1>
              <p className="text-sm text-muted-foreground mt-1">{filtered.length} note{filtered.length !== 1 ? "s" : ""}</p>
            </div>
            <button onClick={() => setShowForm(true)} className="bg-primary text-primary-foreground px-4 py-2 rounded-md text-sm font-medium hover:bg-primary/90">
              New Note
            </button>
          </div>
          <input value={search} onChange={(e) => setSearch(e.target.value)} placeholder="Search notes..." className="w-full mb-6 p-2 border rounded-md bg-background" />
          {showForm && (
            <div className="rounded-lg border bg-card p-6 mb-6 shadow-sm">
              <input value={title} onChange={(e) => setTitle(e.target.value)} placeholder="Note title..." className="w-full mb-3 p-2 border rounded-md bg-background" />
              <RichTextEditor content={content} onChange={setContent} placeholder="Write your note..." />
              <div className="flex gap-2 mt-3">
                <button onClick={handleCreate} disabled={create.isPending} className="bg-primary text-primary-foreground px-4 py-2 rounded-md text-sm font-medium hover:bg-primary/90">
                  {create.isPending ? "Saving..." : "Save"}
                </button>
                <button onClick={() => { setShowForm(false); setContent(""); setTitle(""); }} className="border px-4 py-2 rounded-md text-sm font-medium hover:bg-accent">Cancel</button>
              </div>
            </div>
          )}
          {isLoading ? (
            <p className="text-muted-foreground">Loading notes...</p>
          ) : filtered.length === 0 ? (
            <p className="text-muted-foreground">{debouncedSearch ? "No notes match your search." : activeNotebook ? "No notes in this notebook." : "No notes yet. Create your first note!"}</p>
          ) : (
            <div className="grid gap-4 md:grid-cols-2 xl:grid-cols-3">
              {filtered.map((note) => (
                <div
                  key={note.id}
                  onClick={() => { setEditingNote(note); }}
                  className="rounded-lg border bg-card p-6 shadow-sm hover:shadow-md transition-shadow cursor-pointer"
                >
                  <h3 className="font-semibold text-lg truncate">{note.title}</h3>
                  <p className="text-muted-foreground text-sm mt-2 line-clamp-3">{stripHtml(note.content) || <span className="italic">Empty note</span>}</p>
                  <div className="flex items-center justify-between mt-4">
                    <span className="text-xs text-muted-foreground">{note.word_count} words · {note.reading_time} min read</span>
                    <div className="flex gap-2">
                      <button onClick={(e) => { e.stopPropagation(); exportNote(note); }} className="text-xs text-muted-foreground hover:text-foreground" title="Export Markdown">↓</button>
                      <button onClick={(e) => { e.stopPropagation(); remove.mutate(note.id); }} className="text-destructive text-sm hover:underline">Delete</button>
                    </div>
                  </div>
                </div>
              ))}
            </div>
          )}
        </main>
      </div>
      {editingNote && (
        <div className="fixed inset-0 z-50 flex items-center justify-center bg-black/50" onClick={() => setEditingNote(null)}>
          <div className="bg-card rounded-lg border shadow-xl w-full max-w-2xl max-h-[90vh] overflow-y-auto p-6 m-4" onClick={(e) => e.stopPropagation()}>
            <h2 className="text-xl font-bold mb-4">Edit Note</h2>
            <div className="flex items-center gap-2 mb-4">
              <button onClick={() => exportNote(editingNote)} className="text-xs border px-2 py-1 rounded hover:bg-accent">Export Markdown</button>
            </div>
            <input value={editTitle} onChange={(e) => setEditTitle(e.target.value)} placeholder="Note title..." className="w-full mb-3 p-2 border rounded-md bg-background" />
            <RichTextEditor content={editContent} onChange={setEditContent} placeholder="Write your note..." />
            <div className="flex gap-2 mt-3">
              <button onClick={handleSaveEdit} disabled={update.isPending} className="bg-primary text-primary-foreground px-4 py-2 rounded-md text-sm font-medium hover:bg-primary/90">
                {update.isPending ? "Saving..." : "Save"}
              </button>
              <button onClick={() => setEditingNote(null)} className="border px-4 py-2 rounded-md text-sm font-medium hover:bg-accent">Cancel</button>
            </div>
          </div>
        </div>
      )}
    </div>
  );
}
