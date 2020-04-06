import * as messages from "./messages";
import { logInfo } from "./logging";
import renderer, { pure } from "./renderer";
import { addClass, removeClass, setText, setHtml, setDisplay } from "./util";
import worker from "./worker-client";

import { header, generateLink, whatsThis, toggleEdit, showAll } from "./header";
import { aside } from "./explanation-aside";
import codemirror from "./codemirror";

const document = window.document;
const querySelector = (selector) => document.querySelector(selector);
const createElement = (el) => document.createElement(el);

export const PENDING = 0;
export const SUCCESS = 1;
export const ERROR = 2;

if (typeof __ANALYTICS_URL__ === "string") {
  fetch(__ANALYTICS_URL__, { method: "POST" });
}

const IS_TOUCH_DEVICE = "ontouchstart" in window;

addClass(document.body, IS_TOUCH_DEVICE ? "touch-device" : "non-touch-device");

/* CODEMIRROR INIT */
let cm;
let codemirrorEl;

codemirror({
  isTouchDevice: IS_TOUCH_DEVICE,
  anchor: querySelector(".codemirror-anchor"),
  onClick() {
    onCmClick();
  },
  onMouseMove(_cm, e) {
    onCmMouseMove(e);
  },
  onChange() {
    onCmChange();
  },
})
  .then(({ cm: instance, codemirrorEl: el }) => {
    cm = instance;
    codemirrorEl = el;

    return initialCodeRender(cm);
  })
  .then(() => {
    addClass(document.body, "codemirror-rendered");
    setState({ editorReady: true });
  });

/* HEADER */

const renderGenerateLink = generateLink({
  onAddress(address) {
    setState({ address });
  },
  getValue() {
    return cm && cm.getValue();
  },
});

whatsThis();

const renderToggleEdit = toggleEdit({
  onToggleEdit() {
    setState(({ editable }) => ({ editable: !editable }));
  },
});

const renderShowAll = showAll({
  onToggleShowAll() {
    setState(({ compilation }) => ({
      compilation: {
        ...compilation,
        showAll: !compilation.showAll,
      },
    }));
  },
});

/* EXPLANATION ASIDE */

const renderAside = aside({
  onWrapInBlock() {
    if (cm == null) return;
    const lines = cm.lineCount();
    for (let i = 0; i < lines; i++) {
      cm.indentLine(i, "add");
    }
    cm.replaceRange("fn main() {\n", { line: 0, ch: 0 });
    cm.replaceRange("\n}", {
      line: lines,
      ch: cm.getLineHandle(lines).text.length,
    });
  },
});

/* "REACT" */
let state = {
  compilation: { state: PENDING },
  editable: !IS_TOUCH_DEVICE,
};

let nonUiState = {
  lastRule: -1,
};

const setState = renderer(
  (prevState) => {
    const { compilation } = state;

    // EDITOR
    renderHover({ hoverEl: compilation.hoverEl });
    renderErrorMarks({ error: compilation.error });
    renderElaborationMark({ elaboration: compilation.elaboration });
    renderExplanationMark({ explanation: compilation.explanation });
    renderCodeEditor({
      showAll: compilation.showAll,
      editable: state.editable,
    });

    // ASIDE
    renderAside({
      elaboration: compilation.elaboration,
      error: compilation.error,
      compilationState: compilation.state,
    });

    // HEADER
    renderGenerateLink({
      address: state.address,
      enabled: state.editorReady,
    });
    renderToggleEdit({
      editable: state.editable,
      enabled: state.editorReady,
    });
    renderShowAll({
      showAll: compilation.showAll,
      empty: state.empty,
      canShow: compilation.exploration != null,
      failedCompilation: compilation.state === ERROR,
    });
  },
  {
    get() {
      return state;
    },
    set(nextState) {
      state = nextState;
      if (!self.__PRODUCTION__) {
        window.state = state;
      }
    },
  }
);

/* WORKER */

