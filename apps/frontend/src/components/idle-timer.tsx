"use client";
import { useEffect, useRef } from "react";
import { useAuth } from "@/hooks";
import { api } from "@/lib/api";

export function IdleTimer() {
  const { isUnlocked, sessionMinutes, lock, setSessionElapsed } = useAuth();
  const lastActivity = useRef(Date.now());
  const checkInterval = useRef<ReturnType<typeof setInterval> | null>(null);
  const tickInterval = useRef<ReturnType<typeof setInterval> | null>(null);

  useEffect(() => {
    if (!isUnlocked) {
      if (checkInterval.current) clearInterval(checkInterval.current);
      if (tickInterval.current) clearInterval(tickInterval.current);
      return;
    }

    const reset = () => { lastActivity.current = Date.now(); };
    window.addEventListener("mousemove", reset, { passive: true });
    window.addEventListener("keydown", reset, { passive: true });
    window.addEventListener("mousedown", reset, { passive: true });
    window.addEventListener("touchstart", reset, { passive: true });
    window.addEventListener("scroll", reset, { passive: true });

    checkInterval.current = setInterval(async () => {
      const idleMs = Date.now() - lastActivity.current;
      if (idleMs >= sessionMinutes * 60 * 1000) {
        await lock();
      } else {
        await api.auth.refreshSession();
      }
    }, 30_000);

    tickInterval.current = setInterval(async () => {
      try {
        const session = await api.auth.checkSession();
        if (!session.valid) {
          await lock();
        } else {
          setSessionElapsed(session.elapsed_seconds || 0);
        }
      } catch {
        // ignore
      }
    }, 5_000);

    return () => {
      window.removeEventListener("mousemove", reset);
      window.removeEventListener("keydown", reset);
      window.removeEventListener("mousedown", reset);
      window.removeEventListener("touchstart", reset);
      window.removeEventListener("scroll", reset);
      if (checkInterval.current) clearInterval(checkInterval.current);
      if (tickInterval.current) clearInterval(tickInterval.current);
    };
  }, [isUnlocked, sessionMinutes, lock, setSessionElapsed]);

  return null;
}
