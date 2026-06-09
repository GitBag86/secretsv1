"use client";
import { useNotes, useNotebooks, useTags, useNoteTags } from "@/hooks";
import { NotebookSidebar } from "@/components/notebook-sidebar";
import { NavHeader } from "@/components/nav-header";
import { TemplatePicker } from "@/components/template-picker";
import { RichTextEditor } from "@/components/rich-text-editor";
import { useState, useMemo, useEffect, useRef } from "react";
import { useQuery, useMutation, useQueryClient } from "@tanstack/react-query";
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
  const { tags: allTags, create: createTag } = useTags();
  const [title, setTitle] = useState("");
  const [content, setContent] = useState("");
  const [showForm, setShowForm] = useState(false);
  const [activeNotebook, setActiveNotebook] = useState<string | null>(null);
  const [search, setSearch] = useState("");
  const [debouncedSearch, setDebouncedSearch] = useState("");
  const [editingNote, setEditingNote] = useState<Note | null>(null);
  const [editTitle, setEditTitle] = useState("");
  const [editContent, setEditContent] = useState("");
  const [editTagIds, setEditTagIds] = useState<string[]>([]);
  const [newTagName, setNewTagName] = useState("");

  const { noteTags, setNoteTags } = useNoteTags(editingNote?.id ?? null);
  const queryClient = useQueryClient();
  const fileInputRef = useRef<HTMLInputElement>(null);
  const [attaching, setAttaching] = useState(false);

  const { data: noteAttachments = [] } = useQuery({
    queryKey: ["note-attachments", editingNote?.id],
    queryFn: () => api.attachments.list(editingNote!.id),
    enabled: !!editingNote,
  });

  const invalidateAttachmentQueries = () => {
    queryClient.invalidateQueries({ queryKey: ["note-attachments"] });
    queryClient.invalidateQueries({ queryKey: ["attachment-counts"] });
  };

  const deleteAttachment = useMutation({
    mutationFn: api.attachments.delete,
    onSuccess: invalidateAttachmentQueries,
  });

  const handleAttachFile = async (e: React.ChangeEvent<HTMLInputElement>) => {
    const file = e.target.files?.[0];
    if (!file || !editingNote) return;
    setAttaching(true);
    try {
      // Use arrayBuffer + btoa for reliable base64 encoding
      const arrayBuf = await file.arrayBuffer();
      const bytes = new Uint8Array(arrayBuf);
      let binary = "";
      for (let i = 0; i < bytes.length; i++) binary += String.fromCharCode(bytes[i]);
      const base64 = btoa(binary);
      await api.attachments.attach(editingNote.id, file.name, file.type, base64);
      invalidateAttachmentQueries();
    } catch (err) {
      console.error("Failed to attach file:", err);
    } finally {
      setAttaching(false);
      if (fileInputRef.current) fileInputRef.current.value = "";
    }
  };

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

  // Sync noteTags from API into editTagIds when they load
  useEffect(() => {
    setEditTagIds(noteTags.map((t) => t.id));
  }, [noteTags]);

  const tagMap = useMemo(() => {
    const map = new Map<string, { name: string; color: string }>();
    for (const t of allTags) map.set(t.id, { name: t.name, color: t.color });
    return map;
  }, [allTags]);

  // Fetch all note-tag associations for card display
  const { data: allNoteTags = [] } = useQuery({
    queryKey: ["all-note-tags"],
    queryFn: () => api.tags.listAllNoteTags(),
  });

  const noteTagMap = useMemo(() => {
    const map = new Map<string, string[]>();
    for (const nt of allNoteTags) {
      const existing = map.get(nt.note_id) || [];
      existing.push(nt.tag_id);
      map.set(nt.note_id, existing);
    }
    return map;
  }, [allNoteTags]);

  const getNoteTags = (noteId: string) => {
    const tagIds = noteTagMap.get(noteId) || [];
    return tagIds.map((id) => tagMap.get(id)).filter(Boolean) as { name: string; color: string }[];
  };

  // Fetch attachment counts for card badges
  const { data: attachmentCounts = [] } = useQuery({
    queryKey: ["attachment-counts"],
    queryFn: () => api.attachments.getAllCounts(),
  });

  const attachmentCountMap = useMemo(() => {
    const map = new Map<string, number>();
    for (const ac of attachmentCounts) map.set(ac.note_id, ac.count);
    return map;
  }, [attachmentCounts]);

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
    // Save tag associations
    await setNoteTags.mutateAsync(editTagIds);
    setEditingNote(null);
  };

  const toggleEditTag = (tagId: string) => {
    setEditTagIds((prev) =>
      prev.includes(tagId) ? prev.filter((id) => id !== tagId) : [...prev, tagId]
    );
  };

  const handleCreateTag = async () => {
    if (!newTagName.trim()) return;
    const created = await createTag.mutateAsync({ name: newTagName.trim() });
    setEditTagIds((prev) => [...prev, created.id]);
    setNewTagName("");
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
            <div className="flex gap-2">
              <TemplatePicker onNoteCreated={(note) => setEditingNote(note)} />
              <button onClick={() => setShowForm(true)} className="bg-primary text-primary-foreground px-4 py-2 rounded-md text-sm font-medium hover:bg-primary/90">
                New Note
              </button>
            </div>
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
                  <div className="flex flex-wrap gap-1 mt-2">
                    {getNoteTags(note.id).map((tag) => (
                      <span key={tag.name} className="inline-flex items-center px-1.5 py-0.5 rounded text-[10px] font-medium" style={{ backgroundColor: tag.color + "20", color: tag.color }}>
                        {tag.name}
                      </span>
                    ))}
                  </div>
                  <div className="flex items-center justify-between mt-4">
                    <span className="text-xs text-muted-foreground">
                      {note.word_count} words · {note.reading_time} min read
                      {(attachmentCountMap.get(note.id) ?? 0) > 0 && (
                        <span className="ml-1.5" title={`${attachmentCountMap.get(note.id)} attachment(s)`}>
                          📎
                        </span>
                      )}
                    </span>
                    <div className="flex gap-2">
                      <button onClick={(e) => { e.stopPropagation(); exportNote(note); }} className="text-xs text-muted-foreground hover:text-foreground" title="Export Markdown">↓</button>
                      <button onClick={async (e) => { e.stopPropagation(); await api.trash.archiveNote(note.id); queryClient.invalidateQueries({ queryKey: ["notes"] }); }} className="text-xs text-muted-foreground hover:text-foreground" title="Archive">🗑️</button>
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

            {/* Tags */}
            <div className="mt-4 border rounded-md p-3 bg-muted/20">
              <label className="text-xs font-semibold text-muted-foreground uppercase block mb-2">Tags</label>
              <div className="flex flex-wrap gap-1.5 mb-2">
                {allTags.map((tag) => (
                  <button
                    key={tag.id}
                    type="button"
                    onClick={() => toggleEditTag(tag.id)}
                    className={`text-xs px-2 py-0.5 rounded-full border transition-all ${
                      editTagIds.includes(tag.id)
                        ? "border-transparent text-white font-medium"
                        : "hover:bg-accent"
                    }`}
                    style={{
                      backgroundColor: editTagIds.includes(tag.id) ? tag.color : "transparent",
                      borderColor: tag.color,
                      color: editTagIds.includes(tag.id) ? "#fff" : tag.color,
                    }}
                  >
                    {tag.name}
                  </button>
                ))}
              </div>
              <div className="flex gap-1.5">
                <input
                  value={newTagName}
                  onChange={(e) => setNewTagName(e.target.value)}
                  onKeyDown={(e) => e.key === "Enter" && handleCreateTag()}
                  placeholder="New tag name..."
                  className="flex-1 p-1.5 text-xs border rounded-md bg-background"
                />
                <button
                  type="button"
                  onClick={handleCreateTag}
                  disabled={!newTagName.trim() || createTag.isPending}
                  className="text-xs px-2 py-1 rounded bg-primary text-primary-foreground hover:bg-primary/90 disabled:opacity-50"
                >
                  + Add
                </button>
              </div>
            </div>

            {/* Attachments */}
            <div className="mt-4 border rounded-md p-3 bg-muted/20">
              <label className="text-xs font-semibold text-muted-foreground uppercase block mb-2">Attachments</label>
              {noteAttachments.length > 0 && (
                <div className="space-y-1.5 mb-2">
                  {noteAttachments.map((att) => (
                    <div key={att.id} className="flex items-center gap-2 text-xs">
                      <span
                        className="flex-1 truncate text-primary hover:underline cursor-pointer"
                        onClick={() => api.attachments.open(att.id)}
                        title="Open file"
                      >
                        📎 {att.filename}
                      </span>
                      <span className="text-muted-foreground shrink-0">
                        {att.size > 1024 * 1024
                          ? `${(att.size / 1024 / 1024).toFixed(1)} MB`
                          : `${(att.size / 1024).toFixed(0)} KB`}
                      </span>
                      <button
                        onClick={() => deleteAttachment.mutate(att.id)}
                        className="text-destructive hover:underline shrink-0"
                      >
                        Delete
                      </button>
                    </div>
                  ))}
                </div>
              )}
              <div className="flex gap-1.5 items-center">
                <input
                  ref={fileInputRef}
                  type="file"
                  onChange={handleAttachFile}
                  className="hidden"
                />
                <button
                  type="button"
                  onClick={() => fileInputRef.current?.click()}
                  disabled={attaching}
                  className="text-xs px-2 py-1 rounded bg-primary text-primary-foreground hover:bg-primary/90 disabled:opacity-50"
                >
                  {attaching ? "Uploading..." : "Attach File"}
                </button>
              </div>
            </div>

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
