import { describe, it, expect, vi, beforeEach } from "vitest";

vi.mock("@tauri-apps/api/core", () => ({
  invoke: vi.fn(),
}));

import { invoke } from "@tauri-apps/api/core";
import { api } from "@/lib/api";

const mockInvoke = vi.mocked(invoke);

beforeEach(() => {
  mockInvoke.mockReset();
});

describe("api.auth", () => {
  it("register calls invoke with correct args", async () => {
    mockInvoke.mockResolvedValue({ user: { id: "1" }, token: "tok" });
    const result = await api.auth.register("a@b.com", "pass", "Name");
    expect(mockInvoke).toHaveBeenCalledWith("register", { email: "a@b.com", password: "pass", name: "Name" });
    expect(result.user.id).toBe("1");
  });

  it("login calls invoke with email and password", async () => {
    mockInvoke.mockResolvedValue({ user: { id: "2" }, token: "tok" });
    await api.auth.login("x@y.com", "pw");
    expect(mockInvoke).toHaveBeenCalledWith("login", { email: "x@y.com", password: "pw" });
  });

  it("unlock calls unlock_database", async () => {
    mockInvoke.mockResolvedValue({ success: true });
    const result = await api.auth.unlock("secret");
    expect(mockInvoke).toHaveBeenCalledWith("unlock_database", { password: "secret" });
    expect(result.success).toBe(true);
  });

  it("logout calls invoke", async () => {
    mockInvoke.mockResolvedValue(undefined);
    await api.auth.logout();
    expect(mockInvoke).toHaveBeenCalledWith("logout");
  });
});

describe("api.notes", () => {
  it("list calls list_notes", async () => {
    mockInvoke.mockResolvedValue([]);
    const result = await api.notes.list();
    expect(mockInvoke).toHaveBeenCalledWith("list_notes");
    expect(result).toEqual([]);
  });

  it("get calls get_note with id", async () => {
    mockInvoke.mockResolvedValue({ id: "n1" });
    await api.notes.get("n1");
    expect(mockInvoke).toHaveBeenCalledWith("get_note", { id: "n1" });
  });

  it("create calls create_note with data", async () => {
    mockInvoke.mockResolvedValue({ id: "n2" });
    await api.notes.create({ title: "T", content: "C" });
    expect(mockInvoke).toHaveBeenCalledWith("create_note", { title: "T", content: "C" });
  });

  it("update calls update_note with id and data", async () => {
    mockInvoke.mockResolvedValue({ id: "n1" });
    await api.notes.update("n1", { title: "New" });
    expect(mockInvoke).toHaveBeenCalledWith("update_note", { id: "n1", title: "New" });
  });

  it("delete calls delete_note with id", async () => {
    mockInvoke.mockResolvedValue(undefined);
    await api.notes.delete("n1");
    expect(mockInvoke).toHaveBeenCalledWith("delete_note", { id: "n1" });
  });

  it("search calls search_notes with query", async () => {
    mockInvoke.mockResolvedValue([]);
    await api.notes.search("hello");
    expect(mockInvoke).toHaveBeenCalledWith("search_notes", { query: "hello" });
  });
});

describe("api.todos", () => {
  it("list calls list_todos", async () => {
    mockInvoke.mockResolvedValue([]);
    await api.todos.list();
    expect(mockInvoke).toHaveBeenCalledWith("list_todos");
  });

  it("create calls create_todo", async () => {
    mockInvoke.mockResolvedValue({ id: "t1" });
    await api.todos.create({ title: "Buy milk", priority: "high" });
    expect(mockInvoke).toHaveBeenCalledWith("create_todo", { title: "Buy milk", priority: "high" });
  });

  it("update calls update_todo", async () => {
    mockInvoke.mockResolvedValue({ id: "t1" });
    await api.todos.update("t1", { is_completed: true });
    expect(mockInvoke).toHaveBeenCalledWith("update_todo", { id: "t1", is_completed: true });
  });

  it("delete calls delete_todo", async () => {
    mockInvoke.mockResolvedValue(undefined);
    await api.todos.delete("t1");
    expect(mockInvoke).toHaveBeenCalledWith("delete_todo", { id: "t1" });
  });
});

describe("api.calendar", () => {
  it("list calls list_calendar_events", async () => {
    mockInvoke.mockResolvedValue([]);
    await api.calendar.list();
    expect(mockInvoke).toHaveBeenCalledWith("list_calendar_events");
  });

  it("create calls create_calendar_event", async () => {
    mockInvoke.mockResolvedValue({ id: "e1" });
    await api.calendar.create({ title: "Meeting", start_time: 100, end_time: 200 });
    expect(mockInvoke).toHaveBeenCalledWith("create_calendar_event", {
      title: "Meeting",
      start_time: 100,
      end_time: 200,
    });
  });

  it("update calls update_calendar_event", async () => {
    mockInvoke.mockResolvedValue({ id: "e1" });
    await api.calendar.update("e1", { title: "Rescheduled" });
    expect(mockInvoke).toHaveBeenCalledWith("update_calendar_event", { id: "e1", title: "Rescheduled" });
  });

  it("delete calls delete_calendar_event", async () => {
    mockInvoke.mockResolvedValue(undefined);
    await api.calendar.delete("e1");
    expect(mockInvoke).toHaveBeenCalledWith("delete_calendar_event", { id: "e1" });
  });
});

describe("api.encryption", () => {
  it("setMasterPassword calls set_master_password", async () => {
    mockInvoke.mockResolvedValue({ salt: "abc123" });
    const result = await api.encryption.setMasterPassword("pass");
    expect(mockInvoke).toHaveBeenCalledWith("set_master_password", { password: "pass" });
    expect(result.salt).toBe("abc123");
  });

  it("getSalt calls get_encryption_salt", async () => {
    mockInvoke.mockResolvedValue("salt-value");
    const result = await api.encryption.getSalt();
    expect(mockInvoke).toHaveBeenCalledWith("get_encryption_salt");
    expect(result).toBe("salt-value");
  });
});

describe("api.sync", () => {
  it("push calls sync_push", async () => {
    mockInvoke.mockResolvedValue({ synced: 5 });
    const result = await api.sync.push();
    expect(mockInvoke).toHaveBeenCalledWith("sync_push");
    expect(result.synced).toBe(5);
  });

  it("pull calls sync_pull", async () => {
    mockInvoke.mockResolvedValue({ synced: 3 });
    const result = await api.sync.pull();
    expect(mockInvoke).toHaveBeenCalledWith("sync_pull");
    expect(result.synced).toBe(3);
  });

  it("status calls sync_status", async () => {
    mockInvoke.mockResolvedValue({ pending: 2, last_sync: 1700000000 });
    const result = await api.sync.status();
    expect(mockInvoke).toHaveBeenCalledWith("sync_status");
    expect(result.pending).toBe(2);
  });
});
