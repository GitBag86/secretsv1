"use client";
import { AuthGuard } from "@/components/auth-guard";
import { useCalendar } from "@/hooks";
import { NavHeader } from "@/components/nav-header";
import { useState, useRef, useEffect, useMemo } from "react";
import FullCalendar from "@fullcalendar/react";
import dayGridPlugin from "@fullcalendar/daygrid";
import timeGridPlugin from "@fullcalendar/timegrid";
import interactionPlugin from "@fullcalendar/interaction";
import rrulePlugin from "@fullcalendar/rrule";
import type { EventClickArg, DateSelectArg, EventDropArg } from "@fullcalendar/core";

export default function CalendarPage() {
  const { events, isLoading, create, update, remove } = useCalendar();
  const calendarRef = useRef<FullCalendar>(null);
  const [showModal, setShowModal] = useState(false);
  const [editingId, setEditingId] = useState<string | null>(null);
  const [title, setTitle] = useState("");
  const [description, setDescription] = useState("");
  const [startStr, setStartStr] = useState("");
  const [endStr, setEndStr] = useState("");
  const [allDay, setAllDay] = useState(false);
  const [color, setColor] = useState("#3b82f6");
  const [recurType, setRecurType] = useState("none");
  const [selectionStart, setSelectionStart] = useState<Date | null>(null);
  const [selectionEnd, setSelectionEnd] = useState<Date | null>(null);

  const calendarEvents = useMemo(() =>
    events.map((e) => ({
      id: e.id,
      title: e.title,
      start: new Date(e.start_time * 1000).toISOString(),
      end: new Date(e.end_time * 1000).toISOString(),
      allDay: e.all_day,
      backgroundColor: e.color,
      borderColor: e.color,
      textColor: "#fff",
      extendedProps: { description: e.description },
      ...(e.rrule ? { rrule: { freq: parseRrule(e.rrule), dtstart: new Date(e.start_time * 1000).toISOString() } } : {}),
    })),
    [events]
  );

  const openCreateModal = (start: Date, end: Date) => {
    setEditingId(null);
    setTitle("");
    setDescription("");
    setStartStr(start.toISOString().slice(0, 16));
    setEndStr(end.toISOString().slice(0, 16));
    setAllDay(false);
    setColor("#3b82f6");
    setRecurType("none");
    setSelectionStart(start);
    setSelectionEnd(end);
    setShowModal(true);
  };

  const openEditModal = (event: {
    id: string; title: string; start: Date; end: Date; allDay: boolean;
    backgroundColor: string; extendedProps: { description?: string };
  }) => {
    setEditingId(event.id);
    setTitle(event.title);
    setDescription(event.extendedProps.description || "");
    setStartStr(event.start.toISOString().slice(0, 16));
    setEndStr(event.end.toISOString().slice(0, 16));
    setAllDay(event.allDay);
    setColor(event.backgroundColor);
    setRecurType("none");
    setShowModal(true);
  };

  const handleSave = async () => {
    if (!title.trim()) return;
    const start = Math.floor(new Date(startStr).getTime() / 1000);
    const end = Math.floor(new Date(endStr).getTime() / 1000);
    const rrule = recurType === "none" ? undefined : freqToRrule(recurType, start);
    if (editingId) {
      await update.mutateAsync({ id: editingId, title, description, start_time: start, end_time: end, all_day: allDay, color, rrule });
    } else {
      await create.mutateAsync({ title, description, start_time: start, end_time: end, all_day: allDay, color, rrule });
    }
    setShowModal(false);
  };

  const handleSelect = (info: DateSelectArg) => {
    openCreateModal(info.start, info.end);
  };

  const handleEventClick = (info: EventClickArg) => {
    openEditModal({
      id: info.event.id,
      title: info.event.title,
      start: info.event.start!,
      end: info.event.end || info.event.start!,
      allDay: info.event.allDay,
      backgroundColor: info.event.backgroundColor || "#3b82f6",
      extendedProps: { description: info.event.extendedProps.description as string },
    });
  };

  const handleEventDrop = async (info: EventDropArg) => {
    const startSec = Math.floor(info.event.start!.getTime() / 1000);
    const endSec = Math.floor(info.event.end!.getTime() / 1000);
    try {
      await update.mutateAsync({ id: info.event.id, start_time: startSec, end_time: endSec });
    } catch {
      // Revert the drop on failure
      info.revert();
    }
  };

  const today = () => calendarRef.current?.getApi().today();

  return (
    <AuthGuard>
    <div className="min-h-screen bg-background">
      <NavHeader />
      <main className="container py-6 max-w-6xl mx-auto">
        <div className="flex items-center justify-between mb-4">
          <h1 className="text-3xl font-bold">Calendar</h1>
          <button onClick={today} className="border px-3 py-1.5 rounded-md text-sm font-medium hover:bg-accent">Today</button>
        </div>
        {isLoading ? (
          <p className="text-muted-foreground">Loading calendar...</p>
        ) : (
          <div className="rounded-lg border bg-card p-3 [&_.fc-toolbar-title]:text-lg [&_.fc-button]:!bg-primary [&_.fc-button]:!text-primary-foreground [&_.fc-button]:!border-0 dark:[&_.fc-daygrid-day]:!border-muted [&_.fc-timegrid-slot]:!border-muted [&_.fc-col-header-cell]:!border-muted [&_.fc-scrollgrid]:!border-muted [&_.fc-theme-standard]:!border-muted">
            <FullCalendar
              ref={calendarRef}
              plugins={[dayGridPlugin, timeGridPlugin, interactionPlugin, rrulePlugin]}
              initialView="dayGridMonth"
              headerToolbar={{ left: "prev,next", center: "title", right: "dayGridMonth,timeGridWeek,timeGridDay" }}
              events={calendarEvents}
              selectable
              select={handleSelect}
              eventClick={handleEventClick}
              eventDrop={handleEventDrop}
              editable
              dayMaxEvents
              weekNumbers
              nowIndicator
              height="auto"
            />
          </div>
        )}
        {showModal && (
          <div className="fixed inset-0 bg-black/50 flex items-center justify-center z-50" onClick={() => setShowModal(false)}>
            <div className="rounded-lg border bg-card p-6 w-full max-w-md shadow-lg m-4" onClick={(e) => e.stopPropagation()}>
              <h2 className="text-xl font-bold mb-4">{editingId ? "Edit Event" : "New Event"}</h2>
              <input value={title} onChange={(e) => setTitle(e.target.value)} placeholder="Event title..." className="w-full mb-3 p-2 border rounded-md bg-background" />
              <textarea value={description} onChange={(e) => setDescription(e.target.value)} placeholder="Description (optional)..." className="w-full mb-3 p-2 border rounded-md bg-background min-h-[60px]" />
              <div className="flex gap-2 mb-3">
                <div className="flex-1">
                  <label className="text-xs text-muted-foreground block mb-1">Start</label>
                  <input type="datetime-local" value={startStr} onChange={(e) => setStartStr(e.target.value)} className="w-full p-2 border rounded-md bg-background text-sm" />
                </div>
                <div className="flex-1">
                  <label className="text-xs text-muted-foreground block mb-1">End</label>
                  <input type="datetime-local" value={endStr} onChange={(e) => setEndStr(e.target.value)} className="w-full p-2 border rounded-md bg-background text-sm" />
                </div>
              </div>
              <div className="flex gap-3 mb-3">
                <label className="flex items-center gap-2 text-sm">
                  <input type="checkbox" checked={allDay} onChange={(e) => setAllDay(e.target.checked)} />
                  All day
                </label>
                <label className="flex items-center gap-2 text-sm">
                  Color:
                  <input type="color" value={color} onChange={(e) => setColor(e.target.value)} className="h-7 w-10 rounded border" />
                </label>
              </div>
              <div className="mb-4">
                <label className="text-xs text-muted-foreground block mb-1">Repeat</label>
                <select value={recurType} onChange={(e) => setRecurType(e.target.value)} className="w-full p-2 border rounded-md bg-background text-sm">
                  <option value="none">Does not repeat</option>
                  <option value="daily">Daily</option>
                  <option value="weekly">Weekly</option>
                  <option value="biweekly">Every 2 weeks</option>
                  <option value="monthly">Monthly</option>
                  <option value="yearly">Yearly</option>
                </select>
              </div>
              <div className="flex gap-2 justify-end">
                <button onClick={() => setShowModal(false)} className="border px-4 py-2 rounded-md text-sm font-medium hover:bg-accent">Cancel</button>
                {editingId && (
                  <button onClick={() => { remove.mutate(editingId); setShowModal(false); }} className="border border-destructive text-destructive px-4 py-2 rounded-md text-sm font-medium hover:bg-destructive/10">Delete</button>
                )}
                <button onClick={handleSave} disabled={create.isPending || update.isPending} className="bg-primary text-primary-foreground px-4 py-2 rounded-md text-sm font-medium hover:bg-primary/90">
                  Save
                </button>
              </div>
            </div>
          </div>
        )}
      </main>
    </div>
    </AuthGuard>
  );
}

