const __ANALYTICS_URL__ = self.__ANALYTICS_URL__;

export const log = self.__PRODUCTION__
  ? () => {}
  : (...args) => console.log(...args);
export const logInfo = self.__PRODUCTION__
  ? () => {}
  : (...args) => console.info(...args);

export const reportHit = __ANALYTICS_URL__
  ? () => fetch(__ANALYTICS_URL__, { method: "POST" })
  : () => {};

export const reportError = (kind, e) => {
  const actualError = (e && e.error) || e;
  const errorData = JSON.stringify({
    kind,
    nativeError: actualError instanceof Error,
    line: e && e.lineno,
    column: e && e.colno,
    message: (e && e.message) || (actualError && actualError.message),
    filename: e && e.filename,
    stack: actualError && actualError.stack,
    raw: e,
  });

  if (__ANALYTICS_URL__) {
    fetch(__ANALYTICS_URL__, {
      method: "POST",
      body: errorData,
    });
  } else {
    console.error(JSON.parse(errorData));
  }
};

if (!self.__PRODUCTION__) {
  if (self.window != null) {
    let nativeLogging = false;
    Object.defineProperty(window, "NATIVE_LOGGING", {
      get() {
        return nativeLogging;
      },
      set(value) {
        nativeLogging = value;
        mainWorker.postMessage({
          type: "LOGGING",
          value,
        });
        secondaryWorker.postMessage({
          type: "LOGGING",
          value,
        });
      },
    });
  } else {
    self.NATIVE_LOGGING = false;

    self.logWasm = (arg) => {
      if (self.NATIVE_LOGGING) {
        console.warn(arg);
      }
    };
  }
}

export const handleLogging = (data) => {
  if (data.type === "LOGGING") {
    self.NATIVE_LOGGING = data.value;
    return true;
  }
};

self.addEventListener("error", (e) => {
  reportError(typeof window != null ? "window.onerror" : "self.onerror", e);
});
