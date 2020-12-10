import * as messages from "./messages";
import renderer, { pure } from "./renderer";
import { addClass, removeClass, compareLocations } from "./util";
import worker from "./worker-client";
import { reportHit } from "./logging";

import { TextMarker, Editor } from "codemirror";

import {
  generateLink,
  whatsThis,
  toggleEdit,
  showAll,
  openInPlayground,
} from "./header";
import { aside } from "./explanation-aside";
import codemirror from "./editor";
import {
  Location,
  MissingHint,
  PENDING,
  CompilationState,
  ERROR,
  SUCCESS,
} from "./types";
import { getFromStorage, UNKNOWN, setInStorage } from "./storage";

declare global {
  var __ANALYTICS_URL__: any;
  var __PRODUCTION__: any;
}

const document = window.document;
const querySelector = (selector: string) => document.querySelector(selector);

reportHit();

const IS_TOUCH_DEVICE = "ontouchstart" in window;

addClass(document.body, IS_TOUCH_DEVICE ? "touch-device" : "non-touch-device");

/* CODEMIRROR INIT */
let cm: Editor;
let codemirrorEl: HTMLElement;

codemirror({
  isTouchDevice: IS_TOUCH_DEVICE,
  anchor: querySelector(".codemirror-anchor") as HTMLTextAreaElement,
  onClick() {
    onCmClick();
  },
  onMouseMove(_cm: any, e: MouseEvent) {
    onCmMouseMove(e);
  },
  onChange() {
    onCmChange();
  },
})
  .then(({ cm: instance, codemirrorEl: el }) => {
    cm = instance;

    if (!self.__PRODUCTION__) {
      (window as any).cm = cm;
    }
    codemirrorEl = el;

    return initialCodeRender(cm);
  })
  .then(() => {
    addClass(document.body, "codemirror-rendered");
    setState({ editorReady: true });
  });

/* HEADER */

const renderGenerateLink = generateLink({
  onAddress(address: string) {
    setState({ address });
  },
  getValue() {
    const code = cm && cm.getValue();
    return code != null
      ? {
          code,
          location: nonUiState.lastElaborationRequest,
        }
      : null;
  },
});

whatsThis();

const renderToggleEdit = toggleEdit({
  onToggleEdit() {
    setState(({ editable }: State) => ({ editable: !editable }));
  },
});

const renderShowAll = showAll({
  onToggleShowAll() {
    setState(({ compilation }: State) => ({
      compilation: {
        ...compilation,
        exploration:
          compilation.exploration != null
            ? {
                showAll: !compilation.exploration.showAll,
              }
            : null,
      },
    }));
  },
});

