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

export default function UnlockPage() {
  const [password, setPassword] = useState("");
  const [confirmPassword, setConfirmPassword] = useState("");
  const [error, setError] = useState("");
  const [loading, setLoading] = useState(false);
  const [checking, setChecking] = useState(true);
  const [hasSalt, setHasSalt] = useState(false);
  const { unlock, setupMasterPassword, isUnlocked, isHydrated } = useAuth();
  const router = useRouter();

  // Already unlocked? Go to dashboard
  useEffect(() => {
    if (isHydrated && isUnlocked) {
      router.push("/");
    }
  }, [isHydrated, isUnlocked, router]);

  useEffect(() => {
    api.encryption.getSalt().then((salt) => {
      setHasSalt(salt !== null);
    }).catch(() => {
      setHasSalt(false);
    }).finally(() => {
      setChecking(false);
    });
  }, []);

  const strength = getStrength(password);

  const handleSetup = async (e: React.FormEvent) => {
    e.preventDefault();
    setError("");
    if (password !== confirmPassword) {
      setError("Passwords do not match");
      return;
    }
    if (password.length < 8) {
      setError("Password must be at least 8 characters");
      return;
    }
    setLoading(true);
    try {
      await setupMasterPassword(password);
      router.push("/");
    } catch (err: any) {
      setError(typeof err === "string" ? err : "Failed to set master password");
    } finally {
      setLoading(false);
    }
  };

  const handleUnlock = async (e: React.FormEvent) => {
    e.preventDefault();
    setError("");
    setLoading(true);
    try {
      await unlock(password);
      router.push("/");
    } catch (err: any) {
      setError(typeof err === "string" ? err : "Failed to unlock");
    } finally {
      setLoading(false);
    }
  };

  if (checking) {
    return (
      <div className="min-h-screen flex items-center justify-center bg-background">
        <div className="w-full max-w-sm text-center space-y-4 p-8">
          <div className="animate-spin h-8 w-8 border-4 border-primary border-t-transparent rounded-full mx-auto" />
          <p className="text-sm text-muted-foreground">Checking database...</p>
        </div>
      </div>
    );
  }

  if (hasSalt) {
    // Unlock flow — master password already set
    return (
      <div className="min-h-screen flex items-center justify-center bg-background">
        <div className="w-full max-w-sm space-y-6 p-8">
          <div className="text-center space-y-2">
            <h1 className="text-2xl font-bold">Unlock Database</h1>
            <p className="text-muted-foreground text-sm">
              Enter your master password to unlock the encrypted database
            </p>
          </div>
          <form onSubmit={handleUnlock} className="space-y-4">
            <div className="space-y-2">
              <label className="text-sm font-medium">Master password</label>
              <input
                type="password"
                value={password}
                onChange={(e) => setPassword(e.target.value)}
                required
                placeholder="......."
                className="w-full p-2 border rounded-md bg-background"
                autoFocus
              />
            </div>
            {error && <p className="text-destructive text-sm">{error}</p>}
            <button
              type="submit"
              disabled={loading}
              className="w-full bg-primary text-primary-foreground py-2 rounded-md text-sm font-medium hover:bg-primary/90 disabled:opacity-50"
            >
              {loading ? "Unlocking..." : "Unlock"}
            </button>
          </form>
        </div>
      </div>
    );
  }

  // Setup flow — no master password set yet
  return (
    <div className="min-h-screen flex items-center justify-center bg-background">
      <div className="w-full max-w-sm space-y-6 p-8">
        <div className="text-center space-y-2">
          <h1 className="text-2xl font-bold">Set Master Password</h1>
          <p className="text-muted-foreground text-sm">
            Choose a master password to encrypt your data. This password will be
            required each time you unlock the database.
          </p>
        </div>
        <form onSubmit={handleSetup} className="space-y-4">
          <div className="space-y-2">
            <label className="text-sm font-medium">Master password</label>
            <input
              type="password"
              value={password}
              onChange={(e) => setPassword(e.target.value)}
              required
              minLength={8}
              placeholder="••••••••"
              className="w-full p-2 border rounded-md bg-background"
              autoFocus
            />
            {password && (
              <div className="space-y-1">
                <div className="h-1.5 w-full bg-muted rounded-full overflow-hidden">
                  <div
                    className={`h-full ${strengthColors[strength]} transition-all`}
                    style={{ width: `${((strength + 1) / 4) * 100}%` }}
                  />
                </div>
                <p className="text-xs text-muted-foreground">{strengthLabels[strength]}</p>
              </div>
            )}
          </div>
          <div className="space-y-2">
            <label className="text-sm font-medium">Confirm password</label>
            <input
              type="password"
              value={confirmPassword}
              onChange={(e) => setConfirmPassword(e.target.value)}
              required
              minLength={8}
              placeholder="••••••••"
              className="w-full p-2 border rounded-md bg-background"
            />
            {confirmPassword && password !== confirmPassword && (
              <p className="text-xs text-destructive">Passwords do not match</p>
            )}
          </div>
          {error && <p className="text-destructive text-sm">{error}</p>}
          <button
            type="submit"
            disabled={loading || (password !== confirmPassword)}
            className="w-full bg-primary text-primary-foreground py-2 rounded-md text-sm font-medium hover:bg-primary/90 disabled:opacity-50"
          >
            {loading ? "Setting up..." : "Set Master Password"}
          </button>
        </form>
        <p className="text-center text-xs text-muted-foreground">
          Your master password is the only key to your encrypted data.
          Store it somewhere safe — it cannot be recovered if lost.
        </p>
      </div>
    </div>
  );
}
