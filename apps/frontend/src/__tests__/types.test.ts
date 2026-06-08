import { describe, it, expect } from "vitest";
import type { User, Note, Todo, CalendarEvent } from "@/types";

describe("Type interfaces", () => {
  it("User has required fields", () => {
    const user: User = {
      id: "1",
      email: "test@example.com",
      name: "Test",
      created_at: 1700000000,
      updated_at: 1700000000,
    };
    expect(user.id).toBe("1");
    expect(user.email).toContain("@");
  });

  it("User allows optional name", () => {
    const user: User = {
      id: "1",
      email: "a@b.com",
      created_at: 0,
      updated_at: 0,
    };
    expect(user.name).toBeUndefined();
  });

  it("Note has all fields", () => {
    const note: Note = {
      id: "n1",
      user_id: "u1",
      title: "Hello",
      content: "World",
      word_count: 1,
      reading_time: 1,
      is_pinned: false,
      is_archived: false,
      created_at: 1700000000,
      updated_at: 1700000000,
    };
    expect(note.title).toBe("Hello");
    expect(note.word_count).toBe(1);
  });

  it("Note allows optional notebook_id", () => {
    const note: Note = {
      id: "n1",
      user_id: "u1",
      title: "T",
      content: "C",
      word_count: 0,
      reading_time: 0,
      is_pinned: false,
      is_archived: false,
      created_at: 0,
      updated_at: 0,
    };
    expect(note.notebook_id).toBeUndefined();
  });

  it("Todo priority is constrained", () => {
    const todo: Todo = {
      id: "t1",
      user_id: "u1",
      title: "Do stuff",
      is_completed: false,
      priority: "high",
      created_at: 0,
      updated_at: 0,
    };
    expect(["low", "medium", "high"]).toContain(todo.priority);
  });

  it("CalendarEvent has time range", () => {
    const event: CalendarEvent = {
      id: "e1",
      user_id: "u1",
      title: "Meeting",
      start_time: 1700000000,
      end_time: 1700003600,
      all_day: false,
      color: "#3b82f6",
      created_at: 0,
      updated_at: 0,
    };
    expect(event.end_time).toBeGreaterThan(event.start_time);
  });
});
