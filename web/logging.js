const __ANALYTICS_URL__ = self.__ANALYTICS_URL__;

export const log = self.__PRODUCTION__
  ? () => {}
  : (...args) => console.log(...args);
export const logInfo = self.__PRODUCTION__
  ? () => {}
  : (...args) => console.info(...args);
export const logError = (...args) => console.error(...args);

export const reportHit = __ANALYTICS_URL__
  ? () => fetch(__ANALYTICS_URL__, { method: "POST" })
  : () => {};

export const reportError = __ANALYTICS_URL__
  ? (data) =>
      fetch(__ANALYTICS_URL__, {
        method: "POST",
        body: JSON.stringify(data),
      })
  : logError;

if (!self.__PRODUCTION__) {
  self.NATIVE_LOGGING = true;

  self.logWasm = (arg) => {
    if (self.NATIVE_LOGGING) {
      console.warn(arg);
    }
  };
}

self.addEventListener("error", (e) => {
  reportError({
    line: e && e.lineno,
    column: e && e.colno,
    message: e && e.message,
    filename: e && e.filename,
    stack: e && e.error && e.error.stack,
    raw: e,
  });
});
