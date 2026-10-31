import js from '@eslint/js';
import reactHooks from 'eslint-plugin-react-hooks';
import globals from 'globals';
import tseslint from 'typescript-eslint';

export default tseslint.config(
  { ignores: ['**/dist/**', '**/node_modules/**', 'docs/**', '.claude/**', 'platform/**', 'spikes/**/target/**'] },
  js.configs.recommended,
  ...tseslint.configs.strict,
  {
    languageOptions: { globals: { ...globals.browser, ...globals.node } },
    rules: {
      'no-console': 'error',
    },
  },
  {
    // Engines must stay deterministic and UI-free.
    files: ['packages/sim-des/**', 'packages/sim-physics/**', 'packages/control/**'],
    rules: {
      'no-restricted-properties': [
        'error',
        { object: 'Math', property: 'random', message: 'Use the seeded RNG from @intracraft/core-model.' },
      ],
      'no-restricted-imports': [
        'error',
        { patterns: ['react', 'react-dom', '@react-three/*', 'three'] },
      ],
    },
  },
  {
    files: ['apps/editor/**/*.{ts,tsx}'],
    plugins: { 'react-hooks': reactHooks },
    rules: reactHooks.configs.recommended.rules,
  },
  {
    // Command-line scripts report to the terminal.
    files: ['**/*.test.ts', 'scripts/**', 'spikes/**/*.mjs'],
    rules: { 'no-console': 'off' },
  },
);