function parseRrule(rrule: string): any {
  if (rrule.startsWith("FREQ=DAILY")) return "daily";
  if (rrule.startsWith("FREQ=WEEKLY;INTERVAL=2")) return "biweekly";
  if (rrule.startsWith("FREQ=WEEKLY")) return "weekly";
  if (rrule.startsWith("FREQ=MONTHLY")) return "monthly";
  if (rrule.startsWith("FREQ=YEARLY")) return "yearly";
  return "weekly";
}

function freqToRrule(recurType: string, startTime: number): string {
  const start = new Date(startTime);
  const map: Record<string, string> = {
    daily: "FREQ=DAILY",
    weekly: "FREQ=WEEKLY;BYDAY=" + ["SU", "MO", "TU", "WE", "TH", "FR", "SA"][start.getDay()],
    biweekly: "FREQ=WEEKLY;INTERVAL=2;BYDAY=" + ["SU", "MO", "TU", "WE", "TH", "FR", "SA"][start.getDay()],
    monthly: "FREQ=MONTHLY;BYMONTHDAY=" + start.getDate(),
    yearly: "FREQ=YEARLY;BYMONTH=" + (start.getMonth() + 1) + ";BYMONTHDAY=" + start.getDate(),
  };
  return map[recurType] || "FREQ=WEEKLY";
}
