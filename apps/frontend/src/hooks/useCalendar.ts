"use client";
import { useQuery, useMutation, useQueryClient } from "@tanstack/react-query";
import { api } from "@/lib/api";

export function useCalendar() {
  const queryClient = useQueryClient();
  const { data: events = [], isLoading } = useQuery({ queryKey: ["calendar"], queryFn: api.calendar.list });
  const create = useMutation({
    mutationFn: api.calendar.create,
    onSuccess: () => queryClient.invalidateQueries({ queryKey: ["calendar"] }),
  });
  const update = useMutation({
    mutationFn: ({ id, ...data }: { id: string } & Record<string, any>) => api.calendar.update(id, data),
    onSuccess: () => queryClient.invalidateQueries({ queryKey: ["calendar"] }),
  });
  const remove = useMutation({
    mutationFn: api.calendar.delete,
    onSuccess: () => queryClient.invalidateQueries({ queryKey: ["calendar"] }),
  });
  return { events, isLoading, create, update, remove };
}