let { postMessage: postToWorker, ready: workerIsReadyPromise } = worker({
  onMessage(data) {
    switch (data.type) {
      case messages.COMPILED:
        setState({ compilation: { state: SUCCESS } });
        break;
      case messages.COMPILATION_ERROR:
        setState({ compilation: { state: ERROR, error: data.error } });
        break;
      case messages.EXPLANATION:
        setState(({ compilation }) => ({
          compilation: { ...compilation, explanation: data.location },
        }));
        onExplanation();
        break;
      case messages.ELABORATION:
        setState(({ compilation }) => ({
          compilation: {
            ...compilation,
            elaboration: data.location != null ? data : null,
          },
        }));
        break;
      case messages.EXPLORATION:
        computeExploration(data.exploration);
        setState(({ compilation }) => ({
          compilation: { ...compilation, exploration: true },
        }));
        break;
      default:
        console.error("Unexpected message in window", data);
    }
  },
});

/* CODE */

const styleSheet = (() => {
  const styleEl = document.createElement("style");
  document.head.appendChild(styleEl);
  return styleEl.sheet;
})();

function initialCodeRender(cm) {
  let promise = Promise.resolve();

  const codeParam = [...new window.URLSearchParams(location.search)].find(
    ([key, value]) => key === "code"
  );
  const code =
    codeParam != null ? window.decodeURIComponent(codeParam[1]) : null;

  if (code != null && code.trim() !== "") {
    cm.setValue(code);
    return promise;
  }

  const local = window.localStorage.getItem("code");
  if (local != null && local.trim() !== "") {
    cm.setValue(local);
    return promise;
  }

  promise =
    document.readyState === "loading"
      ? new Promise((resolve) => {
          document.addEventListener("DOMContentLoaded", resolve);
        })
      : Promise.resolve();

  promise = promise.then(() =>
    cm.setValue(querySelector(".default-code").value)
  );

  return promise;
}

const compileOnChange = (() => {
  let workerIsReady = false;
  workerIsReadyPromise.then(() => {
    workerIsReady = true;
  });
  let firstCompilationEnqueued = false;

  return () => {
    if (workerIsReady) {
      doCompile();
    } else if (!firstCompilationEnqueued) {
      firstCompilationEnqueued = true;
      workerIsReadyPromise.then(() => doCompile());
    }
  };
})();

function onCmChange() {
  setState({
    compilation: { state: PENDING },
    address: null,
    empty: cm.getValue() === "",
  });
  compileOnChange();
}

function onCmMouseMove(e) {
  if (state.compilation.state !== SUCCESS) {
    return;
  }

  if (nonUiState.computedMarks) {
    setState(({ compilation }) => ({
      compilation: { ...compilation, hoverEl: e.target },
    }));
    return;
  }

  const { clientX, clientY } = e;

  explain({ clientX, clientY });
}

function onCmClick() {
  elaborate(cm.getCursor("from"));
}

function elaborate(location) {
  if (state.compilation.state !== SUCCESS || state.empty) return;
  postToWorker({
    type: messages.ELABORATE,
    location,
  });
}

const { debounced: explain, done: doneAfterExplanation } = debounceUntilDone(
  function explain({ clientX: left, clientY: top }, done) {
    const { compilation } = state;

    if (compilation.state !== SUCCESS) {
      return done();
    }

    let { line, ch } = cm.coordsChar({ left, top }, "window");

    explainLocation({ line, ch });

    if (explainLocation.cached) {
      done();
    }
  },
  200
);

const explainLocation = memoize(
  function explainLocation({ line, ch }) {
    postToWorker({
      type: messages.EXPLAIN,
      location: { line, ch },
    });
  },
  (prev, current) => {
    if (prev.line === current.line && prev.ch === current.ch) {
      return true;
    }

    const { explanation } = state.compilation;

    return (
      explanation != null &&
      withinRange(current, explanation.start, explanation.end)
    );
  }
);

function onExplanation() {
  if (nonUiState.computedMarks) return;
  doneAfterExplanation();
}

function computeExploration(exploration) {
  nonUiState.computedMarks = exploration.map(({ start, end }, i) => {
    return getMark({ start, end }, `computed-${i}`);
  });

  for (let i = nonUiState.lastRule + 1; i < exploration.length; i++) {
    styleSheet.insertRule(
      `.hover-${i} .computed-${i} { background: #eee8d5 }`,
      styleSheet.cssRules.length
    );
  }

  nonUiState.lastRule = Math.max(exploration.length, nonUiState.lastRule);

  nonUiState.hoverMark && nonUiState.hoverMark.clear();
}

const debouncedCompile = debounce(
  (source) =>
    postToWorker({
      type: messages.COMPILE,
      source,
    }),
  128
);

