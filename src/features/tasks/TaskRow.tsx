import { CalendarDays, Check, Pencil, Trash2, Undo2 } from 'lucide-react';
import { useState } from 'react';
import { formatDateOnly, formatWallTime } from '../../lib/dates';
import { IpcError } from '../../lib/ipc/errors';
import { type Task, type TaskStatus } from '../../lib/ipc/tasks';
import { useSettingsQuery } from '../settings/settingsApi';
import { useTaskMutations } from './tasksApi';

export function TaskRow({ task, onEdit }: { task: Task; onEdit: () => void }) {
  const mutations = useTaskMutations();
  const settings = useSettingsQuery();
  const done = task.status === 'done';
  const [error, setError] = useState('');
  const showError = (value: Error) =>
    setError(
      value instanceof IpcError
        ? value.message
        : 'The task could not be changed.',
    );
  const changeStatus = (status: TaskStatus) => {
    setError('');
    mutations.status.mutate({ id: task.id, status }, { onError: showError });
  };
  const remove = () => {
    if (window.confirm(`Delete “${task.title}”? This cannot be undone.`)) {
      setError('');
      mutations.remove.mutate(task.id, { onError: showError });
    }
  };
  return (
    <article className={`task-row ${done ? 'task-row-done' : ''}`}>
      <button
        className="completion-button"
        type="button"
        aria-label={done ? `Reopen ${task.title}` : `Complete ${task.title}`}
        disabled={mutations.status.isPending}
        onClick={() => changeStatus(done ? 'todo' : 'done')}
      >
        {done ? <Undo2 aria-hidden="true" /> : <Check aria-hidden="true" />}
      </button>
      <div className="task-row-content">
        <div className="task-row-title-line">
          <h2>{task.title}</h2>
          <span className={`badge priority-${task.priority}`}>
            {task.priority}
          </span>
        </div>
        {task.notes ? <p className="task-notes">{task.notes}</p> : null}
        <div className="task-meta">
          {task.dueDate ? (
            <span>
              <CalendarDays aria-hidden="true" />
              {settings.data
                ? formatDateOnly(task.dueDate, settings.data.dateFormat)
                : task.dueDate}
              {task.dueTime
                ? ` at ${settings.data ? formatWallTime(task.dueTime, settings.data.timeFormat) : task.dueTime}`
                : ''}
            </span>
          ) : null}
          {task.project ? (
            <span className="project-label">
              <i style={{ background: task.project.color ?? undefined }} />
              {task.project.name}
            </span>
          ) : null}
          {task.tags.map((tag) => (
            <span className="tag-label" key={tag.id}>
              {tag.name}
            </span>
          ))}
        </div>
        {error ? (
          <p className="task-row-error" role="alert">
            {error}
          </p>
        ) : null}
      </div>
      <div className="task-row-actions">
        <select
          aria-label={`Status for ${task.title}`}
          value={task.status}
          disabled={mutations.status.isPending}
          onChange={(event) => changeStatus(event.target.value as TaskStatus)}
        >
          <option value="todo">To do</option>
          <option value="in_progress">In progress</option>
          <option value="done">Done</option>
        </select>
        <button
          className="icon-button"
          type="button"
          onClick={onEdit}
          aria-label={`Edit ${task.title}`}
        >
          <Pencil aria-hidden="true" />
        </button>
        <button
          className="icon-button danger-button"
          type="button"
          onClick={remove}
          aria-label={`Delete ${task.title}`}
        >
          <Trash2 aria-hidden="true" />
        </button>
      </div>
    </article>
  );
}
