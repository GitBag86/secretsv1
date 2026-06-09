"use client";
import { useEditor, EditorContent } from "@tiptap/react";
import StarterKit from "@tiptap/starter-kit";
import Placeholder from "@tiptap/extension-placeholder";
import TaskList from "@tiptap/extension-task-list";
import TaskItem from "@tiptap/extension-task-item";

interface RichTextEditorProps {
  content: string;
  onChange: (html: string) => void;
  placeholder?: string;
  editable?: boolean;
  minHeight?: string;
}

export function RichTextEditor({ content, onChange, placeholder = "Write something...", editable = true, minHeight = "200px" }: RichTextEditorProps) {
  const editor = useEditor({
    extensions: [
      StarterKit,
      Placeholder.configure({ placeholder }),
      TaskList,
      TaskItem.configure({ nested: true }),
    ],
    content,
    editable,
    onUpdate: ({ editor }) => onChange(editor.getHTML()),
  });

  if (!editor) return null;

  return (
    <div className="border rounded-md bg-background" style={{ minHeight }}>
      <div className="flex flex-wrap gap-1 p-2 border-b bg-muted/30">
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
      </div>
      <EditorContent editor={editor} className="prose prose-sm dark:prose-invert max-w-none p-3 [&_.ProseMirror]:outline-none [&_.ProseMirror]:min-h-[150px]" />
    </div>
  );
}

function ToolBtn({ onClick, active, label, style }: { onClick: () => void; active: boolean; label: string; style?: React.CSSProperties }) {
  return (
    <button
      type="button"
      onClick={onClick}
      style={style}
      className={`px-2 py-1 text-xs rounded ${active ? "bg-primary text-primary-foreground" : "hover:bg-accent"}`}
    >
      {label}
    </button>
  );
}
