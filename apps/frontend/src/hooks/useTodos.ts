"use client";
import { useQuery, useMutation, useQueryClient } from "@tanstack/react-query";
import { api } from "@/lib/api";
import { useUndoStack } from "./useUndoStack";

export function useTodos() {
  const queryClient = useQueryClient();
  const push = useUndoStack((s) => s.push);
  const { data: todos = [], isLoading } = useQuery({ queryKey: ["todos"], queryFn: api.todos.list });
  const create = useMutation({
    mutationFn: api.todos.create,
    onSuccess: (created) => {
      queryClient.invalidateQueries({ queryKey: ["todos"] });
      push({ id: created.id, type: "todo", action: "create", before: null, after: created, label: "Create todo" });
    },
  });
  const update = useMutation({
    mutationFn: ({ id, ...data }: { id: string } & Record<string, any>) => api.todos.update(id, data),
    onSuccess: (_result, vars) => {
      queryClient.invalidateQueries({ queryKey: ["todos"] });
      const cached = queryClient.getQueryData(["todos"]);
      const before = Array.isArray(cached) ? cached.find((t: any) => t.id === vars.id) || null : null;
      push({ id: vars.id, type: "todo", action: "update", before, after: _result, label: "Update todo" });
    },
  });
  const remove = useMutation({
    mutationFn: api.todos.delete,
    onSuccess: (_result, id) => {
      queryClient.invalidateQueries({ queryKey: ["todos"] });
      const cached = queryClient.getQueryData(["todos"]);
      const before = Array.isArray(cached) ? cached.find((t: any) => t.id === id) || null : null;
      push({ id, type: "todo", action: "delete", before, after: null, label: "Delete todo" });
    },
  });
  const bulkUpdate = useMutation({
    mutationFn: ({ ids, is_completed }: { ids: string[]; is_completed: boolean }) => api.todos.bulkUpdate(ids, is_completed),
    onSuccess: () => queryClient.invalidateQueries({ queryKey: ["todos"] }),
  });
  const bulkDelete = useMutation({
    mutationFn: (ids: string[]) => api.todos.bulkDelete(ids),
    onSuccess: () => queryClient.invalidateQueries({ queryKey: ["todos"] }),
  });

  const refresh = () => queryClient.invalidateQueries({ queryKey: ["todos"] });

  return { todos, isLoading, create, update, remove, bulkUpdate, bulkDelete, refresh };
}
