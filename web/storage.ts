import { reportError } from "./logging";

export const UNKNOWN = { __sentinel: true };

export function getFromStorage(key: string) {
  try {
    return localStorage.getItem(key);
  } catch (error) {
    reportStorageError(error);
    return UNKNOWN;
  }
}

export function setInStorage(key: string, value: any) {
  try {
    localStorage.setItem(key, value);
  } catch (error) {
    reportStorageError(error);
  }
}

function reportStorageError(error: Error) {
  reportError("storage", {
    raw: error,
    message: error.message,
    stack: error.stack,
  });
}