const renderOpenInPlayground = openInPlayground({
  getValue() {
    return cm && cm.getValue();
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
export type CompilationError = Span & { isBlock: boolean; msg: string };
export type Elaboration = {
  location: Span;
  title: string;
  elaboration: string;
  extraInfo: { link: string; kind: string }[];
};

type State = {
  editable: boolean;
  compilation: {
    state: CompilationState;
    hoverEl: EventTarget | null;
    elaboration: Elaboration | null;
    hitbox: Span | null;
    exploration: {
      showAll: boolean;
    } | null;
    error: CompilationError | null;
    missing: MissingHint | null;
  };
  address: string | null;
  editorReady: boolean;
  empty: boolean;
};

type Span = {
  start: Location;
  end: Location;
};

let state: State = {
  compilation: {
    state: PENDING,
    hoverEl: null,
    hitbox: null,
    elaboration: null,
    exploration: null,
    error: null,
    missing: null,
  },
  editable: !IS_TOUCH_DEVICE,
  address: null,
  editorReady: false,
  empty: false,
};

const initialCompilation = state.compilation;

type NonUIState = {
  rules: number;
  mark: TextMarker | null;
  hoverMark: TextMarker | null;
  computedMarks: TextMarker[] | null;
  errorMark: TextMarker | null;
  errorContextMark: TextMarker | null;
  hoverIndex: number | null;
  compilationIndex: number;
  elaborationIndex: number | null;
  lastElaborationRequest: Location | null;
  pendingInitialElaboration: Location | null;
};

let nonUiState: NonUIState = {
  rules: 0,
  mark: null,
  hoverMark: null,
  computedMarks: null,
  errorMark: null,
  errorContextMark: null,
  hoverIndex: null,
  compilationIndex: 0,
  elaborationIndex: null,
  lastElaborationRequest: null,
  pendingInitialElaboration: null,
};

const setState = renderer<State>(
  () => {
    const { compilation } = state;

    // EDITOR
    renderHover({ hoverEl: compilation.hoverEl });
    renderErrorMarks({ error: compilation.error });
    renderElaborationMark({ elaboration: compilation.elaboration });
    renderHitboxMark({ hitbox: compilation.hitbox });
    renderCodeEditor({
      showAll: compilation.exploration?.showAll ?? false,
      editable: state.editable,
    });
    // renderMissingTooltip({
    //   missing: compilation.missing,
    // });

    // ASIDE
    renderAside({
      elaboration: compilation.elaboration,
      error: compilation.error,
      compilationState: compilation.state,
      missing: compilation.missing,
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
      showAll:
        !state.empty &&
        compilation.state !== ERROR &&
        compilation.exploration?.showAll,
      enabled: compilation.exploration != null,
    });
    renderOpenInPlayground({
      enabled: !state.empty && compilation.state === SUCCESS,
    });
  },
  {
    get() {
      return state;
    },
    set(nextState: State) {
      state = nextState;
      if (!self.__PRODUCTION__) {
        (window as any).state = state;
      }
    },
  }
);

/* WORKER */

let { postMessage: postToWorker, ready: workerIsReadyPromise } = worker({
  onMessage(data) {
    switch (data.type) {
      case messages.COMPILED:
        onCompilation();
        break;
      case messages.COMPILATION_ERROR:
        setState({
          compilation: {
            ...initialCompilation,
            state: ERROR,
            error: data.error,
          },
        });
        break;
      case messages.HITBOX:
        setState(({ compilation }: State) => ({
          compilation: { ...compilation, hitbox: data.location },
        }));
        onHitboxReceived();
        break;
      case messages.ELABORATION:
        onElaboration(data);
        break;
      case messages.EXPLORATION:
        computeExploration(data.exploration);
        setState(({ compilation }: State) => ({
          compilation: { ...compilation, exploration: { showAll: false } },
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
  return styleEl.sheet as CSSStyleSheet;
})();

function initialCodeRender(cm: any) {
  let promise = Promise.resolve();

  const params = new URLSearchParams(location.search);

  let code = params.get("code");
  const lineParam = params.get("line");
  const chParam = params.get("ch");

  const codeParam = [...new window.URLSearchParams(location.search)].find(
    ([key, _value]) => key === "code"
  );
  code = code != null ? decodeURIComponent(code) : null;
  const line = lineParam != null ? Number(lineParam) : null;
  const ch = chParam != null ? Number(chParam) : null;

  if (code != null && code.trim() !== "") {
    cm.setValue(code);
    const location =
      line != null && Number.isFinite(line) && ch != null && Number.isFinite(ch)
        ? { line, ch }
        : null;

    nonUiState.pendingInitialElaboration = location;
    return promise;
  }

  const local = getFromStorage("code");
  if (typeof local === "string" && local.trim() !== "") {
    cm.setValue(local);
    return promise;
  }

  promise =
    document.readyState === "loading"
      ? new Promise((resolve) => {
          document.addEventListener("DOMContentLoaded", () => resolve());
        })
      : Promise.resolve();

  promise = promise.then(() =>
    cm.setValue((querySelector(".default-code") as any).value)
  );

  return promise;
}

const compileOnChange = (() => {
  let firstCompilationEnqueued = false;
  let firstCompilationDispatched = false;

  return () => {
    if (firstCompilationDispatched || !firstCompilationEnqueued) {
      firstCompilationEnqueued = true;
      workerIsReadyPromise.then(() => {
        firstCompilationDispatched = true;
        doCompile();
      });
    }
  };
})();

function onCmChange() {
  nonUiState.compilationIndex += 1;
  setState({
    compilation: initialCompilation,
    address: null,
    empty: cm.getValue().trim() === "",
  });
  compileOnChange();
}

function onCmMouseMove(e: MouseEvent) {
  if (state.compilation.state !== SUCCESS) {
    return;
  }

  if (nonUiState.computedMarks) {
    setState(({ compilation }: State) => ({
      compilation: { ...compilation, hoverEl: e.target },
    }));
    return;
  }

  computeHitbox(e);
}

function onCmClick() {
  if (nonUiState.pendingInitialElaboration != null) return;
  elaborate(cm.getCursor("from"));
}

function elaborate(location: Location, isReady = false) {
  if ((!isReady && state.compilation.state !== SUCCESS) || state.empty) return;
  nonUiState.elaborationIndex = nonUiState.compilationIndex;
  nonUiState.lastElaborationRequest = location;
  setState({ address: null });
  postToWorker({
    type: messages.ELABORATE,
    location,
  });
}

const { debounced: computeHitbox, done: doneAfterHitbox } = debounceUntilDone(
  function computeHitbox(
    { clientX: left, clientY: top }: MouseEvent,
    done: () => void
  ) {
    const { compilation } = state;

    if (compilation.state !== SUCCESS) {
      return done();
    }

    let { line, ch } = cm.coordsChar({ left, top }, "window");

    computeHitboxAtLocation({ line, ch });

    if (computeHitboxAtLocation.cached) {
      done();
    }
  },
  200
);

const computeHitboxAtLocation = memoize(
  function computeHitboxAtLocation({ line, ch }: Location) {
    postToWorker({
      type: messages.GET_HITBOX,
      location: { line, ch },
    });
  },
  (prev: Location, current: Location) => {
    if (prev.line === current.line && prev.ch === current.ch) {
      return true;
    }

    const { hitbox } = state.compilation;

    return hitbox != null && withinRange(current, hitbox.start, hitbox.end);
  }
);

function onCompilation() {
  setState({ compilation: { ...initialCompilation, state: SUCCESS } });
  if (nonUiState.pendingInitialElaboration != null) {
    elaborate(nonUiState.pendingInitialElaboration, true);
    nonUiState.pendingInitialElaboration = null;
  }
}

function onHitboxReceived() {
  if (nonUiState.computedMarks) return;
  doneAfterHitbox();
}

function onElaboration(elaboration: Elaboration | { location: null }) {
  if (nonUiState.compilationIndex !== nonUiState.elaborationIndex) return;

  const missing =
    elaboration.location == null
      ? computeMissingHint(nonUiState.lastElaborationRequest!!)
      : null;

  setState(({ compilation }: State) => ({
    compilation: {
      ...compilation,
      elaboration: elaboration.location != null ? elaboration : null,
      missing,
    },
  }));
}

function computeMissingHint({ line, ch }: Location) {
  const MARGIN = 5;
  const EMPTY_RE = /^ *$/;

  if (EMPTY_RE.test(cm.getLine(line))) return null;

  const minContextLine = Math.max(0, line - MARGIN);
  const maxContentLine = Math.min(cm.lineCount() - 1, line + MARGIN);

  let lines = [...new Array(maxContentLine - minContextLine + 1)].map((_, i) =>
    cm.getLine(minContextLine + i)
  );

  const indentationPerLine = lines.map((line) =>
    EMPTY_RE.test(line)
      ? Number.POSITIVE_INFINITY
      : line.match(/^ */)?.[0].length ?? 0
  );

  let indentation = Math.min(...indentationPerLine);
  indentation =
    indentation === Number.POSITIVE_INFINITY ? 0 : Math.min(indentation, ch);

  if (indentation > 0) {
    lines.forEach((line, i) => {
      if (!EMPTY_RE.test(line)) {
        lines[i] = line.slice(indentation);
      }
    });
  }
  lines.forEach((line, i) => {
    lines[i] = `${String(i).padStart(2, " ")} | ${line}`;
  });
  lines.splice(
    line - minContextLine + 1,
    0,
    `   | ${" ".repeat(ch - indentation)}↑`
  );

  return {
    code: lines.join("\n"),
    location: {
      line: line - minContextLine,
      ch: ch - indentation,
    },
  };
}

function computeExploration(exploration: Span[]) {
  nonUiState.computedMarks = exploration.map(({ start, end }, i) => {
    return getMark({ start, end }, `computed-${i}`);
  });

  for (let i = nonUiState.rules; i < exploration.length; i++) {
    styleSheet.insertRule(
      `.hover-${i} .computed-${i} { background: #e9deba; font-weight: bold; }`,
      styleSheet.cssRules.length
    );
  }

  nonUiState.rules = Math.max(exploration.length, nonUiState.rules);

  nonUiState.hoverMark && nonUiState.hoverMark.clear();
}

const debouncedCompile = debounce(
  (source: string) =>
    postToWorker({
      type: messages.COMPILE,
      source,
    }),
  512
);

function doCompile() {
  const code = cm.getValue();
  setInStorage("code", code);

  nonUiState.lastElaborationRequest = null;
  if (code.trim() === "") {
    setState({ compilation: { ...initialCompilation, state: SUCCESS } });
  } else {
    postToWorker({ type: messages.STOP_EXPLORATION });
    debouncedCompile(cm.getValue());
  }

  const { computedMarks } = nonUiState;
  nonUiState.computedMarks = null;
  computedMarks &&
    requestAnimationFrame(() => computedMarks.forEach((mark) => mark.clear()));
}

const renderHover = pure(function renderHover({
  hoverEl,
}: {
  hoverEl: EventTarget | null;
}) {
  const PREFIX = "computed-";
  const PREFIX_LEN = PREFIX.length;

  const indices =
    hoverEl &&
    [...(hoverEl as HTMLElement).classList]
      .filter((klass) => klass.startsWith(PREFIX))
      .map((klass) => Number(klass.slice(PREFIX_LEN)));

  // "Most visible" mark
  // Sort by (first to end, last to start) and take the minimum
  let mostVisible = null as { mark: TextMarker; index: number } | null;

  (indices || []).forEach((index) => {
    const mark = nonUiState.computedMarks!![index];

    if (mostVisible == null) {
      mostVisible = { mark, index };
      return;
    }

    const { from: fromA, to: toA } = mostVisible.mark.find();
    const { from: fromB, to: toB } = mark.find();

    const toCmp = compareLocations(toA, toB);
    const cmp = toCmp === 0 ? -compareLocations(fromA, fromB) : toCmp;

    if (cmp > 0) {
      mostVisible = { mark, index };
    }
  });

  const newIndex = mostVisible?.index ?? null;

  if (newIndex != nonUiState.hoverIndex) {
    if (nonUiState.hoverIndex != null) {
      removeClass(codemirrorEl, `hover-${nonUiState.hoverIndex}`);
    }
    if (newIndex != null) {
      addClass(codemirrorEl, `hover-${newIndex}`);
    }
  }

  nonUiState.hoverIndex = newIndex;
});

const renderErrorMarks = pure(function renderErrorMarks({
  error,
}: {
  error: CompilationError | null;
}) {
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
}: {
  elaboration: Elaboration | null;
}) {
  nonUiState.mark && nonUiState.mark.clear();

  if (elaboration != null) {
    nonUiState.mark = getMark(elaboration.location);
  }
});

const renderHitboxMark = pure(function renderHitboxMark({
  hitbox,
}: {
  hitbox: Span | null;
}) {
  nonUiState.hoverMark && nonUiState.hoverMark.clear();
  if (hitbox == null || nonUiState.computedMarks != null) return;
  nonUiState.hoverMark = getMark(hitbox, "hover-highlight");
});

const renderCodeEditor = pure(function renderCodeEditor({
  showAll,
  editable,
}: {
  showAll: boolean;
  editable: boolean;
}) {
  cm.setOption("readOnly", editable ? false : "nocursor");

  if (showAll) {
    addClass(codemirrorEl, "show-all-computed");
  } else {
    removeClass(codemirrorEl, "show-all-computed");
  }
});

/* HELPERS */

function getMark(location: Span, className = "highlighted") {
  return cm.markText(location.start, location.end, {
    className,
  });
}

function memoize<T>(fn: any, memoizer: (prev: T, next: T) => boolean) {
  let last: any = {};

  let memoized: any = (arg: T) => {
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

function debounceUntilDone<T>(
  fn: (arg: T, done: () => void) => void,
  delay: number
) {
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
          fn(arg as T, done);
        }, delay);
      }
    } else {
      isOpen = true;
    }
  };

  return {
    done,
    debounced(arg: T) {
      if (isOpen) {
        isOpen = false;
        fn(arg, done);
      } else {
        last = arg;
      }
    },
  };
}

function withinRange(loc: Location, start: Location, end: Location) {
  return compareLocations(start, loc) <= 0 && compareLocations(loc, end) <= 0;
}

if (!self.__PRODUCTION__) {
  (window as any).nonUiState = nonUiState;
}

// Copy&paste from lodash
function debounce(
  func: (...args: any) => void,
  wait: number,
  options: any = undefined
): (...args: any) => void {
  let lastArgs: any;
  let lastThis: any;
  let maxWait: any;
  let result: any;
  let timerId: number | undefined;
  let lastCallTime: any;

  var lastInvokeTime = 0,
    leading = false,
    maxing = false,
    trailing = true;

  if (options != null) {
    leading = !!options.leading;
    maxing = "maxWait" in options;
    maxWait = maxing ? Math.max(Number(options.maxWait) || 0, wait) : maxWait;
    trailing = "trailing" in options ? !!options.trailing : trailing;
  }

  function invokeFunc(time: number) {
    var args = lastArgs,
      thisArg = lastThis;

    lastArgs = lastThis = undefined;
    lastInvokeTime = time;
    result = func.apply(thisArg, args);
    return result;
  }

  function leadingEdge(time: number) {
    // Reset any `maxWait` timer.
    lastInvokeTime = time;
    // Start the timer for the trailing edge.
    timerId = window.setTimeout(timerExpired, wait);
    // Invoke the leading edge.
    return leading ? invokeFunc(time) : result;
  }

  function remainingWait(time: number) {
    var timeSinceLastCall = time - lastCallTime,
      timeSinceLastInvoke = time - lastInvokeTime,
      timeWaiting = wait - timeSinceLastCall;

    return maxing
      ? Math.min(timeWaiting, maxWait - timeSinceLastInvoke)
      : timeWaiting;
  }

  function shouldInvoke(time: number) {
    var timeSinceLastCall = time - lastCallTime,
      timeSinceLastInvoke = time - lastInvokeTime;

    // Either this is the first call, activity has stopped and we're at the
    // trailing edge, the system time has gone backwards and we're treating
    // it as the trailing edge, or we've hit the `maxWait` limit.
    return (
      lastCallTime === undefined ||
      timeSinceLastCall >= wait ||
      timeSinceLastCall < 0 ||
      (maxing && timeSinceLastInvoke >= maxWait)
    );
  }

  function timerExpired() {
    var time = Date.now();
    if (shouldInvoke(time)) {
      return trailingEdge(time);
    }
    // Restart the timer.
    timerId = window.setTimeout(timerExpired, remainingWait(time));
  }

  function trailingEdge(time: number) {
    timerId = undefined;

    // Only invoke if we have `lastArgs` which means `func` has been
    // debounced at least once.
    if (trailing && lastArgs) {
      return invokeFunc(time);
    }
    lastArgs = lastThis = undefined;
    return result;
  }

  function cancel() {
    if (timerId !== undefined) {
      clearTimeout(timerId);
    }
    lastInvokeTime = 0;
    lastArgs = lastCallTime = lastThis = timerId = undefined;
  }

  function flush() {
    return timerId === undefined ? result : trailingEdge(Date.now());
  }

  function debounced(this: any) {
    var time = Date.now(),
      isInvoking = shouldInvoke(time);

    lastArgs = arguments;
    lastThis = this;
    lastCallTime = time;

    if (isInvoking) {
      if (timerId === undefined) {
        return leadingEdge(lastCallTime);
      }
      if (maxing) {
        // Handle invocations in a tight loop.
        clearTimeout(timerId);
        timerId = window.setTimeout(timerExpired, wait);
        return invokeFunc(lastCallTime);
      }
    }
    if (timerId === undefined) {
      timerId = window.setTimeout(timerExpired, wait);
    }
    return result;
  }
  debounced.cancel = cancel;
  debounced.flush = flush;
  return debounced;
}
