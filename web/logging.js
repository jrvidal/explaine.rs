export const log = self.__PRODUCTION__
  ? () => {}
  : (...args) => console.log(...args);
export const logInfo = self.__PRODUCTION__
  ? () => {}
  : (...args) => console.info(...args);
export const logError = (...args) => console.error(...args);

if (!self.__PRODUCTION__) {
  self.NATIVE_LOGGING = true;

  self.logWasm = (arg) => {
    if (self.NATIVE_LOGGING) {
      console.warn(arg);
    }
  };
}
