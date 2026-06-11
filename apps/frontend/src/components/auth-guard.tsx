"use client";
import { useEffect } from "react";
import { useAuth } from "@/hooks";
import { useRouter } from "next/navigation";

export function AuthGuard({ children }: { children: React.ReactNode }) {
  const { isHydrated, isLoggedIn, isUnlocked } = useAuth();
  const router = useRouter();

  useEffect(() => {
    if (!isHydrated) return;
    if (!isLoggedIn) {
      router.replace("/login");
    } else if (!isUnlocked) {
      router.replace("/unlock");
    }
  }, [isHydrated, isLoggedIn, isUnlocked, router]);

  if (!isHydrated || !isLoggedIn || !isUnlocked) {
    return (
      <div className="min-h-screen flex items-center justify-center bg-background">
        <div className="animate-spin h-8 w-8 border-4 border-primary border-t-transparent rounded-full" />
      </div>
    );
  }

  return <>{children}</>;
}
