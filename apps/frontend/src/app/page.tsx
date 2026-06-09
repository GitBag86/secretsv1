"use client";
import { NavHeader } from "@/components/nav-header";
import { useQuery } from "@tanstack/react-query";
import { api } from "@/lib/api";
import { useMemo } from "react";
import Link from "next/link";

export default function Home() {
  const { data: notes = [], isLoading: notesLoading } = useQuery({ queryKey: ["notes"], queryFn: api.notes.list });
  const { data: todos = [], isLoading: todosLoading } = useQuery({ queryKey: ["todos"], queryFn: api.todos.list });
  const { data: notebooks = [], isLoading: notebooksLoading } = useQuery({ queryKey: ["notebooks"], queryFn: api.notebooks.list });
  const { data: events = [], isLoading: eventsLoading } = useQuery({ queryKey: ["calendar"], queryFn: api.calendar.list });
  const { data: tags = [] } = useQuery({ queryKey: ["tags"], queryFn: api.tags.list });
  const { data: allTodoTags = [] } = useQuery({ queryKey: ["all-todo-tags"], queryFn: () => api.tags.listAllTodoTags() });
  const { data: allNoteTags = [] } = useQuery({ queryKey: ["all-note-tags"], queryFn: () => api.tags.listAllNoteTags() });

  const isLoading = notesLoading || todosLoading || notebooksLoading || eventsLoading;

  const stats = useMemo(() => {
    const activeTodos = todos.filter((t) => !t.is_completed);
    const completedTodos = todos.filter((t) => t.is_completed);
    const highPriority = activeTodos.filter((t) => t.priority === "high");
    const overdueTodos = activeTodos.filter((t) => t.due_date && t.due_date < Date.now() / 1000);
    const dueSoon = activeTodos.filter((t) => t.due_date && t.due_date < Date.now() / 1000 + 86400 * 3);

    const now = Date.now() / 1000;
    const upcomingEvents = events
      .filter((e) => e.start_time > now && e.start_time < now + 86400 * 7)
      .sort((a, b) => a.start_time - b.start_time)
      .slice(0, 6);

    const recentNotes = [...notes]
      .sort((a, b) => b.updated_at - a.updated_at)
      .slice(0, 5);

    // Tag usage: count how many times each tag is used across todos and notes
    const tagCounts = new Map<string, number>();
    for (const tt of allTodoTags) tagCounts.set(tt.tag_id, (tagCounts.get(tt.tag_id) || 0) + 1);
    for (const nt of allNoteTags) tagCounts.set(nt.tag_id, (tagCounts.get(nt.tag_id) || 0) + 1);
    const popularTags = [...tagCounts.entries()]
      .sort((a, b) => b[1] - a[1])
      .slice(0, 8)
      .map(([tagId, count]) => ({ tagId, count }));

    const pinnedNotes = notes.filter((n) => n.is_pinned).length;
    const todoCompletion = todos.length > 0 ? Math.round((completedTodos.length / todos.length) * 100) : 0;

    return {
      totalNotes: notes.length,
      pinnedNotes,
      notebookCount: notebooks.length,
      totalTodos: todos.length,
      activeTodos: activeTodos.length,
      completedTodos: completedTodos.length,
      todoCompletion,
      highPriority: highPriority.length,
      overdueTodos: overdueTodos.length,
      dueSoon: dueSoon.length,
      totalEvents: events.length,
      upcomingEvents,
      recentNotes,
      popularTags,
      totalTagged: allTodoTags.length + allNoteTags.length,
    };
  }, [notes, todos, notebooks, events, allTodoTags, allNoteTags]);

  return (
    <div className="min-h-screen bg-background">
      <NavHeader />
      <main className="container py-6 max-w-5xl mx-auto">
        <div className="flex items-center justify-between mb-6 sm:mb-8">
          <div>
            <h1 className="text-2xl sm:text-3xl font-bold">Dashboard</h1>
            <p className="text-xs sm:text-sm text-muted-foreground mt-1">Welcome to your personal knowledge base</p>
          </div>
        </div>

        {isLoading ? (
          <div className="grid gap-4 sm:grid-cols-2 lg:grid-cols-4 mb-8">
            {[...Array(4)].map((_, i) => (
              <div key={i} className="rounded-xl border bg-card p-5 shadow-sm animate-pulse">
                <div className="h-4 bg-muted rounded w-16 mb-3" />
                <div className="h-8 bg-muted rounded w-12 mb-2" />
                <div className="h-3 bg-muted rounded w-24" />
              </div>
            ))}
          </div>
        ) : (
          <>
          <div className="grid gap-4 sm:grid-cols-2 lg:grid-cols-4 mb-8">
            <StatCard
              href="/notes"
              label="Notes"
              value={stats.totalNotes}
              sub={`${stats.pinnedNotes} pinned · ${stats.notebookCount} notebooks`}
              icon="📝"
              color="from-blue-500/20 to-blue-600/10"
            />
            <StatCard
              href="/todos"
              label="Todos"
              value={stats.totalTodos}
              sub={`${stats.activeTodos} active · ${stats.completedTodos} done`}
              icon="✅"
              color="from-green-500/20 to-green-600/10"
            />
            <StatCard
              href="/todos"
              label="Priority"
              value={stats.highPriority}
              sub={`${stats.overdueTodos} overdue · ${stats.dueSoon} due soon`}
              icon="🔥"
              color="from-orange-500/20 to-orange-600/10"
            />
            <StatCard
              href="/calendar"
              label="Events"
              value={stats.totalEvents}
              sub={`${stats.upcomingEvents.length} upcoming this week`}
              icon="📅"
              color="from-purple-500/20 to-purple-600/10"
            />
          </div>

          {/* Todo Progress + Upcoming Events */}
          <div className="grid gap-6 md:grid-cols-2 mb-8">
          {/* Todo Completion Progress */}
          <div className="rounded-xl border bg-card p-6 shadow-sm">
            <h3 className="font-semibold text-lg mb-1">Todo Progress</h3>
            <p className="text-sm text-muted-foreground mb-4">
              {stats.completedTodos} of {stats.totalTodos} tasks completed
            </p>
            <div className="h-3 w-full bg-muted rounded-full overflow-hidden mb-3">
              <div
                className="h-full bg-primary rounded-full transition-all duration-500"
                style={{ width: `${stats.todoCompletion}%` }}
              />
            </div>
            <div className="flex justify-between text-xs text-muted-foreground">
              <span>{stats.todoCompletion}% complete</span>
              <span>
                {stats.overdueTodos > 0 && (
                  <span className="text-destructive font-medium">{stats.overdueTodos} overdue · </span>
                )}
                {stats.dueSoon - stats.overdueTodos > 0 && (
                  <span className="text-orange-500">{stats.dueSoon - stats.overdueTodos} due in 3 days</span>
                )}
                {stats.overdueTodos === 0 && stats.dueSoon === 0 && <span>All caught up!</span>}
              </span>
            </div>
          </div>

          {/* Upcoming Events */}
          <div className="rounded-xl border bg-card p-6 shadow-sm">
            <h3 className="font-semibold text-lg mb-3">Upcoming Events</h3>
            {stats.upcomingEvents.length === 0 ? (
              <p className="text-sm text-muted-foreground">No events in the next 7 days.</p>
            ) : (
              <div className="space-y-2">
                {stats.upcomingEvents.map((ev) => (
                  <div key={ev.id} className="flex items-center gap-3">
                    <div
                      className="w-2 h-2 rounded-full shrink-0"
                      style={{ backgroundColor: ev.color }}
                    />
                    <div className="flex-1 min-w-0">
                      <p className="text-sm font-medium truncate">{ev.title}</p>
                      <p className="text-xs text-muted-foreground">
                        {new Date(ev.start_time * 1000).toLocaleString("en-US", {
                          weekday: "short",
                          month: "short",
                          day: "numeric",
                          hour: ev.all_day ? undefined : "numeric",
                          minute: ev.all_day ? undefined : "2-digit",
                        })}
                        {ev.all_day && " · All day"}
                      </p>
                    </div>
                  </div>
                ))}
              </div>
            )}
            <Link href="/calendar" className="text-xs text-primary hover:underline mt-3 inline-block">
              View calendar →
            </Link>
          </div>
        </div>          {/* Recent Notes + Popular Tags */}
        <div className="grid gap-6 md:grid-cols-2">
          {/* Recent Notes */}
          <div className="rounded-xl border bg-card p-6 shadow-sm">
            <h3 className="font-semibold text-lg mb-3">Recent Notes</h3>
            {stats.recentNotes.length === 0 ? (
              <p className="text-sm text-muted-foreground">No notes yet.</p>
            ) : (
              <div className="space-y-2">
                {stats.recentNotes.map((n) => (
                  <Link
                    key={n.id}
                    href="/notes"
                    className="block p-2 -mx-2 rounded-md hover:bg-accent/50 transition-colors"
                  >
                    <p className="text-sm font-medium truncate">{n.title || "Untitled"}</p>
                    <p className="text-xs text-muted-foreground">
                      {n.word_count} words · {n.reading_time} min read
                      {n.is_pinned && <span className="ml-2">📌</span>}
                    </p>
                  </Link>
                ))}
              </div>
            )}
            <Link href="/notes" className="text-xs text-primary hover:underline mt-3 inline-block">
              View all notes →
            </Link>
          </div>

          {/* Analytics / Charts */}
          <div className="rounded-xl border bg-card p-6 shadow-sm">
            <h3 className="font-semibold text-lg mb-3">Analytics</h3>
            <div className="space-y-4">
              {/* Priority Distribution */}
              <div>
                <p className="text-xs font-medium text-muted-foreground mb-2">Todo Priority</p>
                <div className="space-y-1.5">
                  {[
                    { label: "High", count: stats.highPriority, color: "bg-red-500" },
                    { label: "Medium", count: stats.activeTodos - stats.highPriority, color: "bg-yellow-500" },
                    { label: "Low", count: todos.filter((t) => !t.is_completed && t.priority === "low").length, color: "bg-green-500" },
                    { label: "Completed", count: stats.completedTodos, color: "bg-blue-500" },
                  ].map(({ label, count, color }) => {
                    const max = stats.totalTodos || 1;
                    const pct = Math.round((count / max) * 100);
                    return (
                      <div key={label} className="flex items-center gap-2">
                        <span className="text-xs w-20 shrink-0 text-muted-foreground">{label}</span>
                        <div className="flex-1 h-2.5 bg-muted rounded-full overflow-hidden">
                          <div className={`h-full ${color} rounded-full transition-all duration-500`} style={{ width: `${pct}%` }} />
                        </div>
                        <span className="text-xs w-6 text-right text-muted-foreground">{count}</span>
                      </div>
                    );
                  })}
                </div>
              </div>
              {/* Notes vs Todos */}
              <div>
                <p className="text-xs font-medium text-muted-foreground mb-2">Content Overview</p>
                <div className="flex items-end gap-3 h-28">
                  {[
                    { label: "Notes", count: stats.totalNotes, color: "bg-blue-500" },
                    { label: "Todos", count: stats.totalTodos, color: "bg-green-500" },
                    { label: "Events", count: stats.totalEvents, color: "bg-purple-500" },
                    { label: "Notebooks", count: stats.notebookCount, color: "bg-orange-500" },
                  ].map(({ label, count, color }) => {
                    const max = Math.max(stats.totalNotes, stats.totalTodos, stats.totalEvents, stats.notebookCount, 1);
                    const height = Math.round((count / max) * 100);
                    return (
                      <div key={label} className="flex-1 flex flex-col items-center gap-1">
                        <div className="relative w-full flex items-end justify-center" style={{ height: '112px' }}>
                          <div
                            className={`w-full max-w-[40px] ${color} rounded-t-md transition-all duration-500`}
                            style={{ height: `${height}%` }}
                          />
                        </div>
                        <span className="text-[10px] text-muted-foreground">{label}</span>
                        <span className="text-xs font-medium">{count}</span>
                      </div>
                    );
                  })}
                </div>
              </div>
            </div>
          </div>

          {/* Popular Tags */}
          <div className="rounded-xl border bg-card p-6 shadow-sm">
            <h3 className="font-semibold text-lg mb-1">Popular Tags</h3>
            <p className="text-xs text-muted-foreground mb-3">
              {tags.length} tags · {stats.totalTagged} total taggings across notes and todos
            </p>
            {tags.length === 0 ? (
              <p className="text-sm text-muted-foreground">No tags yet.</p>
            ) : stats.popularTags.length === 0 ? (
              <p className="text-sm text-muted-foreground">Tags exist but none are in use.</p>
            ) : (
              <div className="flex flex-wrap gap-2">
                {stats.popularTags.map(({ tagId, count }) => {
                  const tag = tags.find((t) => t.id === tagId);
                  if (!tag) return null;
                  return (
                    <div
                      key={tagId}
                      className="flex items-center gap-1.5 px-2.5 py-1.5 rounded-lg text-xs font-medium"
                      style={{ backgroundColor: tag.color + "18", color: tag.color }}
                    >
                      <span>{tag.name}</span>
                      <span
                        className="ml-0.5 px-1.5 py-0.5 rounded-full text-[10px] font-bold"
                        style={{ backgroundColor: tag.color + "30", color: tag.color }}
                      >
                        {count}
                      </span>
                    </div>
                  );
                })}
              </div>
            )}
          </div>
        </div>
        </>
      </main>
    </div>
  );
}

function StatCard({
  href, label, value, sub, icon, color,
}: {
  href: string; label: string; value: number; sub: string; icon: string; color: string;
}) {
  return (
    <Link href={href} className={`rounded-xl border bg-gradient-to-br ${color} p-4 sm:p-5 shadow-sm hover:shadow-md transition-all hover:-translate-y-0.5`}>
      <div className="flex items-start justify-between">
        <div>
          <p className="text-xs sm:text-sm font-medium text-muted-foreground">{label}</p>
          <p className="text-2xl sm:text-3xl font-bold mt-1">{value}</p>
          <p className="text-[10px] sm:text-xs text-muted-foreground mt-1.5">{sub}</p>
        </div>
        <span className="text-xl sm:text-2xl">{icon}</span>
      </div>
    </Link>
  );
}

