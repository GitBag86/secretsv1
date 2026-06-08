"use client";
import { useTodos } from "@/hooks";
import { NavHeader } from "@/components/nav-header";
import { useState, useMemo } from "react";

type FilterMode = "all" | "active" | "completed";
type SortMode = "created" | "due_date" | "priority";

const priorityOrder = { high: 0, medium: 1, low: 2 };

export default function TodosPage() {
  const { todos, isLoading, create, update, remove, bulkUpdate, bulkDelete } = useTodos();
  const [title, setTitle] = useState("");
  const [priority, setPriority] = useState<"low" | "medium" | "high">("medium");
  const [filter, setFilter] = useState<FilterMode>("all");
  const [sort, setSort] = useState<SortMode>("created");
  const [search, setSearch] = useState("");

  const handleCreate = async () => {
    if (!title.trim()) return;
    await create.mutateAsync({ title, priority });
    setTitle("");
  };

  const toggleTodo = (id: string, is_completed: boolean) => {
    update.mutate({ id, is_completed: !is_completed });
  };

  const filtered = useMemo(() => {
    let result = [...todos];
    if (filter === "active") result = result.filter((t) => !t.is_completed);
    if (filter === "completed") result = result.filter((t) => t.is_completed);
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
  }, [todos, filter, sort, search]);

  const activeCount = todos.filter((t) => !t.is_completed).length;
  const completedIds = todos.filter((t) => t.is_completed).map((t) => t.id);
  const allIds = todos.map((t) => t.id);
  const allCompleted = todos.length > 0 && todos.every((t) => t.is_completed);

  return (
    <div className="min-h-screen bg-background">
      <NavHeader />
      <main className="container py-6 max-w-3xl mx-auto">
        <h1 className="text-3xl font-bold mb-6">Todos</h1>
        <div className="flex gap-2 mb-4">
          <input value={title} onChange={(e) => setTitle(e.target.value)} onKeyDown={(e) => e.key === "Enter" && handleCreate()} placeholder="Add a new todo..." className="flex-1 p-2 border rounded-md bg-background" />
          <select value={priority} onChange={(e) => setPriority(e.target.value as any)} className="p-2 border rounded-md bg-background">
            <option value="low">Low</option>
            <option value="medium">Medium</option>
            <option value="high">High</option>
          </select>
          <button onClick={handleCreate} disabled={create.isPending} className="bg-primary text-primary-foreground px-4 py-2 rounded-md text-sm font-medium hover:bg-primary/90">
            Add
          </button>
        </div>
        <div className="flex items-center gap-3 mb-4 flex-wrap">
          <input value={search} onChange={(e) => setSearch(e.target.value)} placeholder="Search todos..." className="p-1.5 text-sm border rounded-md bg-background w-48" />
          <div className="flex gap-1">
            {(["all", "active", "completed"] as const).map((f) => (
              <button key={f} onClick={() => setFilter(f)} className={`text-xs px-2.5 py-1 rounded-full border ${filter === f ? "bg-primary text-primary-foreground border-primary" : "hover:bg-accent"}`}>
                {f === "all" ? `All (${todos.length})` : f === "active" ? `Active (${activeCount})` : `Done (${todos.length - activeCount})`}
              </button>
            ))}
          </div>
          <select value={sort} onChange={(e) => setSort(e.target.value as SortMode)} className="text-xs p-1.5 border rounded-md bg-background ml-auto">
            <option value="created">Newest</option>
            <option value="due_date">Due date</option>
            <option value="priority">Priority</option>
          </select>
        </div>
        {todos.length > 0 && (
          <div className="flex gap-2 mb-4 text-xs">
            <button onClick={() => bulkUpdate.mutate({ ids: allIds, is_completed: !allCompleted })} disabled={bulkUpdate.isPending} className="px-2 py-1 rounded border hover:bg-accent">
              {allCompleted ? "Mark all active" : "Mark all done"}
            </button>
            <button onClick={() => bulkDelete.mutate(completedIds)} disabled={bulkDelete.isPending || completedIds.length === 0} className="px-2 py-1 rounded border text-destructive hover:bg-destructive/10">
              Delete completed ({completedIds.length})
            </button>
          </div>
        )}
        {isLoading ? (
          <p className="text-muted-foreground">Loading todos...</p>
        ) : filtered.length === 0 ? (
          <p className="text-muted-foreground">{search ? "No todos match your search." : "No todos yet!"}</p>
        ) : (
          <div className="space-y-1.5">
            {filtered.map((todo) => (
              <div key={todo.id} className="flex items-center gap-3 rounded-lg border bg-card p-3.5 shadow-sm">
                <input type="checkbox" checked={todo.is_completed} onChange={() => toggleTodo(todo.id, todo.is_completed)} className="h-4 w-4" />
                <div className="flex-1 min-w-0">
                  <p className={`font-medium truncate ${todo.is_completed ? "line-through text-muted-foreground" : ""}`}>{todo.title}</p>
                  {todo.due_date && (
                    <p className={`text-xs mt-0.5 ${todo.due_date < Date.now() / 1000 && !todo.is_completed ? "text-destructive" : "text-muted-foreground"}`}>
                      Due {new Date(todo.due_date * 1000).toLocaleDateString()}
                    </p>
                  )}
                </div>
                <span className={`text-xs px-2 py-0.5 rounded-full shrink-0 ${todo.priority === "high" ? "bg-red-100 text-red-800" : todo.priority === "medium" ? "bg-yellow-100 text-yellow-800" : "bg-green-100 text-green-800"}`}>
                  {todo.priority}
                </span>
                <button onClick={() => remove.mutate(todo.id)} className="text-destructive text-sm hover:underline shrink-0">Delete</button>
              </div>
            ))}
          </div>
        )}
      </main>
    </div>
  );
}
