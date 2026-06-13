"use client";
import { AuthGuard } from "@/components/auth-guard";
import { useTodos, useTags } from "@/hooks";
import { NavHeader } from "@/components/nav-header";
import { useState, useMemo, useEffect, useRef } from "react";
import { useQuery, useMutation, useQueryClient } from "@tanstack/react-query";
import { api } from "@/lib/api";
import type { RecurringTodo, Tag } from "@/types";

const RECURRENCE_LABELS: Record<string, string> = {
  daily: "Daily",
  weekly: "Weekly",
  biweekly: "Biweekly",
  monthly: "Monthly",
  quarterly: "Quarterly",
  yearly: "Yearly",
};

const RECURRENCE_OPTIONS = [
  { value: "", label: "No repeat" },
  { value: "daily", label: "Daily" },
  { value: "weekly", label: "Weekly" },
  { value: "biweekly", label: "Biweekly" },
  { value: "monthly", label: "Monthly" },
  { value: "quarterly", label: "Quarterly" },
  { value: "yearly", label: "Yearly" },
];

type FilterMode = "all" | "active" | "completed";
type SortMode = "created" | "due_date" | "priority";

const priorityOrder = { high: 0, medium: 1, low: 2 };

export default function TodosPage() {
  const { todos, isLoading, create, update, remove, bulkUpdate, bulkDelete, refresh } = useTodos();
  const [title, setTitle] = useState("");
  const [priority, setPriority] = useState<"low" | "medium" | "high">("medium");
  const [filter, setFilter] = useState<FilterMode>("all");
  const [sort, setSort] = useState<SortMode>("created");
  const [search, setSearch] = useState("");
  const [editingTagTodo, setEditingTagTodo] = useState<string | null>(null);
  const [selectedTagId, setSelectedTagId] = useState<string | null>(null);
  const [newTagName, setNewTagName] = useState("");
  const [selectedIds, setSelectedIds] = useState<Set<string>>(new Set());
  const { tags: allTags, create: createTag } = useTags();
  const { data: recurringTodos = [] } = useQuery({
    queryKey: ["recurring-todos"],
    queryFn: api.recurringTodos.list,
  });
  const recurringMap = useMemo(() => {
    const map = new Map<string, RecurringTodo>();
    for (const rt of recurringTodos) map.set(rt.todo_id, rt);
    return map;
  }, [recurringTodos]);

  const { data: allTodoTags = [] } = useQuery({
    queryKey: ["all-todo-tags"],
    queryFn: () => api.tags.listAllTodoTags(),
  });

  const todoTagMap = useMemo(() => {
    const map = new Map<string, string[]>();
    for (const tt of allTodoTags) {
      const existing = map.get(tt.todo_id) || [];
      existing.push(tt.tag_id);
      map.set(tt.todo_id, existing);
    }
    return map;
  }, [allTodoTags]);

  const tagMap = useMemo(() => {
    const map = new Map<string, Tag>();
    for (const t of allTags) map.set(t.id, t);
    return map;
  }, [allTags]);

  const queryClient = useQueryClient();
  const titleRef = useRef<HTMLInputElement>(null);

  const handleCreate = async () => {
    if (!title.trim()) return;
    await create.mutateAsync({ title, priority });
    setTitle("");
  };

  const toggleTodo = (id: string, is_completed: boolean) => {
    update.mutate(
      { id, is_completed: !is_completed },
      { onSuccess: () => queryClient.invalidateQueries({ queryKey: ["recurring-todos"] }) }
    );
  };

  const handleSetRecurrence = async (todoId: string, rule: string) => {
    if (rule) {
      await api.recurringTodos.set(todoId, rule);
    } else {
      await api.recurringTodos.remove(todoId);
    }
    queryClient.invalidateQueries({ queryKey: ["recurring-todos"] });
  };

  const toggleTodoTag = async (todoId: string, tagId: string) => {
    const current = todoTagMap.get(todoId) || [];
    const next = current.includes(tagId)
      ? current.filter((id) => id !== tagId)
      : [...current, tagId];
    await api.tags.setTodoTags(todoId, next);
    queryClient.invalidateQueries({ queryKey: ["all-todo-tags"] });
  };

  const handleCreateTagForTodo = async () => {
    if (!newTagName.trim() || !editingTagTodo) return;
    const created = await createTag.mutateAsync({ name: newTagName.trim() });
    // Auto-assign the new tag
    const current = todoTagMap.get(editingTagTodo) || [];
    await api.tags.setTodoTags(editingTagTodo, [...current, created.id]);
    queryClient.invalidateQueries({ queryKey: ["all-todo-tags"] });
    setNewTagName("");
  };

  const filtered = useMemo(() => {
    let result = [...todos];
    if (filter === "active") result = result.filter((t) => !t.is_completed);
    if (filter === "completed") result = result.filter((t) => t.is_completed);
    if (selectedTagId) {
      result = result.filter((t) => (todoTagMap.get(t.id) || []).includes(selectedTagId));
    }
    if (search) {
      const q = search.toLowerCase();
      result = result.filter((t) => t.title.toLowerCase().includes(q));
    }
    result.sort((a, b) => {
      if (sort === "priority") return (priorityOrder[a.priority] ?? 2) - (priorityOrder[b.priority] ?? 2);
      if (sort === "due_date") return (a.due_date ?? Infinity) - (b.due_date ?? Infinity);
      return b.created_at - a.created_at;
    });
    return result;
  }, [todos, filter, sort, search, selectedTagId, todoTagMap]);

  // Ref to latest filtered list for keyboard shortcuts
  const filteredRef = useRef(filtered);
  filteredRef.current = filtered;

  useEffect(() => {
    const handler = (e: KeyboardEvent) => {
      if (e.ctrlKey && e.key === "n") {
        e.preventDefault();
        titleRef.current?.focus();
      }
      if (e.ctrlKey && e.key === "e") {
        e.preventDefault();
        const current = filteredRef.current;
        if (current.length > 0) {
          setEditingTagTodo((prev) => (prev === current[0].id ? null : current[0].id));
        }
      }
    };
    window.addEventListener("keydown", handler);
    return () => window.removeEventListener("keydown", handler);
  }, []);

  const activeCount = todos.filter((t) => !t.is_completed).length;
  const completedIds = todos.filter((t) => t.is_completed).map((t) => t.id);
  const allIds = todos.map((t) => t.id);
  const allCompleted = todos.length > 0 && todos.every((t) => t.is_completed);

  return (
    <AuthGuard>
    <div className="min-h-screen bg-background">
      <NavHeader />
      <main className="container py-6 max-w-3xl mx-auto">
        <h1 className="text-2xl sm:text-3xl font-bold mb-4 sm:mb-6">Todos</h1>
        <div className="flex flex-col sm:flex-row gap-2 mb-4">
          <input ref={titleRef} value={title} onChange={(e) => setTitle(e.target.value)} onKeyDown={(e) => e.key === "Enter" && handleCreate()} placeholder="Add a new todo..." className="flex-1 p-2 border rounded-md bg-background" />
          <div className="flex gap-2">
            <select value={priority} onChange={(e) => setPriority(e.target.value as any)} className="flex-1 p-2 border rounded-md bg-background">
              <option value="low">Low</option>
              <option value="medium">Medium</option>
              <option value="high">High</option>
            </select>
            <button onClick={handleCreate} disabled={create.isPending} className="bg-primary text-primary-foreground px-4 py-2 rounded-md text-sm font-medium hover:bg-primary/90">
              Add
            </button>
          </div>
        </div>
        <div className="flex flex-col sm:flex-row items-start sm:items-center gap-2 sm:gap-3 mb-4">
          <input value={search} onChange={(e) => setSearch(e.target.value)} placeholder="Search todos..." className="p-1.5 text-sm border rounded-md bg-background w-full sm:w-48" />
          <div className="flex gap-1">
            {(["all", "active", "completed"] as const).map((f) => (
              <button key={f} onClick={() => setFilter(f)} className={`text-xs px-2.5 py-1 rounded-full border ${filter === f ? "bg-primary text-primary-foreground border-primary" : "hover:bg-accent"}`}>
                {f === "all" ? `All (${todos.length})` : f === "active" ? `Active (${activeCount})` : `Done (${todos.length - activeCount})`}
              </button>
            ))}
          </div>
          <select value={sort} onChange={(e) => setSort(e.target.value as SortMode)} className="text-xs p-1.5 border rounded-md bg-background sm:ml-auto w-full sm:w-auto">
            <option value="created">Newest</option>
            <option value="due_date">Due date</option>
            <option value="priority">Priority</option>
          </select>
        </div>
        {/* Tag filter bar */}
        {allTags.length > 0 && (
          <div className="flex flex-wrap items-center gap-1.5 mb-3">
            <button
              onClick={() => setSelectedTagId(null)}
              className={`text-xs px-2.5 py-1 rounded-full border transition-all ${
                !selectedTagId ? "bg-primary text-primary-foreground border-primary" : "hover:bg-accent"
              }`}
            >
              All
            </button>
            {allTags.map((tag) => (
              <button
                key={tag.id}
                onClick={() => setSelectedTagId(selectedTagId === tag.id ? null : tag.id)}
                className={`text-xs px-2.5 py-1 rounded-full border transition-all ${
                  selectedTagId === tag.id ? "text-white font-medium" : "hover:bg-accent"
                }`}
                style={{
                  backgroundColor: selectedTagId === tag.id ? tag.color : "transparent",
                  borderColor: tag.color,
                  color: selectedTagId === tag.id ? "#fff" : tag.color,
                }}
              >
                {tag.name}
              </button>
            ))}
          </div>
        )}
{todos.length > 0 && (
          <div className="flex gap-2 mb-4 text-xs">
            <input
              type="checkbox"
              checked={selectedIds.size > 0 && selectedIds.size === filtered.length}
              ref={(el) => {
                if (el) el.indeterminate = selectedIds.size > 0 && selectedIds.size < filtered.length;
              }}
              onChange={(e) => {
                if (e.target.checked) {
                  setSelectedIds(new Set(filtered.map((t) => t.id)));
                } else {
                  setSelectedIds(new Set());
                }
              }}
              className="h-4 w-4"
              title="Select all"
            />
            <button onClick={() => bulkUpdate.mutate({ ids: allIds, is_completed: !allCompleted })} disabled={bulkUpdate.isPending} className="px-2 py-1 rounded border hover:bg-accent">
              {allCompleted ? "Mark all active" : "Mark all done"}
            </button>
            <button onClick={() => {
              const selectedCompleted = Array.from(selectedIds).filter((id) => todos.find((t) => t.id === id)?.is_completed);
              if (selectedCompleted.length > 0) {
                if (confirm(`Delete ${selectedCompleted.length} selected item(s)?`)) {
                  bulkDelete.mutate(selectedCompleted);
                  setSelectedIds(new Set());
                }
              } else if (completedIds.length > 0) {
                if (confirm(`Delete all ${completedIds.length} completed items?`)) {
                  bulkDelete.mutate(completedIds);
                }
              }
            }} disabled={bulkDelete.isPending || (selectedIds.size === 0 && completedIds.length === 0)} className="px-2 py-1 rounded border text-destructive hover:bg-destructive/10">
              {selectedIds.size > 0 ? `Delete selected (${selectedIds.size})` : `Delete completed (${completedIds.length})`}
            </button>
          </div>
        )}
        {isLoading ? (
          <p className="text-muted-foreground">Loading todos...</p>
        ) : filtered.length === 0 ? (
          <p className="text-muted-foreground">{search ? "No todos match your search." : selectedTagId ? "No todos with this tag." : "No todos yet!"}</p>
        ) : (
          <div className="space-y-1.5">
            {filtered.map((todo) => (
              <div key={todo.id} className="rounded-lg border bg-card p-3 sm:p-3.5 shadow-sm">
                <div className="flex items-start sm:items-center gap-2 sm:gap-3">
                  <input
                       type="checkbox"
                       checked={selectedIds.has(todo.id)}
                       onChange={(e) => {
                         e.stopPropagation();
                         const newSet = new Set(selectedIds);
                         if (e.target.checked) {
                           newSet.add(todo.id);
                         } else {
                           newSet.delete(todo.id);
                         }
                         setSelectedIds(newSet);
                       }}
                       onClick={(e) => e.stopPropagation()}
                       className="h-4 w-4"
                       title="Select for bulk action"
                     />
                     <input type="checkbox" checked={todo.is_completed} onChange={() => toggleTodo(todo.id, todo.is_completed)} className="h-4 w-4" />
                  <div className="flex-1 min-w-0">
                    <p className={`font-medium truncate ${todo.is_completed ? "line-through text-muted-foreground" : ""}`}>{todo.title}</p>
                    <div className="flex items-center gap-2 mt-0.5 flex-wrap">
                      {todo.due_date && (
                        <p className={`text-xs ${todo.due_date < Date.now() / 1000 && !todo.is_completed ? "text-destructive" : "text-muted-foreground"}`}>
                          Due {new Date(todo.due_date * 1000).toLocaleDateString()}
                        </p>
                      )}
                      {(todoTagMap.get(todo.id) || []).map((tagId) => {
                        const tag = tagMap.get(tagId);
                        if (!tag) return null;
                        return (
                          <span
                            key={tagId}
                            className="inline-flex items-center px-1.5 py-0.5 rounded text-[10px] font-medium"
                            style={{ backgroundColor: tag.color + "20", color: tag.color }}
                          >
                            {tag.name}
                          </span>
                        );
                      })}
                      <button
                        onClick={(e) => { e.stopPropagation(); setEditingTagTodo(editingTagTodo === todo.id ? null : todo.id); }}
                        className="text-xs text-muted-foreground hover:text-foreground"
                        title="Manage tags"
                      >
                        🏷️
                      </button>
                      {recurringMap.has(todo.id) && (
                        <span className="text-xs text-primary font-medium" title={`Repeats ${RECURRENCE_LABELS[recurringMap.get(todo.id)!.recurrence_rule]?.toLowerCase() ?? ""}`}>
                          🔄 {RECURRENCE_LABELS[recurringMap.get(todo.id)!.recurrence_rule] ?? recurringMap.get(todo.id)!.recurrence_rule}
                        </span>
                      )}
                    </div>
                  </div>
                  <select
                  value={recurringMap.get(todo.id)?.recurrence_rule ?? ""}
                  onChange={(e) => handleSetRecurrence(todo.id, e.target.value)}
                  className="text-xs p-1 border rounded-md bg-background shrink-0 max-w-[90px]"
                  onClick={(e) => e.stopPropagation()}
                >
                  {RECURRENCE_OPTIONS.map((opt) => (
                    <option key={opt.value} value={opt.value}>
                      {opt.label}
                    </option>
                  ))}
                </select>
                <span className={`text-xs px-2 py-0.5 rounded-full shrink-0 ${todo.priority === "high" ? "bg-red-100 text-red-800" : todo.priority === "medium" ? "bg-yellow-100 text-yellow-800" : "bg-green-100 text-green-800"}`}>
                  {todo.priority}
                </span>
                <button onClick={async () => { await api.trash.archiveTodo(todo.id); queryClient.invalidateQueries({ queryKey: ["todos"] }); queryClient.invalidateQueries({ queryKey: ["all-todo-tags"] }); }} className="text-xs text-muted-foreground hover:text-foreground shrink-0" title="Archive">🗑️</button>
                <button onClick={() => remove.mutate(todo.id)} className="text-destructive text-xs sm:text-sm hover:underline shrink-0">Delete</button>
                </div>
                {editingTagTodo === todo.id && (
                  <div className="mt-2 pt-2 border-t border-muted">
                    <div className="flex flex-wrap gap-1.5 mb-1.5">
                      {allTags.map((tag) => {
                        const active = (todoTagMap.get(todo.id) || []).includes(tag.id);
                        return (
                          <button
                            key={tag.id}
                            type="button"
                            onClick={() => toggleTodoTag(todo.id, tag.id)}
                            className={`text-xs px-2 py-0.5 rounded-full border transition-all ${
                              active ? "text-white font-medium" : "hover:bg-accent"
                            }`}
                            style={{
                              backgroundColor: active ? tag.color : "transparent",
                              borderColor: tag.color,
                              color: active ? "#fff" : tag.color,
                            }}
                          >
                            {tag.name}
                          </button>
                        );
                      })}
                    </div>
                    <div className="flex gap-1.5">
                      <input
                        value={newTagName}
                        onChange={(e) => setNewTagName(e.target.value)}
                        onKeyDown={(e) => e.key === "Enter" && handleCreateTagForTodo()}
                        placeholder="New tag name..."
                        className="flex-1 p-1 text-xs border rounded-md bg-background"
                      />
                      <button
                        type="button"
                        onClick={handleCreateTagForTodo}
                        disabled={!newTagName.trim() || createTag.isPending}
                        className="text-xs px-2 py-1 rounded bg-primary text-primary-foreground hover:bg-primary/90 disabled:opacity-50"
                      >
                        + Add
                      </button>
                    </div>
                  </div>
                )}
              </div>
            ))}
          </div>
        )}
      </main>
    </div>
    </AuthGuard>
  );
}
