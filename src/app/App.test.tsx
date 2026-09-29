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
    expect(await screen.findByText('Zenfolio')).toBeInTheDocument();
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
    const tasksLink = await screen.findByRole('link', { name: 'Tasks' });
    fireEvent.click(tasksLink);
    await waitFor(() => expect(window.location.hash).toBe('#/tasks'));
    expect(screen.getByRole('heading', { level: 1 })).toHaveTextContent(
      'Tasks',
    );
    expect(tasksLink).toHaveClass('nav-item-active');
  });

  test('persists light, dark, and system theme selection', async () => {
    render(<App />);
    const theme = await screen.findByRole('combobox', { name: 'Color theme' });
    await waitFor(() => expect(theme).not.toBeDisabled());
    expect(theme).toHaveValue('system');
    expect(document.documentElement).toHaveAttribute('data-theme', 'light');
    fireEvent.change(theme, { target: { value: 'dark' } });
    await waitFor(() =>
      expect(document.documentElement).toHaveAttribute('data-theme', 'dark'),
    );
    await waitFor(() => expect(theme).not.toBeDisabled());
    fireEvent.change(theme, { target: { value: 'light' } });
    await waitFor(() =>
      expect(document.documentElement).toHaveAttribute('data-theme', 'light'),
    );
    await waitFor(() => expect(theme).not.toBeDisabled());
    fireEvent.change(theme, { target: { value: 'system' } });
    await waitFor(() => expect(theme).toHaveValue('system'));
  });
});
