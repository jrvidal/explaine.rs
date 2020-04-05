export const log = self.__PRODUCTION__
  ? () => {}
  : (...args) => console.log(...args);
export const logInfo = self.__PRODUCTION__
  ? () => {}
  : (...args) => console.info(...args);
export const logError = (...args) => console.error(...args);
