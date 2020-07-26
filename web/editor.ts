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
  anchor: HTMLTextAreaElement;
  isTouchDevice: boolean;
  onChange: (editor: Editor, event: EditorChange) => void;
  onMouseMove: (editor: Editor, event: MouseEvent) => void;
  onClick: (editor: Editor, event: MouseEvent) => void;
}) {
  let cmPromise = importPromise.then(([{ fromTextArea }]) => {
    const cm = fromTextArea(anchor, {
      mode: "rust",
      lineNumbers: true,
      theme: "solarized",
      readOnly: isTouchDevice ? "nocursor" : false,
      indentUnit: 4,
    });

    const codemirrorEl = cm.getWrapperElement();

    cm.on("change", (_instance: Editor, e: EditorChange) => onChange(cm, e));
    codemirrorEl.addEventListener("mousemove", (e: MouseEvent) =>
      onMouseMove(cm, e)
    );
    codemirrorEl.addEventListener("click", (e: MouseEvent) => onClick(cm, e));

    return { cm, codemirrorEl };
  });

  cmPromise.catch((e) => reportError("cmPromise", e));

  return cmPromise;
}
