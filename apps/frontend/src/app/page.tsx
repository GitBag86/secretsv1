"use client";
import { NavHeader } from "@/components/nav-header";

export default function Home() {
  return (
    <div className="min-h-screen bg-background">
      <NavHeader />
      <main className="container py-6">
        <h1 className="text-3xl font-bold">Dashboard</h1>
        <p className="text-muted-foreground mt-2">Welcome to your personal knowledge base</p>
        <div className="grid gap-4 md:grid-cols-3 mt-6">
          <a href="/notes" className="rounded-lg border bg-card p-6 shadow-sm hover:shadow-md transition-shadow">
            <h3 className="font-semibold text-lg">Notes</h3>
            <p className="text-muted-foreground text-sm mt-2">Create and organize your thoughts</p>
          </a>
          <a href="/todos" className="rounded-lg border bg-card p-6 shadow-sm hover:shadow-md transition-shadow">
            <h3 className="font-semibold text-lg">Todos</h3>
            <p className="text-muted-foreground text-sm mt-2">Track your tasks and goals</p>
          </a>
          <a href="/calendar" className="rounded-lg border bg-card p-6 shadow-sm hover:shadow-md transition-shadow">
            <h3 className="font-semibold text-lg">Calendar</h3>
            <p className="text-muted-foreground text-sm mt-2">Manage your schedule</p>
          </a>
        </div>
      </main>
    </div>
  );
}
