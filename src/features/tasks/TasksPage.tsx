import { ArrowDownAZ, ListFilter, Plus, Search, Settings2 } from 'lucide-react';
import { useEffect, useMemo, useRef, useState } from 'react';
import { useSearchParams } from 'react-router-dom';
import { PageHeader } from '../../components/ui/PageHeader';
import {
  taskFiltersSchema,
  type Task,
  type TaskFilters,
} from '../../lib/ipc/tasks';
import { TaskForm } from './TaskForm';
import { TaskRow } from './TaskRow';
import { TaxonomyManager } from './TaxonomyManager';
import { useProjectsQuery, useTagsQuery, useTasksQuery } from './tasksApi';

export function TasksPage() {
  const [params, setParams] = useSearchParams();
  const filters = useMemo(() => filtersFromParams(params), [params]);
  const tasks = useTasksQuery(filters);
  const projects = useProjectsQuery();
  const tags = useTagsQuery();
  const [editing, setEditing] = useState<Task | null | undefined>(undefined);
  const [managing, setManaging] = useState(false);
  const searchRef = useRef<HTMLInputElement>(null);

  useEffect(() => {
    const shortcuts = (event: KeyboardEvent) => {
      const target = event.target as HTMLElement | null;
      if (target?.matches('input, textarea, select, [contenteditable="true"]'))
        return;
      if (event.key === '/') {
        event.preventDefault();
        searchRef.current?.focus();
      }
      if (event.key.toLowerCase() === 'n') setEditing(null);
    };
    window.addEventListener('keydown', shortcuts);
    return () => window.removeEventListener('keydown', shortcuts);
  }, []);

  const updateFilter = (key: string, value: string) => {
    const next = new URLSearchParams(params);
    if (value) next.set(key, value);
    else next.delete(key);
    setParams(next, { replace: true });
  };

  const hasFilters = Boolean(
    filters.search ||
    filters.status ||
    filters.priority ||
    filters.projectId ||
    filters.tagId,
  );
  const loading = tasks.isPending || projects.isPending || tags.isPending;
  const failed = tasks.isError || projects.isError || tags.isError;

  return (
    <div className="page tasks-page">
      <div className="tasks-heading-row">
        <PageHeader
          eyebrow="Focus"
          title="Tasks"
          description="Capture, organize, and finish the work that matters."
        />
        <div className="tasks-heading-actions">
          <button
            className="button"
            type="button"
            onClick={() => setManaging(true)}
          >
            <Settings2 aria-hidden="true" /> Manage
          </button>
          <button
            className="button button-primary"
            type="button"
            onClick={() => setEditing(null)}
          >
            <Plus aria-hidden="true" /> New task
          </button>
        </div>
      </div>

      <section className="task-toolbar" aria-label="Task filters">
        <label className="search-field">
          <Search aria-hidden="true" />
          <span className="sr-only">Search tasks</span>
          <input
            ref={searchRef}
            aria-label="Search tasks"
            placeholder="Search tasks"
            value={filters.search ?? ''}
            onChange={(event) => updateFilter('search', event.target.value)}
          />
          <kbd>/</kbd>
        </label>
        <ListFilter aria-hidden="true" className="toolbar-icon" />
        <select
          aria-label="Filter by status"
          value={filters.status ?? ''}
          onChange={(event) => updateFilter('status', event.target.value)}
        >
          <option value="">All statuses</option>
          <option value="todo">To do</option>
          <option value="in_progress">In progress</option>
          <option value="done">Done</option>
        </select>
        <select
          aria-label="Filter by priority"
          value={filters.priority ?? ''}
          onChange={(event) => updateFilter('priority', event.target.value)}
        >
          <option value="">All priorities</option>
          <option value="low">Low</option>
          <option value="medium">Medium</option>
          <option value="high">High</option>
        </select>
        <select
          aria-label="Filter by project"
          value={filters.projectId ?? ''}
          onChange={(event) => updateFilter('project', event.target.value)}
        >
          <option value="">All projects</option>
          {(projects.data ?? []).map((project) => (
            <option key={project.id} value={project.id}>
              {project.name}
            </option>
          ))}
        </select>
        <select
          aria-label="Filter by tag"
          value={filters.tagId ?? ''}
          onChange={(event) => updateFilter('tag', event.target.value)}
        >
          <option value="">All tags</option>
          {(tags.data ?? []).map((tag) => (
            <option key={tag.id} value={tag.id}>
              {tag.name}
            </option>
          ))}
        </select>
        <select
          aria-label="Sort tasks"
          value={filters.sortField}
          onChange={(event) => updateFilter('sort', event.target.value)}
        >
          <option value="dueDate">Due date</option>
          <option value="createdAt">Created date</option>
          <option value="priority">Priority</option>
        </select>
        <button
          className="icon-button sort-direction"
          type="button"
          aria-label={`Sort ${filters.sortDirection === 'asc' ? 'descending' : 'ascending'}`}
          onClick={() =>
            updateFilter(
              'direction',
              filters.sortDirection === 'asc' ? 'desc' : 'asc',
            )
          }
        >
          <ArrowDownAZ
            className={filters.sortDirection === 'desc' ? 'icon-rotated' : ''}
            aria-hidden="true"
          />
        </button>
      </section>

      {loading ? (
        <TaskState
          title="Loading tasks…"
          description="Reading your local task list."
        />
      ) : null}
      {failed ? (
        <TaskState
          title="Tasks could not be loaded"
          description="Check the local database and try again."
          error
        />
      ) : null}
      {!loading && !failed && tasks.data?.length ? (
        <section className="task-list" aria-label="Tasks">
          <div className="task-count">
            {tasks.data.length} {tasks.data.length === 1 ? 'task' : 'tasks'}
          </div>
          {tasks.data.map((task) => (
            <TaskRow
              key={task.id}
              task={task}
              onEdit={() => setEditing(task)}
            />
          ))}
        </section>
      ) : null}
      {!loading && !failed && !tasks.data?.length ? (
        <TaskState
          title={
            hasFilters
              ? 'No tasks match these filters'
              : 'Start with one clear task'
          }
          description={
            hasFilters
              ? 'Adjust the search or filters to see more tasks.'
              : 'Create a task and keep your next action visible.'
          }
          action={
            hasFilters
              ? () => setParams({}, { replace: true })
              : () => setEditing(null)
          }
          actionLabel={hasFilters ? 'Clear filters' : 'Create first task'}
        />
      ) : null}

      {editing !== undefined ? (
        <TaskForm
          task={editing}
          projects={projects.data ?? []}
          tags={tags.data ?? []}
          onClose={() => setEditing(undefined)}
        />
      ) : null}
      {managing ? (
        <TaxonomyManager
          projects={projects.data ?? []}
          tags={tags.data ?? []}
          onClose={() => setManaging(false)}
        />
      ) : null}
    </div>
  );
}

function filtersFromParams(params: URLSearchParams): TaskFilters {
  return taskFiltersSchema.parse({
    search: params.get('search') || null,
    status: params.get('status') || null,
    priority: params.get('priority') || null,
    projectId: params.get('project') || null,
    tagId: params.get('tag') || null,
    sortField: params.get('sort') || 'dueDate',
    sortDirection: params.get('direction') || 'asc',
  });
}

function TaskState({
  title,
  description,
  action,
  actionLabel,
  error = false,
}: {
  title: string;
  description: string;
  action?: () => void;
  actionLabel?: string;
  error?: boolean;
}) {
  return (
    <section className={`task-state ${error ? 'task-state-error' : ''}`}>
      <h2>{title}</h2>
      <p>{description}</p>
      {action ? (
        <button
          className="button button-primary"
          type="button"
          onClick={action}
        >
          {actionLabel}
        </button>
      ) : null}
    </section>
  );
}
