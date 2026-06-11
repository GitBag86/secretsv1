"use client";
import { useState, useEffect } from "react";
import { useAuth } from "@/hooks";
import { api } from "@/lib/api";
import { useRouter } from "next/navigation";

export default function LoginPage() {
  const [email, setEmail] = useState("");
  const [password, setPassword] = useState("");
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

  const handleSubmit = async (e: React.FormEvent) => {
    e.preventDefault();
    setError("");
    setLoading(true);
    try {
      const res = await api.auth.login(email, password);
      setUser(res.user);
      setLoggedIn(true);
      router.push("/unlock");
    } catch (err: any) {
      setError(typeof err === "string" ? err : "Login failed");
    } finally {
      setLoading(false);
    }
  };

  return (
    <div className="min-h-screen flex items-center justify-center bg-background">
      <div className="w-full max-w-sm space-y-6 p-8">
        <div className="text-center space-y-2">
          <h1 className="text-2xl font-bold">Welcome back</h1>
          <p className="text-muted-foreground text-sm">Sign in to your knowledge base</p>
        </div>
        <form onSubmit={handleSubmit} className="space-y-4">
          <div className="space-y-2">
            <label className="text-sm font-medium">Email</label>
            <input type="email" value={email} onChange={(e) => setEmail(e.target.value)} required placeholder="you@example.com" className="w-full p-2 border rounded-md bg-background" />
          </div>
          <div className="space-y-2">
            <label className="text-sm font-medium">Password</label>
            <input type="password" value={password} onChange={(e) => setPassword(e.target.value)} required placeholder="••••••••" className="w-full p-2 border rounded-md bg-background" />
          </div>
          {error && <p className="text-destructive text-sm">{error}</p>}
          <button type="submit" disabled={loading} className="w-full bg-primary text-primary-foreground py-2 rounded-md text-sm font-medium hover:bg-primary/90 disabled:opacity-50">
            {loading ? "Signing in..." : "Sign in"}
          </button>
        </form>
        <p className="text-center text-sm text-muted-foreground">
          Don't have an account? <a href="/register" className="text-primary hover:underline">Register</a>
        </p>
      </div>
    </div>
  );
}
