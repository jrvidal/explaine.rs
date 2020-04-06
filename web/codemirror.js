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

    cm.on("change", (e) => onChange(cm, e));
    codemirrorEl.addEventListener("mousemove", (e) => onMouseMove(cm, e));
    codemirrorEl.addEventListener("click", (e) => onClick(cm, e));

    return { cm, codemirrorEl };
  });

  cmPromise.catch((e) => console.error(e));

  return cmPromise;
}
