export type Location = CodeMirror.Position;

export type MissingHint = {
  code: string;
  location: Location;
};

export type CompilationState = 0 | 1 | 2;
export const PENDING: CompilationState = 0;
export const SUCCESS: CompilationState = 1;
export const ERROR: CompilationState = 2;
