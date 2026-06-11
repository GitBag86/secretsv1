"use client";
import { useState, useEffect } from "react";
import { useAuth } from "@/hooks";
import { api } from "@/lib/api";
import { useRouter } from "next/navigation";

const strengthLabels = ["Weak", "Fair", "Good", "Strong"];
const strengthColors = ["bg-red-500", "bg-yellow-500", "bg-green-400", "bg-green-600"];

function getStrength(pw: string): number {
  let s = 0;
  if (pw.length >= 8) s++;
  if (pw.length >= 12) s++;
  if (/[A-Z]/.test(pw) && /[a-z]/.test(pw)) s++;
  if (/\d/.test(pw) && /[^A-Za-z0-9]/.test(pw)) s++;
  return Math.min(s, 3);
}

export default function RegisterPage() {
  const [email, setEmail] = useState("");
  const [name, setName] = useState("");
  const [password, setPassword] = useState("");
  const [confirm, setConfirm] = useState("");
  const [error, setError] = useState("");
  const [loading, setLoading] = useState(false);
  const { setUser, setLoggedIn, isLoggedIn, isUnlocked, isHydrated } = useAuth();
  const router = useRouter();

  // Already logged in? Redirect accordingly
  useEffect(() => {
    if (!isHydrated) return;
    if (isLoggedIn && isUnlocked) router.push("/");
    else if (isLoggedIn && !isUnlocked) router.push("/unlock");
  }, [isHydrated, isLoggedIn, isUnlocked, router]);

  const strength = getStrength(password);

  const handleSubmit = async (e: React.FormEvent) => {
    e.preventDefault();
    setError("");
    if (password !== confirm) { setError("Passwords do not match"); return; }
    if (password.length < 8) { setError("Password must be at least 8 characters"); return; }
    setLoading(true);
    try {
      const res = await api.auth.register(email, password, name || undefined);
      setUser(res.user);
      setLoggedIn(true);
      router.push("/unlock");
    } catch (err: any) {
      setError(typeof err === "string" ? err : "Registration failed");
    } finally {
      setLoading(false);
    }
  };

  return (
    <div className="min-h-screen flex items-center justify-center bg-background">
      <div className="w-full max-w-sm space-y-6 p-8">
        <div className="text-center space-y-2">
          <h1 className="text-2xl font-bold">Create account</h1>
          <p className="text-muted-foreground text-sm">Start building your knowledge base</p>
        </div>
        <form onSubmit={handleSubmit} className="space-y-4">
          <div className="space-y-2">
            <label className="text-sm font-medium">Name (optional)</label>
            <input type="text" value={name} onChange={(e) => setName(e.target.value)} placeholder="Your name" className="w-full p-2 border rounded-md bg-background" />
          </div>
          <div className="space-y-2">
            <label className="text-sm font-medium">Email</label>
            <input type="email" value={email} onChange={(e) => setEmail(e.target.value)} required placeholder="you@example.com" className="w-full p-2 border rounded-md bg-background" />
          </div>
          <div className="space-y-2">
            <label className="text-sm font-medium">Password</label>
            <input type="password" value={password} onChange={(e) => setPassword(e.target.value)} required minLength={8} placeholder="••••••••" className="w-full p-2 border rounded-md bg-background" />
            {password && (
              <div className="space-y-1">
                <div className="h-1.5 w-full bg-muted rounded-full overflow-hidden">
                  <div className={`h-full ${strengthColors[strength]} transition-all`} style={{ width: `${((strength + 1) / 4) * 100}%` }} />
                </div>
                <p className="text-xs text-muted-foreground">{strengthLabels[strength]}</p>
              </div>
            )}
          </div>
          <div className="space-y-2">
            <label className="text-sm font-medium">Confirm password</label>
            <input type="password" value={confirm} onChange={(e) => setConfirm(e.target.value)} required minLength={8} placeholder="••••••••" className="w-full p-2 border rounded-md bg-background" />
          </div>
          {error && <p className="text-destructive text-sm">{error}</p>}
          <button type="submit" disabled={loading} className="w-full bg-primary text-primary-foreground py-2 rounded-md text-sm font-medium hover:bg-primary/90 disabled:opacity-50">
            {loading ? "Creating..." : "Create account"}
          </button>
        </form>
        <p className="text-center text-sm text-muted-foreground">
          Already have an account? <a href="/login" className="text-primary hover:underline">Sign in</a>
        </p>
      </div>
    </div>
  );
}
