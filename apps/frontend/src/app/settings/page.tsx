"use client";
import { NavHeader } from "@/components/nav-header";
import { useAuth } from "@/hooks";
import { api } from "@/lib/api";
import { useState, useEffect } from "react";
import { useTheme } from "next-themes";
import { useQuery, useMutation, useQueryClient } from "@tanstack/react-query";

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
  const [syncUrl, setSyncUrl] = useState("");
  const [syncKey, setSyncKey] = useState("");
  const [configuring, setConfiguring] = useState(false);
  const [syncMessage, setSyncMessage] = useState<string | null>(null);
  const [syncConfigured, setSyncConfigured] = useState(false);
  const [syncing, setSyncing] = useState(false);
  const [pulling, setPulling] = useState(false);
  const [syncPending, setSyncPending] = useState(0);
  const [syncResult, setSyncResult] = useState<any>(null);
  const queryClient = useQueryClient();
  const { data: allTags = [] } = useQuery({ queryKey: ["tags"], queryFn: api.tags.list });
  const [editTagId, setEditTagId] = useState<string | null>(null);
  const [editTagName, setEditTagName] = useState("");
  const [editTagColor, setEditTagColor] = useState("#6b7280");
  const [newTagName, setNewTagName] = useState("");
  const [newTagColor, setNewTagColor] = useState("#6b7280");
  const [exporting, setExporting] = useState(false);
  const [importing, setImporting] = useState(false);

  const tagUpdate = useMutation({
    mutationFn: ({ id, name, color }: { id: string; name: string; color: string }) =>
      api.tags.update(id, { name, color }),
    onSuccess: () => {
      queryClient.invalidateQueries({ queryKey: ["tags"] });
      setEditTagId(null);
    },
  });

  const tagDelete = useMutation({
    mutationFn: api.tags.delete,
    onSuccess: () => queryClient.invalidateQueries({ queryKey: ["tags"] }),
  });

  const tagCreate = useMutation({
    mutationFn: (data: { name: string; color?: string }) => api.tags.create(data),
    onSuccess: () => {
      queryClient.invalidateQueries({ queryKey: ["tags"] });
      setNewTagName("");
      setNewTagColor("#6b7280");
    },
  });

  useEffect(() => setMounted(true), []);

  // Load sync config on mount
  useEffect(() => {
    if (!isUnlocked) return;
    api.sync.status().then((s) => {
      setSyncPending(s.pending);
      setSyncConfigured(s.configured);
    }).catch(() => {});
    api.sync.getConfig().then((cfg) => {
      if (cfg.url) setSyncUrl(cfg.url);
      setSyncConfigured(!!cfg.url);
    }).catch(() => {});
  }, [isUnlocked]);

  const handleConfigureSync = async () => {
    setConfiguring(true);
    setSyncMessage(null);
    try {
      const res = await api.sync.configure(syncUrl, syncKey);
      setSyncConfigured(true);
      setSyncMessage(res.connection_ok ? "Connection OK! Sync configured." : "Saved but couldn't connect to Supabase. Check your URL and key.");
    } catch (e: any) {
      setSyncMessage(`Error: ${e}`);
    }
    setConfiguring(false);
  };

  const handleSyncPush = async () => {
    setSyncing(true);
    setSyncResult(null);
    try {
      const res = await api.sync.push();
      setSyncResult(res);
      setSyncPending(res.remaining || 0);
    } catch (e: any) {
      setSyncResult({ error: String(e) });
    }
    setSyncing(false);
  };

  const handleSyncPull = async () => {
    setPulling(true);
    setSyncResult(null);
    try {
      const res = await api.sync.pull();
      setSyncResult(res);
    } catch (e: any) {
      setSyncResult({ error: String(e) });
    }
    setPulling(false);
  };

  const handleSyncStatus = async () => {
    setSyncResult(null);
    try {
      const res = await api.sync.status();
      setSyncResult(res);
      setSyncPending(res.pending);
    } catch (e: any) {
      setSyncResult({ error: String(e) });
    }
  };

  const saveTimeout = async () => {
    const m = parseInt(timeoutInput, 10);
    if (isNaN(m) || m < 1 || m > 480) return;
    setSaving(true);
    await api.auth.setSessionTimeout(m);
    setSessionMinutes(m);
    setSaving(false);
  };

  const handleCreateTag = async () => {
    if (!newTagName.trim()) return;
    tagCreate.mutate({ name: newTagName.trim(), color: newTagColor });
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
          {isUnlocked && (
            <div className="space-y-3">
              <div className="flex gap-2">
                <input
                  type="url"
                  value={syncUrl}
                  onChange={(e) => setSyncUrl(e.target.value)}
                  placeholder="https://your-project.supabase.co"
                  className="flex-1 p-2 text-sm border rounded-md bg-background"
                />
              </div>
              <div className="flex gap-2">
                <input
                  type="password"
                  value={syncKey}
                  onChange={(e) => setSyncKey(e.target.value)}
                  placeholder="Supabase anon key"
                  className="flex-1 p-2 text-sm border rounded-md bg-background"
                />
                <button
                  onClick={configuring ? undefined : handleConfigureSync}
                  disabled={configuring || !syncUrl || !syncKey}
                  className="bg-primary text-primary-foreground px-3 py-2 rounded-md text-xs font-medium hover:bg-primary/90 disabled:opacity-50"
                >
                  {configuring ? "Testing..." : "Save"}
                </button>
              </div>
              {syncMessage && (
                <p className={`text-xs ${syncMessage.includes("Error") || syncMessage.includes("Failed") ? "text-destructive" : "text-green-600"}`}>
                  {syncMessage}
                </p>
              )}
              <div className="flex items-center gap-2 text-sm">
                <span className="text-muted-foreground">
                  {syncConfigured ? "✓ Configured" : "Not configured"}
                </span>
              </div>
              <div className="flex gap-2">
                <button
                  onClick={syncing ? undefined : handleSyncPush}
                  disabled={syncing || !syncConfigured}
                  className="bg-primary text-primary-foreground px-3 py-1.5 rounded-md text-xs font-medium hover:bg-primary/90 disabled:opacity-50"
                >
                  {syncing ? "Syncing..." : `Push Changes (${syncPending} pending)`}
                </button>
                <button
                  onClick={pulling ? undefined : handleSyncPull}
                  disabled={pulling || !syncConfigured}
                  className="border px-3 py-1.5 rounded-md text-xs font-medium hover:bg-accent disabled:opacity-50"
                >
                  {pulling ? "Pulling..." : "Pull Changes"}
                </button>
                <button
                  onClick={handleSyncStatus}
                  className="border px-3 py-1.5 rounded-md text-xs font-medium hover:bg-accent"
                >
                  Refresh Status
                </button>
              </div>
              {syncResult && (
                <pre className="text-xs text-muted-foreground bg-muted/30 p-2 rounded overflow-x-auto">
                  {JSON.stringify(syncResult, null, 2)}
                </pre>
              )}
            </div>
          )}
          {!isUnlocked && (
            <p className="text-sm text-muted-foreground">
              <a href="/unlock" className="text-primary hover:underline">Unlock</a> the database to configure sync.
            </p>
          )}
        </section>

        <section className="rounded-lg border bg-card p-6 space-y-4">
          <h2 className="text-lg font-semibold">Export / Import</h2>
          <p className="text-sm text-muted-foreground">
            Export all your data as JSON, or import from a previously exported file.
            Exported data is decrypted and contains all notes, todos, events, notebooks, and tags.
          </p>
          <div className="flex flex-wrap gap-3">
            <button
              disabled={exporting || importing}
              onClick={async () => {
                setExporting(true);
                try {
                  const json = await api.data.exportData();
                  const blob = new Blob([json], { type: "application/json" });
                  const url = URL.createObjectURL(blob);
                  const a = document.createElement("a");
                  a.href = url;
                  a.download = `knowledge-base-export-${Date.now()}.json`;
                  a.click();
                  URL.revokeObjectURL(url);
                } catch (e: any) {
                  alert(`Export failed: ${e}`);
                }
                setExporting(false);
              }}
              className="bg-primary text-primary-foreground px-4 py-2 rounded-md text-sm font-medium hover:bg-primary/90 disabled:opacity-50"
            >
              {exporting ? "Exporting..." : "📥 Export All Data"}
            </button>
            <label className="border px-4 py-2 rounded-md text-sm font-medium hover:bg-accent cursor-pointer disabled:opacity-50">
              {importing ? "Importing..." : "📤 Import JSON"}
              <input
                type="file"
                accept=".json"
                className="hidden"
                disabled={exporting || importing}
                onChange={async (e) => {
                  const file = e.target.files?.[0];
                  if (!file) return;
                  const text = await file.text();
                  if (!confirm(`Import "${file.name}"? This will add new entries without overwriting existing ones.`)) {
                    e.target.value = "";
                    return;
                  }
                  setImporting(true);
                  try {
                    const res = await api.data.importData(text);
                    if (res.errors.length > 0) {
                      alert(`Import completed with ${res.errors.length} error(s):\n${res.errors.slice(0, 5).join("\n")}${res.errors.length > 5 ? `\n...and ${res.errors.length - 5} more` : ""}`);
                    } else {
                      alert(`Import successful!\n${res.notes} notes, ${res.todos} todos, ${res.events} events, ${res.notebooks} notebooks, ${res.tags} tags`);
                    }
                    queryClient.invalidateQueries({ queryKey: ["notes"] });
                    queryClient.invalidateQueries({ queryKey: ["todos"] });
                    queryClient.invalidateQueries({ queryKey: ["tags"] });
                    queryClient.invalidateQueries({ queryKey: ["events"] });
                    queryClient.invalidateQueries({ queryKey: ["notebooks"] });
                    queryClient.invalidateQueries({ queryKey: ["all-note-tags"] });
                    queryClient.invalidateQueries({ queryKey: ["all-todo-tags"] });
                  } catch (e: any) {
                    alert(`Import failed: ${e}`);
                  }
                  setImporting(false);
                  e.target.value = "";
                }}
              />
            </label>
          </div>
        </section>

        <section className="rounded-lg border bg-card p-6 space-y-4">
          <h2 className="text-lg font-semibold">Tags</h2>
          <p className="text-sm text-muted-foreground">Manage all tags used across notes and todos.</p>

          {/* Create new tag */}
          <div className="flex flex-col sm:flex-row items-stretch sm:items-center gap-2">
            <div className="flex-1 flex gap-2">
              <input
                value={newTagName}
                onChange={(e) => setNewTagName(e.target.value)}
                onKeyDown={(e) => e.key === "Enter" && handleCreateTag()}
                placeholder="New tag name..."
                className="flex-1 p-2 text-sm border rounded-md bg-background"
              />
              <input
                type="color"
                value={newTagColor}
                onChange={(e) => setNewTagColor(e.target.value)}
                className="w-9 h-9 p-0.5 border rounded cursor-pointer shrink-0"
                title="Tag color"
              />
            </div>
            <button
              onClick={handleCreateTag}
              disabled={!newTagName.trim() || tagCreate.isPending}
              className="bg-primary text-primary-foreground px-3 py-2 rounded-md text-xs font-medium hover:bg-primary/90 disabled:opacity-50"
            >
              {tagCreate.isPending ? "..." : "Add Tag"}
            </button>
          </div>

          {/* Tag list */}
          {allTags.length === 0 ? (
            <p className="text-sm text-muted-foreground">No tags yet. Create one above!</p>
          ) : (
            <div className="space-y-1.5">
              {allTags.map((tag) => {
                const isEditing = editTagId === tag.id;
                return (
                  <div key={tag.id} className="flex flex-col sm:flex-row items-start sm:items-center gap-2 rounded-md border p-2.5">
                    <div className="flex items-center gap-2 w-full sm:w-auto">
                      <div
                        className="w-3 h-3 rounded-full shrink-0"
                        style={{ backgroundColor: tag.color }}
                      />
                      {isEditing ? (
                        <input
                          value={editTagName}
                          onChange={(e) => setEditTagName(e.target.value)}
                          onKeyDown={(e) => e.key === "Enter" && tagUpdate.mutate({ id: tag.id, name: editTagName, color: editTagColor })}
                          className="flex-1 p-1 text-sm border rounded-md bg-background"
                        />
                      ) : (
                        <span className="flex-1 text-sm font-medium">{tag.name}</span>
                      )}
                      {!isEditing && <span className="text-xs text-muted-foreground">{tag.color}</span>}
                    </div>
                    {isEditing && (
                      <div className="flex items-center gap-2">
                        <input
                          type="color"
                          value={editTagColor}
                          onChange={(e) => setEditTagColor(e.target.value)}
                          className="w-8 h-8 p-0.5 border rounded cursor-pointer"
                        />
                      </div>
                    )}
                    <div className="flex gap-1.5 sm:ml-auto">
                      {isEditing ? (
                        <>
                          <button
                            onClick={() => tagUpdate.mutate({ id: tag.id, name: editTagName, color: editTagColor })}
                            disabled={tagUpdate.isPending}
                            className="text-xs px-2 py-1 rounded bg-primary text-primary-foreground hover:bg-primary/90 disabled:opacity-50"
                          >
                            Save
                          </button>
                          <button
                            onClick={() => setEditTagId(null)}
                            className="text-xs px-2 py-1 rounded border hover:bg-accent"
                          >
                            Cancel
                          </button>
                        </>
                      ) : (
                        <>
                          <button
                            onClick={() => { setEditTagId(tag.id); setEditTagName(tag.name); setEditTagColor(tag.color); }}
                            className="text-xs px-2 py-1 rounded border hover:bg-accent"
                          >
                            Edit
                          </button>
                          <button
                            onClick={() => {
                              if (confirm(`Delete tag "${tag.name}"? This will remove it from all notes and todos.`)) {
                                tagDelete.mutate(tag.id);
                              }
                            }}
                            disabled={tagDelete.isPending}
                            className="text-xs px-2 py-1 rounded border text-destructive hover:bg-destructive/10 disabled:opacity-50"
                          >
                            Delete
                          </button>
                        </>
                      )}
                    </div>
                  </div>
                );
              })}
            </div>
          )}
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
