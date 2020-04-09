import { Editor, EditorChange } from "codemirror";
import { reportError } from "./logging";

let importPromise = Promise.all([
  import("codemirror"),
  import("codemirror/addon/mode/simple"),
  import("codemirror/mode/rust/rust"),
]);

export default function codemirror({
  anchor,
  isTouchDevice,
  onChange,
  onMouseMove,
  onClick,
}: {
  anchor: HTMLElement;
  isTouchDevice: boolean;
  onChange: (editor: Editor, event: EditorChange) => void;
  onMouseMove: (editor: Editor, event: MouseEvent) => void;
  onClick: (editor: Editor, event: MouseEvent) => void;
}) {
  let cmPromise = importPromise.then(([{ default: CodeMirror }]) => {
    const cm = CodeMirror.fromTextArea(anchor, {
      mode: "rust",
      lineNumbers: true,
      theme: "solarized",
      readOnly: isTouchDevice ? "nocursor" : false,
      indentUnit: 4,
    });

    const codemirrorEl = cm.getWrapperElement();

    cm.on("change", (e: EditorChange) => onChange(cm, e));
    codemirrorEl.addEventListener("mousemove", (e: MouseEvent) =>
      onMouseMove(cm, e)
    );
    codemirrorEl.addEventListener("click", (e: MouseEvent) => onClick(cm, e));

    return { cm, codemirrorEl };
  });

  cmPromise.catch((e) =>
    reportError("cmPromise", {
      message: e && e.message,
      error: e,
    })
  );

  return cmPromise;
}
