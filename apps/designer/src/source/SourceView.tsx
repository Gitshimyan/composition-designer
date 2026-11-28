import { useEffect, useRef, useState } from "react";
import { basicSetup } from "codemirror";
import { StreamLanguage } from "@codemirror/language";
import { toml } from "@codemirror/legacy-modes/mode/toml";
import { Compartment, EditorState } from "@codemirror/state";
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
  editable: boolean;
  blocked: boolean;
  onReview: (source: string) => void;
}

export default function SourceView({
  text,
  revealLine,
  editable,
  blocked,
  onReview,
}: SourceViewProps) {
  const host = useRef<HTMLDivElement>(null);
  const view = useRef<EditorView | null>(null);
  const textRef = useRef(text);
  const onReviewRef = useRef(onReview);
  const compartment = useRef(new Compartment()).current;
  const [dirty, setDirty] = useState(false);
  const [preserved, setPreserved] = useState(false);
  const dirtyRef = useRef(false);
  const canEdit = editable && !blocked;

  dirtyRef.current = dirty;
  textRef.current = text;
  onReviewRef.current = onReview;

  useEffect(() => {
    if (!host.current) return;
    const state = EditorState.create({
      doc: textRef.current,
      extensions: [
        basicSetup,
        StreamLanguage.define(toml),
        oneDark,
        theme,
        compartment.of(editableConfig(editable && !blocked)),
        EditorView.lineWrapping,
        EditorView.updateListener.of((update) => {
          if (update.docChanged) {
            setDirty(update.state.doc.toString() !== textRef.current);
          }
        }),
      ],
    });
    const editor = new EditorView({ state, parent: host.current });
    view.current = editor;
    return () => {
      editor.destroy();
      view.current = null;
    };
    // The editor is created once; text/read-only updates are dispatched below.
    // eslint-disable-next-line react-hooks/exhaustive-deps
  }, []);

  useEffect(() => {
    const editor = view.current;
    if (!editor) return;
    editor.dispatch({
      effects: compartment.reconfigure(editableConfig(editable && !blocked)),
    });
  }, [compartment, editable, blocked]);

  useEffect(() => {
    const editor = view.current;
    if (!editor) return;
    textRef.current = text;
    const current = editor.state.doc.toString();
    if (current === text) {
      setDirty(false);
      setPreserved(false);
      return;
    }
    if (dirtyRef.current) {
      // Keep the local draft across an external reload; the next review
      // validates it against the new base revision.
      setPreserved(true);
      return;
    }
    editor.dispatch({
      changes: { from: 0, to: current.length, insert: text },
    });
    setDirty(false);
    setPreserved(false);
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

  function review() {
    const editor = view.current;
    if (!editor || !canEdit) return;
    onReviewRef.current(editor.state.doc.toString());
  }

  function discard() {
    const editor = view.current;
    if (!editor) return;
    const current = editor.state.doc.toString();
    editor.dispatch({
      changes: { from: 0, to: current.length, insert: textRef.current },
    });
    setDirty(false);
    setPreserved(false);
  }

  return (
    <div className="source-editor-shell">
      <div className="source-toolbar">
        <span className="source-mode">
          {blocked
            ? "External change detected · Draft kept · Reload before editing"
            : preserved
              ? "Draft kept across external change · Review before writing"
              : canEdit
                ? "Editable · Review before writing"
                : "Read-only"}
        </span>
        {canEdit && (
          <div className="source-actions">
            <button
              type="button"
              className="button button-ghost source-action"
              onClick={discard}
              disabled={!dirty}
            >
              Discard
            </button>
            <button
              type="button"
              className="button button-primary source-action"
              onClick={review}
              disabled={!dirty}
            >
              Review source
            </button>
          </div>
        )}
      </div>
      <div
        className="source-view"
        ref={host}
        aria-label={canEdit ? "Editable TOML source" : "Read-only TOML source"}
      />
    </div>
  );
}

function editableConfig(editable: boolean) {
  return [
    EditorState.readOnly.of(!editable),
    EditorView.editable.of(editable),
  ];
}