function doCompile() {
  const code = cm.getValue();
  window.localStorage.setItem("code", code);

  if (code.trim() === "") {
    setState({ compilation: { state: SUCCESS } });
  } else {
    debouncedCompile(cm.getValue());
  }

  const { computedMarks } = nonUiState;
  nonUiState.computedMarks = null;
  computedMarks &&
    requestAnimationFrame(() => computedMarks.forEach((mark) => mark.clear()));
}

const renderHover = pure(function renderHover({ hoverEl }) {
  const klass =
    hoverEl &&
    [...hoverEl.classList].find((klass) => klass.startsWith("computed-"));
  const newIndex = klass && Number(klass.replace("computed-", ""));

  if (nonUiState.hoverIndex != null && newIndex !== nonUiState.hoverIndex) {
    removeClass(codemirrorEl, `hover-${nonUiState.hoverIndex}`);
  }

  if (newIndex != null) {
    addClass(codemirrorEl, `hover-${newIndex}`);
  }

  nonUiState.hoverIndex = newIndex;
});

const renderErrorMarks = pure(function renderErrorMarks({ error }) {
  nonUiState.errorMark && nonUiState.errorMark.clear();
  nonUiState.errorContextMark && nonUiState.errorContextMark.clear();

  if (error != null) {
    nonUiState.errorMark = getMark(error, "compilation-error");
    nonUiState.errorContextMark = getMark(
      {
        start: {
          ...error.start,
          ch: 0,
        },
        end: {
          ...error.end,
          ch: cm.getLine(error.end.line).length,
        },
      },
      "compilation-error"
    );
  }
});

const renderElaborationMark = pure(function renderElaborationMark({
  elaboration,
}) {
  nonUiState.mark && nonUiState.mark.clear();

  if (elaboration != null) {
    nonUiState.mark = getMark(elaboration.location);
  }
});

const renderExplanationMark = pure(function renderExplanationMark({
  explanation,
}) {
  nonUiState.hoverMark && nonUiState.hoverMark.clear();
  if (explanation == null || nonUiState.computedMarks != null) return;
  nonUiState.hoverMark = getMark(explanation);
});

const renderCodeEditor = pure(function renderCodeEditor({ showAll, editable }) {
  cm.setOption("readOnly", editable ? false : "nocursor");

  if (showAll) {
    addClass(codemirrorEl, "show-all-computed");
  } else {
    removeClass(codemirrorEl, "show-all-computed");
  }
});

/* HELPERS */

function getMark(location, className = "highlighted") {
  return cm.markText(location.start, location.end, {
    className,
  });
}

function debounce(fn, delay) {
  let enqueued = null;
  let lastArg = null;
  const wrapped = () => fn(lastArg);

  return (arg) => {
    lastArg = arg;
    if (enqueued != null) {
      window.clearTimeout(enqueued);
    }
    enqueued = setTimeout(wrapped, delay);
  };
}

function singleExecutionUntilReady(fn, check, promise) {
  let last;
  let enqueued = false;

  return (...args) => {
    if (check()) {
      fn(...args);
    } else {
      last = args;

      if (!enqueued) {
        promise.then(() => fn(...last));
      }
    }
  };
}

function memoize(fn, memoizer) {
  let last = {};

  let memoized = (arg) => {
    if (memoizer(last, arg)) {
      last = arg;
      memoized.cached = true;
    } else {
      last = arg;
      memoized.cached = false;
      fn(arg);
    }
  };

  return memoized;
}

function debounceUntilDone(fn, delay) {
  let isOpen = true;
  const sentinel = {};
  let last = sentinel;
  let enqueued = false;

  const done = () => {
    if (last !== sentinel) {
      if (!enqueued) {
        enqueued = true;
        window.setTimeout(() => {
          enqueued = false;
          const arg = last;
          last = sentinel;
          fn(arg, done);
        }, delay);
      }
    } else {
      isOpen = true;
    }
  };

  return {
    done,
    debounced(arg) {
      if (isOpen) {
        isOpen = false;
        fn(arg, done);
      } else {
        last = arg;
      }
    },
  };
}

function withinRange({ line, ch }, start, end) {
  return (
    (start.line < line || (start.line === line && start.ch <= ch)) &&
    (line < end.line || (line === end.line && ch <= end.ch))
  );
}

if (!self.__PRODUCTION__) {
  window.cm = cm;
  window.nonUiState = nonUiState;
}
