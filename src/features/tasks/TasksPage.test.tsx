import { invoke } from '@tauri-apps/api/core';
import { fireEvent, render, screen, waitFor } from '@testing-library/react';
import { beforeEach, describe, expect, test, vi } from 'vitest';
import { App } from '../../app/App';

const project = {
  id: 'project-1',
  name: 'Work',
  color: null,
  archived: false,
  createdAt: 'created',
  updatedAt: 'updated',
};
const tag = {
  id: 'tag-1',
  name: 'Focus',
  color: null,
  createdAt: 'created',
  updatedAt: 'updated',
};
const settings = {
  currencyCode: 'VND',
  currencyExponent: 0,
  applicationTimezone: 'Asia/Ho_Chi_Minh',
  dateFormat: 'DD/MM/YYYY',
  timeFormat: '24h',
  firstWeekday: 1,
  theme: 'system',
  createdAt: 'created',
  updatedAt: 'updated',
};

function task(id: string, title: string) {
  return {
    id,
    title,
    notes: 'Notes',
    dueDate: '2026-10-01',
    dueTime: '09:30',
    priority: 'high',
    status: 'todo',
    project,
    tags: [tag],
    completedAt: null,
    createdAt: 'created',
    updatedAt: 'updated',
  };
}

function installHandler(initialTasks: ReturnType<typeof task>[] = []) {
  let tasks = [...initialTasks];
  vi.mocked(invoke).mockImplementation(async (command, args) => {
    if (command === 'get_startup_status')
      return {
        state: 'ready',
        databasePath: 'C:/Zenfolio/development/productivity.sqlite3',
        environment: 'development',
      };
    if (command === 'get_settings') return settings;
    if (command === 'list_projects') return [project];
    if (command === 'list_tags') return [tag];
    if (command === 'list_tasks') {
      const filters =
        (args as { filters?: Record<string, string | null> })?.filters ?? {};
      return tasks.filter(
        (item) =>
          (!filters.search ||
            item.title
              .toLowerCase()
              .includes(String(filters.search).toLowerCase())) &&
          (!filters.status || item.status === filters.status) &&
          (!filters.priority || item.priority === filters.priority) &&
          (!filters.projectId || item.project?.id === filters.projectId) &&
          (!filters.tagId ||
            item.tags.some((value) => value.id === filters.tagId)),
      );
    }
    if (command === 'create_task') {
      const input = (args as { input: Record<string, unknown> }).input;
      const created = {
        ...task('created-task', String(input.title)),
        ...input,
        project: input.projectId ? project : null,
        tags: (input.tagIds as string[]).includes(tag.id) ? [tag] : [],
        completedAt: input.status === 'done' ? 'completed' : null,
      };
      tasks = [...tasks, created as ReturnType<typeof task>];
      return created;
    }
    if (command === 'update_task') {
      const input = (args as { input: Record<string, unknown> }).input;
      const id = (args as { id: string }).id;
      const updated = {
        ...tasks.find((item) => item.id === id)!,
        ...input,
        project: input.projectId ? project : null,
        tags: (input.tagIds as string[]).includes(tag.id) ? [tag] : [],
      };
      tasks = tasks.map((item) =>
        item.id === id ? (updated as ReturnType<typeof task>) : item,
      );
      return updated;
    }
    if (command === 'set_task_status') {
      const input = (args as { input: { id: string; status: string } }).input;
      const updated = {
        ...tasks.find((item) => item.id === input.id)!,
        status: input.status,
        completedAt: input.status === 'done' ? 'completed' : null,
      };
      tasks = tasks.map((item) =>
        item.id === input.id ? (updated as ReturnType<typeof task>) : item,
      );
      return updated;
    }
    if (command === 'delete_task') {
      tasks = tasks.filter(
        (item) => item.id !== (args as { input: { id: string } }).input.id,
      );
      return null;
    }
    throw new Error(`Unexpected test command: ${command}`);
  });
}

