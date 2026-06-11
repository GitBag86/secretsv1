"use client";
import { useAuth } from "@/hooks";
import { useState, useEffect } from "react";
import Link from "next/link";
import { SearchPalette } from "./search-palette";

export function NavHeader() {
  const { user, isUnlocked, lock, sessionElapsed, sessionMinutes } = useAuth();
  const [mounted, setMounted] = useState(false);
  const [mobileOpen, setMobileOpen] = useState(false);
  useEffect(() => setMounted(true), []);

  const isActive = (path: string) => {
    if (typeof window === "undefined") return false;
    return window.location.pathname === path;
  };

  const remaining = sessionMinutes * 60 - sessionElapsed;

  return (
    <>
      <SearchPalette />
    <header className="sticky top-0 z-50 w-full border-b bg-background/95 backdrop-blur">
      <div className="container flex h-14 items-center justify-between">
        <Link href="/" className="flex items-center space-x-2 font-bold">KnowledgeBase</Link>
        {/* Desktop nav */}
        <nav className="hidden sm:flex items-center space-x-6">
          <Link href="/notes" className={`text-sm font-medium ${isActive("/notes") ? "text-primary" : "hover:text-primary"}`}>Notes</Link>
          <Link href="/todos" className={`text-sm font-medium ${isActive("/todos") ? "text-primary" : "hover:text-primary"}`}>Todos</Link>
          <Link href="/calendar" className={`text-sm font-medium ${isActive("/calendar") ? "text-primary" : "hover:text-primary"}`}>Calendar</Link>
          <Link href="/settings" className={`text-sm font-medium ${isActive("/settings") ? "text-primary" : "hover:text-primary"}`}>Settings</Link>
          <Link href="/trash" className={`text-sm font-medium ${isActive("/trash") ? "text-primary" : "hover:text-primary"}`}>Trash</Link>
        </nav>
        <div className="flex items-center gap-2">
          {mounted && isUnlocked && (
            <div className="flex items-center gap-2 text-xs text-muted-foreground">
              <span className="tabular-nums hidden sm:inline">{Math.max(0, Math.floor(remaining / 60))}:{String(Math.max(0, remaining % 60)).padStart(2, "0")}</span>
              <button onClick={lock} className="hover:text-foreground" title="Lock database">🔒</button>
            </div>
          )}
          {/* Mobile menu button */}
          <button
            onClick={() => setMobileOpen(!mobileOpen)}
            className="sm:hidden p-1.5 rounded-md hover:bg-accent"
            aria-label="Toggle menu"
          >
            <svg className="w-5 h-5" fill="none" stroke="currentColor" viewBox="0 0 24 24">
              {mobileOpen ? (
                <path strokeLinecap="round" strokeLinejoin="round" strokeWidth={2} d="M6 18L18 6M6 6l12 12" />
              ) : (
                <path strokeLinecap="round" strokeLinejoin="round" strokeWidth={2} d="M4 6h16M4 12h16M4 18h16" />
              )}
            </svg>
          </button>
        </div>
      </div>
      {/* Mobile nav */}
      {mobileOpen && (
        <div className="sm:hidden border-t bg-background px-4 py-3 space-y-2">
          <Link href="/notes" onClick={() => setMobileOpen(false)} className={`block px-3 py-2 rounded-md text-sm font-medium ${isActive("/notes") ? "bg-primary/10 text-primary" : "hover:bg-accent"}`}>Notes</Link>
          <Link href="/todos" onClick={() => setMobileOpen(false)} className={`block px-3 py-2 rounded-md text-sm font-medium ${isActive("/todos") ? "bg-primary/10 text-primary" : "hover:bg-accent"}`}>Todos</Link>
          <Link href="/calendar" onClick={() => setMobileOpen(false)} className={`block px-3 py-2 rounded-md text-sm font-medium ${isActive("/calendar") ? "bg-primary/10 text-primary" : "hover:bg-accent"}`}>Calendar</Link>
          <Link href="/settings" onClick={() => setMobileOpen(false)} className={`block px-3 py-2 rounded-md text-sm font-medium ${isActive("/settings") ? "bg-primary/10 text-primary" : "hover:bg-accent"}`}>Settings</Link>
          <Link href="/trash" onClick={() => setMobileOpen(false)} className={`block px-3 py-2 rounded-md text-sm font-medium ${isActive("/trash") ? "bg-primary/10 text-primary" : "hover:bg-accent"}`}>Trash</Link>
        </div>
      )}
    </header>
    </>
  );
}
