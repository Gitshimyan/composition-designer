import { useEffect, useRef } from "react";
import { basicSetup } from "codemirror";
import { StreamLanguage } from "@codemirror/language";
import { toml } from "@codemirror/legacy-modes/mode/toml";
import { EditorState } from "@codemirror/state";
import { oneDark } from "@codemirror/theme-one-dark";
import { EditorView } from "@codemirror/view";

const theme = EditorView.theme(
  {
    "&": {
      height: "100%",
      fontSize: "11px",
      backgroundColor: "#0b1119",
    },
    ".cm-scroller": {
      fontFamily: "Cascadia Code, Consolas, monospace",
      lineHeight: "1.55",
      overflow: "auto",
    },
    ".cm-gutters": {
      backgroundColor: "#0b1119",
      color: "#5e7187",
      border: "none",
    },
    ".cm-activeLine": {
      backgroundColor: "rgba(94, 224, 189, 0.06)",
    },
    ".cm-activeLineGutter": {
      backgroundColor: "rgba(94, 224, 189, 0.08)",
      color: "#9ed9c7",
    },
    ".cm-selectionBackground, ::selection": {
      backgroundColor: "rgba(94, 224, 189, 0.22) !important",
    },
    ".cm-cursor": {
      borderLeftColor: "#5ee0bd",
    },
  },
  { dark: true },
);

interface SourceViewProps {
  text: string;
  revealLine: { line: number; nonce: number } | null;
}

export default function SourceView({ text, revealLine }: SourceViewProps) {
  const host = useRef<HTMLDivElement>(null);
  const view = useRef<EditorView | null>(null);

  useEffect(() => {
    if (!host.current) return;
    const state = EditorState.create({
      doc: text,
      extensions: [
        basicSetup,
        StreamLanguage.define(toml),
        oneDark,
        theme,
        EditorState.readOnly.of(true),
        EditorView.editable.of(false),
        EditorView.lineWrapping,
      ],
    });
    const editor = new EditorView({ state, parent: host.current });
    view.current = editor;
    return () => {
      editor.destroy();
      view.current = null;
    };
    // The editor is created once; text updates are dispatched below.
    // eslint-disable-next-line react-hooks/exhaustive-deps
  }, []);

  useEffect(() => {
    const editor = view.current;
    if (!editor) return;
    const current = editor.state.doc.toString();
    if (current === text) return;
    editor.dispatch({
      changes: { from: 0, to: current.length, insert: text },
    });
  }, [text]);

  useEffect(() => {
    const editor = view.current;
    if (!editor || !revealLine) return;
    const lineNumber = Math.min(
      Math.max(revealLine.line, 1),
      editor.state.doc.lines,
    );
    const line = editor.state.doc.line(lineNumber);
    editor.dispatch({
      selection: { anchor: line.from },
      effects: EditorView.scrollIntoView(line.from, { y: "center" }),
    });
    editor.focus();
  }, [revealLine]);

  return <div className="source-view" ref={host} aria-label="Read-only TOML source" />;
}
