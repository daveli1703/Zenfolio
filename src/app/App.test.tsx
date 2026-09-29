import { render, screen } from '@testing-library/react';
import { expect, test } from 'vitest';
import { App } from './App';

test('the offline entry screen renders without a native bridge or account', () => {
  render(<App />);
  expect(screen.getByRole('main')).toBeInTheDocument();
  expect(screen.getByRole('heading', { level: 1 })).toHaveTextContent(
    'A little more room for life.',
  );
  expect(screen.getByText(/No account required/)).toBeInTheDocument();
});
