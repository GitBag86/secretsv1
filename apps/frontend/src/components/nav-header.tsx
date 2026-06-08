"use client";
import { useAuth } from "@/hooks";
import { useState, useEffect } from "react";

export function NavHeader() {
  const { user, isUnlocked, lock, sessionElapsed, sessionMinutes } = useAuth();
  const [mounted, setMounted] = useState(false);
  useEffect(() => setMounted(true), []);

  const isActive = (path: string) => {
    if (typeof window === "undefined") return false;
    return window.location.pathname === path;
  };

  const remaining = sessionMinutes * 60 - sessionElapsed;

  return (
    <header className="sticky top-0 z-50 w-full border-b bg-background/95 backdrop-blur">
      <div className="container flex h-14 items-center justify-between">
        <a href="/" className="flex items-center space-x-2 font-bold">KnowledgeBase</a>
        <nav className="flex items-center space-x-6">
          <a href="/notes" className={`text-sm font-medium ${isActive("/notes") ? "text-primary" : "hover:text-primary"}`}>Notes</a>
          <a href="/todos" className={`text-sm font-medium ${isActive("/todos") ? "text-primary" : "hover:text-primary"}`}>Todos</a>
          <a href="/calendar" className={`text-sm font-medium ${isActive("/calendar") ? "text-primary" : "hover:text-primary"}`}>Calendar</a>
          <a href="/settings" className={`text-sm font-medium ${isActive("/settings") ? "text-primary" : "hover:text-primary"}`}>Settings</a>
        </nav>
        {mounted && isUnlocked && (
          <div className="flex items-center gap-2 text-xs text-muted-foreground">
            <span className="tabular-nums">{Math.max(0, Math.floor(remaining / 60))}:{String(Math.max(0, remaining % 60)).padStart(2, "0")}</span>
            <button onClick={lock} className="hover:text-foreground" title="Lock database">🔒</button>
          </div>
        )}
      </div>
    </header>
  );
}
