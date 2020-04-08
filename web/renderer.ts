import { logInfo } from "./logging";

export default function renderer<S>(
  renderFn: (state: S) => void,
  state: { get: () => S; set: (state: S) => void }
) {
  let next = {};
  let nextRender: number | null = null;

  const doRender = () => {
    const newState = Object.assign({}, state.get(), next);
    const prevState = state.get();
    state.set(newState);
    next = {};
    nextRender = null;
    renderFn(prevState);
  };

  return (nextState: Partial<S> | ((state: S) => Partial<S>)) => {
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

export function pure<S>(fn: (state: S) => void) {
  let last: S | { _sentinel: {} } = { _sentinel: {} };

  return (arg: S) => {
    const changed = Object.keys({ ...last, ...arg }).some(
      (key) => (last as any)[key] !== (arg as any)[key]
    );
    last = arg;
    if (changed) {
      fn(arg);
    }
  };
}
