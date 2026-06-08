"use client";
import { useQuery, useMutation, useQueryClient } from "@tanstack/react-query";
import { api } from "@/lib/api";

export function useNotebooks() {
  const queryClient = useQueryClient();
  const { data: notebooks = [], isLoading } = useQuery({ queryKey: ["notebooks"], queryFn: api.notebooks.list });
  const create = useMutation({
    mutationFn: api.notebooks.create,
    onSuccess: () => queryClient.invalidateQueries({ queryKey: ["notebooks"] }),
  });
  const update = useMutation({
    mutationFn: ({ id, ...data }: { id: string } & Record<string, any>) => api.notebooks.update(id, data),
    onSuccess: () => queryClient.invalidateQueries({ queryKey: ["notebooks"] }),
  });
  const remove = useMutation({
    mutationFn: api.notebooks.delete,
    onSuccess: () => queryClient.invalidateQueries({ queryKey: ["notebooks"] }),
  });
  return { notebooks, isLoading, create, update, remove };
}
