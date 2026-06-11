"use client";
import { AuthGuard } from "@/components/auth-guard";
import { NavHeader } from "@/components/nav-header";
import { useQuery, useMutation, useQueryClient } from "@tanstack/react-query";
import { api } from "@/lib/api";
import { useState } from "react";

export default function TrashPage() {
  const queryClient = useQueryClient();
  const [tab, setTab] = useState<"notes" | "todos">("notes");

  const { data: archivedNotes = [], isLoading: notesLoading } = useQuery({
    queryKey: ["archived-notes"],
    queryFn: () => api.trash.listNotes(),
  });

  const { data: archivedTodos = [], isLoading: todosLoading } = useQuery({
    queryKey: ["archived-todos"],
    queryFn: () => api.trash.listTodos(),
  });

  const invalidate = () => {
    queryClient.invalidateQueries({ queryKey: ["archived-notes"] });
    queryClient.invalidateQueries({ queryKey: ["archived-todos"] });
    queryClient.invalidateQueries({ queryKey: ["notes"] });
    queryClient.invalidateQueries({ queryKey: ["todos"] });
  };

  const restoreNote = useMutation({
    mutationFn: api.trash.restoreNote,
    onSuccess: invalidate,
  });
  const deleteNote = useMutation({
    mutationFn: api.trash.permanentlyDeleteNote,
    onSuccess: invalidate,
  });
  const restoreTodo = useMutation({
    mutationFn: api.trash.restoreTodo,
    onSuccess: invalidate,
  });
  const deleteTodo = useMutation({
    mutationFn: api.trash.permanentlyDeleteTodo,
    onSuccess: invalidate,
  });

  const isLoading = notesLoading || todosLoading;

  return (
    <AuthGuard>
    <div className="min-h-screen bg-background">
      <NavHeader />
      <main className="container py-6 max-w-3xl mx-auto">
        <div className="flex items-center justify-between mb-6">
          <div>
            <h1 className="text-3xl font-bold">Trash</h1>
            <p className="text-sm text-muted-foreground mt-1">
              {archivedNotes.length + archivedTodos.length} archived item{(archivedNotes.length + archivedTodos.length) !== 1 ? "s" : ""}
            </p>
          </div>
        </div>

        {/* Tabs */}
        <div className="flex gap-1 mb-6 border-b">
          <button
            onClick={() => setTab("notes")}
            className={`px-4 py-2 text-sm font-medium border-b-2 transition-colors ${
              tab === "notes" ? "border-primary text-primary" : "border-transparent text-muted-foreground hover:text-foreground"
            }`}
          >
            Notes ({archivedNotes.length})
          </button>
          <button
            onClick={() => setTab("todos")}
            className={`px-4 py-2 text-sm font-medium border-b-2 transition-colors ${
              tab === "todos" ? "border-primary text-primary" : "border-transparent text-muted-foreground hover:text-foreground"
            }`}
          >
            Todos ({archivedTodos.length})
          </button>
        </div>

        {isLoading ? (
          <p className="text-muted-foreground">Loading...</p>
        ) : tab === "notes" ? (
          archivedNotes.length === 0 ? (
            <div className="text-center py-12">
              <p className="text-lg text-muted-foreground">Trash is empty</p>
              <p className="text-sm text-muted-foreground mt-1">Archived notes will appear here.</p>
            </div>
          ) : (
            <div className="space-y-2">
              {archivedNotes.map((note) => (
                <div key={note.id} className="flex items-center gap-3 rounded-lg border bg-card p-4">
                  <div className="flex-1 min-w-0">
                    <p className="text-sm font-medium truncate">{note.title || "Untitled"}</p>
                    <p className="text-xs text-muted-foreground mt-0.5">
                      {note.word_count} words · Archived {new Date(note.updated_at * 1000).toLocaleDateString()}
                    </p>
                  </div>
                  <button
                    onClick={() => restoreNote.mutate(note.id)}
                    disabled={restoreNote.isPending}
                    className="text-xs px-3 py-1.5 rounded bg-primary text-primary-foreground hover:bg-primary/90 disabled:opacity-50"
                  >
                    Restore
                  </button>
                  <button
                    onClick={() => {
                      if (confirm(`Permanently delete "${note.title || "Untitled"}"?`)) {
                        deleteNote.mutate(note.id);
                      }
                    }}
                    disabled={deleteNote.isPending}
                    className="text-xs px-3 py-1.5 rounded border text-destructive hover:bg-destructive/10 disabled:opacity-50"
                  >
                    Delete forever
                  </button>
                </div>
              ))}
            </div>
          )
        ) : (
          archivedTodos.length === 0 ? (
            <div className="text-center py-12">
              <p className="text-lg text-muted-foreground">Trash is empty</p>
              <p className="text-sm text-muted-foreground mt-1">Archived todos will appear here.</p>
            </div>
          ) : (
            <div className="space-y-2">
              {archivedTodos.map((todo) => (
                <div key={todo.id} className="flex items-center gap-3 rounded-lg border bg-card p-4">
                  <div className="flex-1 min-w-0">
                    <p className="text-sm font-medium truncate">{todo.title}</p>
                    <p className="text-xs text-muted-foreground mt-0.5">
                      {todo.is_completed ? "Completed" : `${todo.priority} priority`} · Archived {new Date(todo.updated_at * 1000).toLocaleDateString()}
                    </p>
                  </div>
                  <button
                    onClick={() => restoreTodo.mutate(todo.id)}
                    disabled={restoreTodo.isPending}
                    className="text-xs px-3 py-1.5 rounded bg-primary text-primary-foreground hover:bg-primary/90 disabled:opacity-50"
                  >
                    Restore
                  </button>
                  <button
                    onClick={() => {
                      if (confirm(`Permanently delete "${todo.title}"?`)) {
                        deleteTodo.mutate(todo.id);
                      }
                    }}
                    disabled={deleteTodo.isPending}
                    className="text-xs px-3 py-1.5 rounded border text-destructive hover:bg-destructive/10 disabled:opacity-50"
                  >
                    Delete forever
                  </button>
                </div>
              ))}
            </div>
          )
        )}
      </main>
    </div>
    </AuthGuard>
  );
}
