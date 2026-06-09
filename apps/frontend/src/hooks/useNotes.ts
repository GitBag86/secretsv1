"use client";
import { useQuery, useMutation, useQueryClient } from "@tanstack/react-query";
import { api } from "@/lib/api";
import { useUndoStack } from "./useUndoStack";

export function useNotes() {
  const queryClient = useQueryClient();
  const push = useUndoStack((s) => s.push);
  const { data: notes = [], isLoading } = useQuery({ queryKey: ["notes"], queryFn: api.notes.list });
  const create = useMutation({
    mutationFn: api.notes.create,
    onSuccess: (created) => {
      queryClient.invalidateQueries({ queryKey: ["notes"] });
      push({ id: created.id, type: "note", action: "create", before: null, after: created, label: "Create note" });
    },
  });
  const update = useMutation({
    mutationFn: ({ id, ...data }: { id: string } & Record<string, any>) => api.notes.update(id, data),
    onSuccess: (_result, vars) => {
      queryClient.invalidateQueries({ queryKey: ["notes"] });
      const cached = queryClient.getQueryData(["notes"]);
      const before = Array.isArray(cached) ? cached.find((n: any) => n.id === vars.id) || null : null;
      push({ id: vars.id, type: "note", action: "update", before, after: _result, label: "Update note" });
    },
  });
  const remove = useMutation({
    mutationFn: api.notes.delete,
    onSuccess: (_result, id) => {
      queryClient.invalidateQueries({ queryKey: ["notes"] });
      const cached = queryClient.getQueryData(["notes"]);
      const before = Array.isArray(cached) ? cached.find((n: any) => n.id === id) || null : null;
      push({ id, type: "note", action: "delete", before, after: null, label: "Delete note" });
    },
  });
  return { notes, isLoading, create, update, remove };
}