describe('Tasks page', () => {
  beforeEach(() => {
    window.location.hash = '#/tasks';
    installHandler();
  });

  test('shows a useful empty state and creates, edits, completes, and reopens a task', async () => {
    render(<App />);
    expect(
      await screen.findByText('Start with one clear task'),
    ).toBeInTheDocument();
    fireEvent.click(screen.getByRole('button', { name: 'New task' }));
    fireEvent.change(screen.getByLabelText('Title'), {
      target: { value: 'Prepare review' },
    });
    fireEvent.change(screen.getByLabelText('Due date'), {
      target: { value: '2026-10-01' },
    });
    fireEvent.change(screen.getByLabelText('Due time'), {
      target: { value: '09:30' },
    });
    fireEvent.change(screen.getByLabelText('Priority'), {
      target: { value: 'high' },
    });
    fireEvent.change(screen.getByLabelText('Project'), {
      target: { value: project.id },
    });
    fireEvent.click(screen.getByLabelText('Focus'));
    fireEvent.click(screen.getByRole('button', { name: 'Create task' }));
    expect(
      await screen.findByRole('heading', { name: 'Prepare review' }),
    ).toBeInTheDocument();
    expect(invoke).toHaveBeenCalledWith(
      'create_task',
      expect.objectContaining({
        input: expect.objectContaining({
          projectId: project.id,
          tagIds: [tag.id],
          dueTime: '09:30',
        }),
      }),
    );

    fireEvent.click(
      screen.getByRole('button', { name: 'Edit Prepare review' }),
    );
    fireEvent.change(screen.getByLabelText('Title'), {
      target: { value: 'Prepare final review' },
    });
    fireEvent.change(screen.getByLabelText('Due time'), {
      target: { value: '' },
    });
    fireEvent.click(screen.getByRole('button', { name: 'Save task' }));
    expect(
      await screen.findByRole('heading', { name: 'Prepare final review' }),
    ).toBeInTheDocument();

    fireEvent.click(
      screen.getByRole('button', { name: 'Complete Prepare final review' }),
    );
    expect(
      await screen.findByRole('button', {
        name: 'Reopen Prepare final review',
      }),
    ).toBeInTheDocument();
    fireEvent.click(
      screen.getByRole('button', { name: 'Reopen Prepare final review' }),
    );
    expect(
      await screen.findByRole('button', {
        name: 'Complete Prepare final review',
      }),
    ).toBeInTheDocument();
  });

  test('sends search, combined filters, and each sorting mode through the typed query', async () => {
    installHandler([task('one', 'Quarterly report')]);
    render(<App />);
    await screen.findByRole('heading', { name: 'Quarterly report' });
    fireEvent.change(screen.getByLabelText('Search tasks'), {
      target: { value: 'Quarterly' },
    });
    fireEvent.change(screen.getByLabelText('Filter by status'), {
      target: { value: 'todo' },
    });
    fireEvent.change(screen.getByLabelText('Filter by priority'), {
      target: { value: 'high' },
    });
    fireEvent.change(screen.getByLabelText('Filter by project'), {
      target: { value: project.id },
    });
    fireEvent.change(screen.getByLabelText('Filter by tag'), {
      target: { value: tag.id },
    });
    for (const mode of ['createdAt', 'priority', 'dueDate'])
      fireEvent.change(screen.getByLabelText('Sort tasks'), {
        target: { value: mode },
      });
    fireEvent.click(screen.getByRole('button', { name: 'Sort descending' }));
    await waitFor(() =>
      expect(invoke).toHaveBeenCalledWith('list_tasks', {
        filters: expect.objectContaining({
          search: 'Quarterly',
          status: 'todo',
          priority: 'high',
          projectId: project.id,
          tagId: tag.id,
          sortField: 'dueDate',
          sortDirection: 'desc',
        }),
      }),
    );
  });

  test('preserves the draft and shows the safe error when creation fails', async () => {
    const normal = vi.mocked(invoke).getMockImplementation()!;
    vi.mocked(invoke).mockImplementation(async (command, args) => {
      if (command === 'create_task')
        throw {
          code: 'WRITE_FAILED',
          message: 'Zenfolio could not write to local storage.',
        };
      return normal(command, args);
    });
    render(<App />);
    await screen.findByText('Start with one clear task');
    fireEvent.click(screen.getByRole('button', { name: 'New task' }));
    fireEvent.change(screen.getByLabelText('Title'), {
      target: { value: 'Keep this draft' },
    });
    fireEvent.click(screen.getByRole('button', { name: 'Create task' }));
    expect(await screen.findByRole('alert')).toHaveTextContent(
      'could not write',
    );
    expect(screen.getByLabelText('Title')).toHaveValue('Keep this draft');
  });
});
