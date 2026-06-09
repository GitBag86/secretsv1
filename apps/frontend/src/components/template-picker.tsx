"use client";
import { useState, useRef, useEffect } from "react";
import { useQuery, useMutation, useQueryClient } from "@tanstack/react-query";
import { api } from "@/lib/api";
import type { Note } from "@/types";

export function TemplatePicker({
  onNoteCreated,
}: {
  onNoteCreated: (note: Note) => void;
}) {
  const [open, setOpen] = useState(false);
  const [title, setTitle] = useState("");
  const ref = useRef<HTMLDivElement>(null);
  const queryClient = useQueryClient();

  const { data: templates = [] } = useQuery({
    queryKey: ["templates"],
    queryFn: () => api.templates.list(),
  });

  const createFromTemplate = useMutation({
    mutationFn: ({ templateId, title: noteTitle }: { templateId: string; title?: string }) =>
      api.templates.createNoteFrom(templateId, noteTitle),
    onSuccess: (note) => {
      queryClient.invalidateQueries({ queryKey: ["notes"] });
      setOpen(false);
      setTitle("");
      onNoteCreated(note);
    },
  });

  useEffect(() => {
    const handleClickOutside = (e: MouseEvent) => {
      if (ref.current && !ref.current.contains(e.target as Node)) {
        setOpen(false);
      }
    };
    document.addEventListener("mousedown", handleClickOutside);
    return () => document.removeEventListener("mousedown", handleClickOutside);
  }, []);

  if (templates.length === 0) return null;

  return (
    <div ref={ref} className="relative">
      <button
        onClick={() => setOpen(!open)}
        className="border px-3 py-2 rounded-md text-sm font-medium hover:bg-accent"
      >
        📋 From Template
      </button>
      {open && (
        <div className="absolute right-0 top-full mt-1 z-40 w-64 rounded-lg border bg-card shadow-xl overflow-hidden">
          <div className="p-2 border-b">
            <input
              value={title}
              onChange={(e) => setTitle(e.target.value)}
              placeholder="Note title (optional)..."
              className="w-full p-1.5 text-xs border rounded-md bg-background"
            />
          </div>
          <div className="max-h-48 overflow-y-auto p-1">
            {templates.map((tmpl) => (
              <button
                key={tmpl.id}
                onClick={() => createFromTemplate.mutate({ templateId: tmpl.id, title: title || undefined })}
                disabled={createFromTemplate.isPending}
                className="w-full text-left p-2 rounded-md text-sm hover:bg-accent disabled:opacity-50"
              >
                <span className="font-medium">{tmpl.name}</span>
              </button>
            ))}
          </div>
        </div>
      )}
    </div>
  );
}
