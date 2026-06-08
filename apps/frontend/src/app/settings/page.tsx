"use client";
import { NavHeader } from "@/components/nav-header";
import { useAuth } from "@/hooks";
import { api } from "@/lib/api";
import { useState, useEffect } from "react";
import { useTheme } from "next-themes";

export default function SettingsPage() {
  const { user, logout, isUnlocked, lock, sessionMinutes, setSessionMinutes, sessionElapsed } = useAuth();
  const { theme, setTheme } = useTheme();
  const [mounted, setMounted] = useState(false);
  const [timeoutInput, setTimeoutInput] = useState(String(sessionMinutes));
  const [saving, setSaving] = useState(false);
  const [showRotate, setShowRotate] = useState(false);
  const [curPw, setCurPw] = useState("");
  const [newPw, setNewPw] = useState("");
  const [rotating, setRotating] = useState(false);
  const [rotateResult, setRotateResult] = useState<string | null>(null);

  useEffect(() => setMounted(true), []);

  const saveTimeout = async () => {
    const m = parseInt(timeoutInput, 10);
    if (isNaN(m) || m < 1 || m > 480) return;
    setSaving(true);
    await api.auth.setSessionTimeout(m);
    setSessionMinutes(m);
    setSaving(false);
  };

  const handleRotate = async () => {
    if (!curPw || !newPw || newPw.length < 4) return;
    setRotating(true);
    setRotateResult(null);
    try {
      const res = await api.encryption.rotate(curPw, newPw);
      setRotateResult(`Key rotated: ${res.notes} notes, ${res.todos} todos, ${res.events} events re-encrypted.`);
      setCurPw("");
      setNewPw("");
      setShowRotate(false);
    } catch (e: any) {
      setRotateResult(`Error: ${e}`);
    }
    setRotating(false);
  };

  return (
    <div className="min-h-screen bg-background">
      <NavHeader />
      <main className="container py-6 max-w-2xl mx-auto space-y-8">
        <h1 className="text-3xl font-bold">Settings</h1>

        {user && (
          <section className="rounded-lg border bg-card p-6 space-y-4">
            <h2 className="text-lg font-semibold">Account</h2>
            <div className="text-sm space-y-1">
              <p><span className="text-muted-foreground">Email:</span> {user.email}</p>
              {user.name && <p><span className="text-muted-foreground">Name:</span> {user.name}</p>}
            </div>
            <button onClick={logout} className="text-destructive text-sm hover:underline">Sign out</button>
          </section>
        )}

        <section className="rounded-lg border bg-card p-6 space-y-4">
          <h2 className="text-lg font-semibold">Appearance</h2>
          <div className="flex gap-2">
            {["light", "dark", "system"].map((t) => (
              <button key={t} onClick={() => setTheme(t)}
                className={`px-4 py-2 rounded-md text-sm font-medium border ${theme === t ? "bg-primary text-primary-foreground border-primary" : "hover:bg-accent"}`}>
                {t.charAt(0).toUpperCase() + t.slice(1)}
              </button>
            ))}
          </div>
        </section>

        <section className="rounded-lg border bg-card p-6 space-y-4">
          <h2 className="text-lg font-semibold">Session</h2>
          {isUnlocked ? (
            <>
              <p className="text-sm text-muted-foreground">
                Session active · {Math.floor(sessionElapsed / 60)}m {sessionElapsed % 60}s elapsed
              </p>
              <div className="flex items-center gap-2">
                <label className="text-sm">Auto-lock after</label>
                <input type="number" min={1} max={480} value={timeoutInput} onChange={(e) => setTimeoutInput(e.target.value)}
                  className="w-20 p-1.5 border rounded-md bg-background text-sm text-center" />
                <span className="text-sm text-muted-foreground">minutes</span>
                <button onClick={saveTimeout} disabled={saving}
                  className="bg-primary text-primary-foreground px-3 py-1.5 rounded-md text-xs font-medium hover:bg-primary/90 disabled:opacity-50">
                  {saving ? "..." : "Save"}
                </button>
              </div>
              <button onClick={lock} className="text-destructive text-sm hover:underline">Lock database now</button>
            </>
          ) : (
            <p className="text-sm text-muted-foreground">Database is locked. <a href="/unlock" className="text-primary hover:underline">Unlock</a> to configure session settings.</p>
          )}
        </section>

        <section className="rounded-lg border bg-card p-6 space-y-4">
          <h2 className="text-lg font-semibold">Encryption</h2>
          <p className="text-sm text-muted-foreground">
            Content is encrypted with AES-256-GCM at rest. Set a master password to protect your data.
          </p>
          {isUnlocked && (
            <div className="space-y-3">
              <button onClick={() => { setShowRotate(!showRotate); setRotateResult(null); }} className="text-sm border px-3 py-1.5 rounded-md hover:bg-accent">
                {showRotate ? "Cancel" : "Rotate Encryption Key"}
              </button>
              {showRotate && (
                <div className="space-y-2 border rounded-md p-3 bg-muted/30">
                  <input type="password" value={curPw} onChange={(e) => setCurPw(e.target.value)} placeholder="Current password" className="w-full p-2 border rounded-md bg-background text-sm" />
                  <input type="password" value={newPw} onChange={(e) => setNewPw(e.target.value)} placeholder="New password (min 4 chars)" className="w-full p-2 border rounded-md bg-background text-sm" />
                  <button onClick={handleRotate} disabled={rotating || !curPw || newPw.length < 4} className="bg-primary text-primary-foreground px-3 py-1.5 rounded-md text-xs font-medium hover:bg-primary/90 disabled:opacity-50">
                    {rotating ? "Rotating..." : "Rotate Key"}
                  </button>
                </div>
              )}
              {rotateResult && <p className="text-sm text-muted-foreground">{rotateResult}</p>}
            </div>
          )}
          {!isUnlocked && <a href="/unlock" className="inline-block bg-primary text-primary-foreground px-4 py-2 rounded-md text-sm font-medium hover:bg-primary/90">Set Master Password</a>}
        </section>

        <section className="rounded-lg border bg-card p-6 space-y-4">
          <h2 className="text-lg font-semibold">Sync</h2>
          <p className="text-sm text-muted-foreground">
            Sync your data across devices using Supabase. Configure in your environment variables.
          </p>
          <button disabled className="bg-primary/50 text-primary-foreground px-4 py-2 rounded-md text-sm font-medium cursor-not-allowed">
            Sync Status — Coming Soon
          </button>
        </section>

        <section className="rounded-lg border bg-card p-6 space-y-4">
          <h2 className="text-lg font-semibold">About</h2>
          <div className="text-sm text-muted-foreground space-y-1">
            <p>KnowledgeBase v1.0.0</p>
            <p>Local-first, encrypted personal knowledge system</p>
          </div>
        </section>
      </main>
    </div>
  );
}
