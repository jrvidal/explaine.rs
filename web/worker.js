import wasm_bindgen from "../pkg/explainers";
import * as messages from "./messages";
import wasmUrl from "../pkg/explainers_bg.wasm";
import { logInfo, reportError, handleLogging } from "./logging";

logInfo("workerMain");

const state = {
  source: null,
  session: null,
  explanation: null,
  exploration: null,
};

wasm_bindgen(wasmUrl)
  .then(() => postMessage({ type: messages.READY }))
  .catch((e) =>
    reportError("wasm_bindgen", {
      error: e,
      message: e && e.message,
    })
  );

self.onmessage = (e) => {
  const { data } = e;
  logInfo("Worker received", data.type, data);
  switch (data.type) {
    case messages.COMPILE:
      compile(data.source);
      break;
    case messages.EXPLAIN:
      explain(data.location);
      break;
    case messages.ELABORATE:
      elaborate(data.location);
      break;
    default:
      if (!self.__PRODUCTION__) {
        if (handleLogging(data)) {
          return;
        }
      }
      console.error("Unexpected message in worker", data);
  }
};

function compile(source) {
  if (state.session) {
    state.session.free();
    state.session = null;
  }
  const result = wasm_bindgen.Session.new(source);
  const errorMsg = result.error_message();
  const location = result.error_location();

  const error =
    errorMsg != null
      ? {
          msg: errorMsg,
          start: {
            line: location[0] - 1,
            ch: location[1],
          },
          end: {
            line: location[2] - 1,
            ch: location[3],
          },
          isBlock: result.is_block(),
        }
      : null;

  state.session = result.session();
  state.error = error;
  notifySession();
  exploreLoop(state.session, true);
}

function notifySession() {
  postMessage({
    type:
      state.session != null ? messages.COMPILED : messages.COMPILATION_ERROR,
    error: state.error,
  });
}

function exploreLoop(session, init = false) {
  if (session != state.session || state.session == null) {
    return;
  }

  const LENGTH = 5;
  const DELAY = 16;

  if (init) {
    state.exploration = {
      buffer: new self.Uint32Array(LENGTH * 4),
      result: [],
      byStart: new Map(),
    };
  }

  const { buffer, result, byStart } = state.exploration;
  const written = state.session.explore(buffer);

  for (let i = 0; i < written; i++) {
    const span = {
      start: { line: buffer[4 * i] - 1, ch: buffer[4 * i + 1] },
      end: { line: buffer[4 * i + 2] - 1, ch: buffer[4 * i + 3] },
    };
    if (!byStart.has(span.start.line)) {
      byStart.set(span.start.line, []);
    }

    const exists = byStart
      .get(span.start.line)
      .some(
        (s) =>
          s.start.line === span.start.line &&
          s.start.ch === span.start.ch &&
          s.end.line === span.end.line &&
          s.end.ch === span.end.ch
      );
    if (!exists) {
      byStart.get(span.start.line).push(span);
      result.push(span);
    }
  }

  if (written < LENGTH) {
    logInfo("Exploration finished...");
    postMessage({
      type: messages.EXPLORATION,
      exploration: state.exploration.result,
    });
    return;
  }

  setTimeout(() => exploreLoop(session), DELAY);
}

function explain(location) {
  doExplain(location);
  notifyExplanation();
}

function elaborate(location) {
  doExplain(location);
  notifyElaboration();
}

function doExplain(location) {
  if (state.explanation) {
    state.explanation.free();
    state.explanation = null;
  }

  state.explanation =
    state.session && state.session.explain(location.line + 1, location.ch);
}

function notifyExplanation() {
  postMessage({
    type: messages.EXPLANATION,
    location: explanationLocation(state.explanation),
  });
}

function notifyElaboration() {
  postMessage({
    type: messages.ELABORATION,
    location: explanationLocation(state.explanation),
    elaboration: state.explanation && state.explanation.elaborate(),
    title: state.explanation && state.explanation.title(),
    book: state.explanation && state.explanation.book(),
    keyword: state.explanation && state.explanation.keyword(),
    std: state.explanation && state.explanation.std(),
  });
}

function explanationLocation(explanation) {
  return explanation != null
    ? {
        start: {
          line: explanation.start_line - 1,
          ch: explanation.start_column,
        },
        end: {
          line: explanation.end_line - 1,
          ch: explanation.end_column,
        },
      }
    : null;
}
