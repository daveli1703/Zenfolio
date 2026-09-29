import { fireEvent, render, screen, waitFor } from '@testing-library/react';
import { beforeEach, describe, expect, test } from 'vitest';
import { App } from './App';

describe('Zenfolio application shell', () => {
  beforeEach(() => {
    window.location.hash = '';
    delete document.documentElement.dataset.theme;
  });

  test('renders the brand and all seven navigation destinations', async () => {
    render(<App />);
    expect(screen.getByText('Zenfolio')).toBeInTheDocument();
    expect(
      screen.getByRole('navigation', { name: 'Primary navigation' }),
    ).toBeInTheDocument();
    expect(screen.getAllByRole('link')).toHaveLength(7);
    await waitFor(() =>
      expect(screen.getByRole('heading', { level: 1 })).toHaveTextContent(
        'Dashboard',
      ),
    );
  });

  test('navigates with hashes and updates the active destination', async () => {
    render(<App />);
    const tasksLink = screen.getByRole('link', { name: 'Tasks' });
    fireEvent.click(tasksLink);
    await waitFor(() => expect(window.location.hash).toBe('#/tasks'));
    expect(screen.getByRole('heading', { level: 1 })).toHaveTextContent(
      'Tasks',
    );
    expect(tasksLink).toHaveClass('nav-item-active');
  });

  test('allows light, dark, and system theme selection', () => {
    render(<App />);
    const theme = screen.getByRole('combobox', { name: 'Color theme' });
    expect(theme).toHaveValue('system');
    expect(document.documentElement).toHaveAttribute('data-theme', 'light');
    fireEvent.change(theme, { target: { value: 'dark' } });
    expect(document.documentElement).toHaveAttribute('data-theme', 'dark');
    fireEvent.change(theme, { target: { value: 'light' } });
    expect(document.documentElement).toHaveAttribute('data-theme', 'light');
    fireEvent.change(theme, { target: { value: 'system' } });
    expect(theme).toHaveValue('system');
  });
});
