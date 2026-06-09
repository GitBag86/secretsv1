"use client";
import { useQuery, useMutation, useQueryClient } from "@tanstack/react-query";
import { api } from "@/lib/api";

export function useTags() {
  const queryClient = useQueryClient();
  const { data: tags = [], isLoading } = useQuery({ queryKey: ["tags"], queryFn: api.tags.list });
  const create = useMutation({
    mutationFn: api.tags.create,
    onSuccess: () => queryClient.invalidateQueries({ queryKey: ["tags"] }),
  });
  const update = useMutation({
    mutationFn: ({ id, ...data }: { id: string } & Record<string, any>) => api.tags.update(id, data),
    onSuccess: () => queryClient.invalidateQueries({ queryKey: ["tags"] }),
  });
  const remove = useMutation({
    mutationFn: api.tags.delete,
    onSuccess: () => {
      queryClient.invalidateQueries({ queryKey: ["tags"] });
      queryClient.invalidateQueries({ queryKey: ["note-tags"] });
    },
  });
  return { tags, isLoading, create, update, remove };
}

export function useNoteTags(noteId: string | null) {
  const queryClient = useQueryClient();
  const { data: noteTags = [], isLoading } = useQuery({
    queryKey: ["note-tags", noteId],
    queryFn: () => api.tags.getNoteTags(noteId!),
    enabled: !!noteId,
  });
  const setNoteTags = useMutation({
    mutationFn: (tagIds: string[]) => api.tags.setNoteTags(noteId!, tagIds),
    onSuccess: () => {
      queryClient.invalidateQueries({ queryKey: ["note-tags", noteId] });
      queryClient.invalidateQueries({ queryKey: ["tags"] });
      queryClient.invalidateQueries({ queryKey: ["all-note-tags"] });
    },
  });
  return { noteTags, isLoading, setNoteTags };
}
