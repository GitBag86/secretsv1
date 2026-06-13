"use client";
import { useEditor, EditorContent } from "@tiptap/react";
import StarterKit from "@tiptap/starter-kit";
import Placeholder from "@tiptap/extension-placeholder";
import TaskList from "@tiptap/extension-task-list";
import TaskItem from "@tiptap/extension-task-item";
import Link from "@tiptap/extension-link";
import Image from "@tiptap/extension-image";
import { useEffect, useRef, useState, useCallback } from "react";

interface RichTextEditorProps {
  content: string;
  onChange: (html: string) => void;
  placeholder?: string;
  editable?: boolean;
  minHeight?: string;
  maxHeight?: string;
  showResizeHandle?: boolean;
  onImageUpload?: (file: File) => Promise<string>;
}

export function RichTextEditor({
  content,
  onChange,
  placeholder = "Write something...",
  editable = true,
  minHeight = "200px",
  maxHeight,
  showResizeHandle = false,
  onImageUpload,
}: RichTextEditorProps) {
  const editor = useEditor({
    extensions: [
      StarterKit,
      Placeholder.configure({ placeholder }),
      TaskList,
      TaskItem.configure({ nested: true }),
      Link.configure({ openOnClick: false }),
      Image.configure({ inline: false }),
    ],
    content,
    editable,
    onUpdate: ({ editor }) => onChange(editor.getHTML()),
  });

  const containerRef = useRef<HTMLDivElement>(null);
  const [height, setHeight] = useState(minHeight);
  const [isResizing, setIsResizing] = useState(false);
  const startYRef = useRef(0);
  const startHeightRef = useRef(0);

  useEffect(() => {
    if (editor && content && !editor.isDestroyed) {
      const currentHtml = editor.getHTML();
      if (currentHtml !== content) {
        editor.commands.setContent(content, { emitUpdate: false });
      }
    }
  }, [editor, content]);

  useEffect(() => {
    if (!showResizeHandle || !editable) return;

    const handleMouseMove = (e: MouseEvent) => {
      if (!isResizing) return;
      const delta = e.clientY - startYRef.current;
      const newHeight = Math.max(
        parseInt(minHeight),
        Math.min(
          maxHeight ? parseInt(maxHeight) : 2000,
          startHeightRef.current + delta
        )
      );
      setHeight(`${newHeight}px`);
    };

    const handleMouseUp = () => setIsResizing(false);

    document.addEventListener("mousemove", handleMouseMove);
    document.addEventListener("mouseup", handleMouseUp);
    return () => {
      document.removeEventListener("mousemove", handleMouseMove);
      document.removeEventListener("mouseup", handleMouseUp);
    };
  }, [isResizing, minHeight, maxHeight, showResizeHandle, editable]);

  const handleResizeStart = useCallback((e: React.MouseEvent) => {
    e.preventDefault();
    setIsResizing(true);
    startYRef.current = e.clientY;
    const currentHeight = parseInt(height);
    startHeightRef.current = isNaN(currentHeight) ? parseInt(minHeight) : currentHeight;
  }, [height, minHeight]);

  const setLink = useCallback(() => {
    if (!editor) return;
    const url = window.prompt("Enter URL", "");
    if (url === null) return;
    if (url === "") {
      editor.chain().focus().unsetLink().run();
      return;
    }
    editor.chain().focus().setLink({ href: url }).run();
  }, [editor]);

  const imageInputRef = useRef<HTMLInputElement>(null);
  const handleImageUpload = useCallback(async (e: React.ChangeEvent<HTMLInputElement>) => {
    const file = e.target.files?.[0];
    if (!file || !editor || !onImageUpload) return;
    try {
      const url = await onImageUpload(file);
      editor.chain().focus().setImage({ src: url }).run();
    } catch (err) {
      console.error("Image upload failed:", err);
    }
    if (e.target) e.target.value = "";
  }, [editor, onImageUpload]);

  if (!editor) return null;

  return (
    <div
      ref={containerRef}
      className="border rounded-md bg-background"
      style={{ minHeight: height, maxHeight: maxHeight }}
    >
      <div className="flex flex-wrap gap-1 p-2 border-b bg-muted/30">
        <ToolBtn onClick={() => editor.chain().focus().undo().run()} active={false} label="↶" disabled={!editor.can().undo()} />
        <ToolBtn onClick={() => editor.chain().focus().redo().run()} active={false} label="↷" disabled={!editor.can().redo()} />
        <span className="w-px bg-border mx-1" />
        <ToolBtn onClick={() => editor.chain().focus().toggleBold().run()} active={editor.isActive("bold")} label="B" style={{ fontWeight: "bold" }} />
        <ToolBtn onClick={() => editor.chain().focus().toggleItalic().run()} active={editor.isActive("italic")} label="I" style={{ fontStyle: "italic" }} />
        <ToolBtn onClick={() => editor.chain().focus().toggleStrike().run()} active={editor.isActive("strike")} label="S" style={{ textDecoration: "line-through" }} />
        <ToolBtn onClick={() => editor.chain().focus().toggleCode().run()} active={editor.isActive("code")} label="<>" />
        <span className="w-px bg-border mx-1" />
        <ToolBtn onClick={() => editor.chain().focus().toggleHeading({ level: 1 }).run()} active={editor.isActive("heading", { level: 1 })} label="H1" />
        <ToolBtn onClick={() => editor.chain().focus().toggleHeading({ level: 2 }).run()} active={editor.isActive("heading", { level: 2 })} label="H2" />
        <ToolBtn onClick={() => editor.chain().focus().toggleHeading({ level: 3 }).run()} active={editor.isActive("heading", { level: 3 })} label="H3" />
        <span className="w-px bg-border mx-1" />
        <ToolBtn onClick={() => editor.chain().focus().toggleBulletList().run()} active={editor.isActive("bulletList")} label="•" />
        <ToolBtn onClick={() => editor.chain().focus().toggleOrderedList().run()} active={editor.isActive("orderedList")} label="1." />
        <ToolBtn onClick={() => editor.chain().focus().toggleTaskList().run()} active={editor.isActive("taskList")} label="☑" />
        <ToolBtn onClick={() => editor.chain().focus().toggleBlockquote().run()} active={editor.isActive("blockquote")} label="❝" />
        <ToolBtn onClick={() => editor.chain().focus().toggleCodeBlock().run()} active={editor.isActive("codeBlock")} label="{}" />
        {onImageUpload && (
          <>
            <span className="w-px bg-border mx-1" />
            <input
              ref={imageInputRef}
              type="file"
              accept="image/*"
              onChange={handleImageUpload}
              className="hidden"
            />
            <ToolBtn onClick={() => imageInputRef.current?.click()} active={false} label="🖼️" />
          </>
        )}
        <ToolBtn onClick={setLink} active={editor.isActive("link")} label="🔗" />
      </div>
      <EditorContent editor={editor} className="prose prose-sm dark:prose-invert max-w-none p-3 [&_.ProseMirror]:outline-none [&_.ProseMirror]:min-h-[150px] overflow-y-auto" style={{ height: `calc(${height} - 42px)` }} />
      {showResizeHandle && editable && (
        <div
          className="h-2 bg-border hover:bg-primary/50 cursor-ns-resize transition-colors"
          onMouseDown={handleResizeStart}
        />
      )}
    </div>
  );
}

function ToolBtn({ onClick, active, label, style, disabled }: { onClick: () => void; active: boolean; label: string; style?: React.CSSProperties; disabled?: boolean }) {
  return (
    <button
      type="button"
      onClick={onClick}
      disabled={disabled}
      style={style}
      className={`px-2 py-1 text-xs rounded ${active ? "bg-primary text-primary-foreground" : "hover:bg-accent"} disabled:opacity-50 disabled:cursor-not-allowed`}
    >
      {label}
    </button>
  );
}