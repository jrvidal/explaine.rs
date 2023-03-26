import { defineConfig } from 'vite';

export default defineConfig(({ mode }) => ({
  root: 'web',
  define: mode === 'production' ? {
    'self.__PRODUCTION__': true,
  } : {}
}));
