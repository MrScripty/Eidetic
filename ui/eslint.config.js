import js from '@eslint/js';
import prettier from 'eslint-config-prettier';
import svelte from 'eslint-plugin-svelte';
import globals from 'globals';
import tseslint from 'typescript-eslint';

export default tseslint.config(
  {
    ignores: ['build/**', 'dist/**', '.svelte-kit/**', 'node_modules/**'],
  },
  js.configs.recommended,
  ...svelte.configs['flat/recommended'],
  {
    files: ['src/**/*.{js,ts}', 'src/**/*.svelte.ts'],
    plugins: {
      '@typescript-eslint': tseslint.plugin,
    },
    languageOptions: {
      parser: tseslint.parser,
      globals: {
        ...globals.browser,
        ...globals.node,
        $bindable: 'readonly',
        $derived: 'readonly',
        $effect: 'readonly',
        $props: 'readonly',
        $state: 'readonly',
      },
    },
    rules: {
      'no-undef': 'off',
      'no-unused-vars': 'off',
      '@typescript-eslint/no-unused-vars': [
        'error',
        { argsIgnorePattern: '^_', varsIgnorePattern: '^_' },
      ],
      '@typescript-eslint/no-explicit-any': 'error',
    },
  },
  {
    files: ['src/**/*.svelte'],
    languageOptions: {
      globals: {
        ...globals.browser,
        ...globals.node,
      },
      parserOptions: {
        parser: tseslint.parser,
      },
    },
    rules: {
      'no-undef': 'off',
      'no-unused-vars': 'off',
      'svelte/no-at-html-tags': 'off',
      'svelte/prefer-svelte-reactivity': 'off',
      'svelte/prefer-writable-derived': 'off',
      'svelte/require-each-key': 'off',
      'svelte/valid-compile': 'error',
    },
  },
  prettier,
);
