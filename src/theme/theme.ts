import { createTheme } from '@mantine/core';

export const appTheme = createTheme({
  primaryColor: 'dark',
  defaultRadius: 'sm',
  fontFamily: '"Segoe UI", "Hiragino Sans", "Yu Gothic UI", "Meiryo", sans-serif',
  fontFamilyMonospace: 'Consolas, "Courier New", monospace',
  headings: {
    fontFamily: '"Segoe UI", "Hiragino Sans", "Yu Gothic UI", "Meiryo", sans-serif',
    fontWeight: '600',
  },
  defaultGradient: undefined,
  components: {
    Paper: {
      defaultProps: {
        radius: 'md',
        withBorder: true,
      },
    },
    Card: {
      defaultProps: {
        radius: 'md',
        withBorder: true,
      },
    },
    Text: {
      defaultProps: {
        size: 'sm',
      },
    },
    Button: {
      defaultProps: {
        radius: 'sm',
      },
    },
  },
});
