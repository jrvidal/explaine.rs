import { logInfo } from "./logging";

export default function renderer(renderFn, state) {
  let next = {};
  let nextRender = null;

  const doRender = () => {
    const newState = Object.assign({}, state.get(), next);
    const prevState = state.get();
    state.set(newState);
    next = {};
    nextRender = null;
    renderFn(prevState);
  };

  return (nextState) => {
    const trueNextState =
      typeof nextState === "function"
        ? nextState({ ...state.get(), ...next })
        : nextState;

    logInfo("setState: ", trueNextState);

    Object.assign(next, trueNextState);

    if (nextRender == null) {
      nextRender = window.requestAnimationFrame(() => doRender());
    }
  };
}

export function pure(fn) {
  let last = { _sentinel: {} };

  return (arg) => {
    const changed = Object.keys({ ...last, ...arg }).some(
      (key) => last[key] !== arg[key]
    );
    last = arg;
    if (changed) {
      fn(arg);
    }
  };
}
