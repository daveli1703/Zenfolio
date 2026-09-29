import { X } from 'lucide-react';
import {
  type FormEvent,
  type ReactNode,
  useEffect,
  useRef,
  useState,
} from 'react';
import { IpcError } from '../../lib/ipc/errors';
import {
  taskInputSchema,
  type Project,
  type Tag,
  type Task,
  type TaskInput,
} from '../../lib/ipc/tasks';
import { useTaskMutations } from './tasksApi';

type TaskFormProps = {
  task: Task | null;
  projects: Project[];
  tags: Tag[];
  onClose: () => void;
};

const emptyTask: TaskInput = {
  title: '',
  notes: null,
  dueDate: null,
  dueTime: null,
  priority: 'medium',
  status: 'todo',
  projectId: null,
  tagIds: [],
};

export function TaskForm({ task, projects, tags, onClose }: TaskFormProps) {
  const mutations = useTaskMutations();
  const mutation = task ? mutations.update : mutations.create;
  const [draft, setDraft] = useState<TaskInput>(() =>
    task
      ? {
          title: task.title,
          notes: task.notes,
          dueDate: task.dueDate,
          dueTime: task.dueTime,
          priority: task.priority,
          status: task.status,
          projectId: task.project?.id ?? null,
          tagIds: task.tags.map((tag) => tag.id),
        }
      : emptyTask,
  );
  const [fieldErrors, setFieldErrors] = useState<Record<string, string>>({});
  const [formError, setFormError] = useState('');

  const submit = (event: FormEvent) => {
    event.preventDefault();
    setFormError('');
    setFieldErrors({});
    const parsed = taskInputSchema.safeParse(draft);
    if (!parsed.success) {
      const errors: Record<string, string> = {};
      for (const issue of parsed.error.issues) {
        const field = String(issue.path[0] ?? 'form');
        errors[field] ??= issue.message;
      }
      setFieldErrors(errors);
      return;
    }
    const onError = (error: Error) => {
      if (error instanceof IpcError) {
        setFieldErrors(error.fieldErrors ?? {});
        setFormError(error.message);
      } else {
        setFormError(
          'The task could not be saved. Your changes are still here.',
        );
      }
    };
    if (task) {
      mutations.update.mutate(
        { id: task.id, input: parsed.data },
        { onSuccess: onClose, onError },
      );
    } else {
      mutations.create.mutate(parsed.data, { onSuccess: onClose, onError });
    }
  };

  const toggleTag = (id: string) =>
    setDraft((current) => ({
      ...current,
      tagIds: current.tagIds.includes(id)
        ? current.tagIds.filter((tagId) => tagId !== id)
        : [...current.tagIds, id],
    }));
  const selectableProjects = projects.filter(
    (project) => !project.archived || project.id === task?.project?.id,
  );

  return (
    <Modal title={task ? 'Edit task' : 'Create task'} onClose={onClose}>
      <form className="task-form" onSubmit={submit}>
        <FormField label="Title" error={fieldErrors.title}>
          <input
            autoFocus
            value={draft.title}
            onChange={(event) =>
              setDraft({ ...draft, title: event.target.value })
            }
          />
        </FormField>
        <FormField label="Notes" error={fieldErrors.notes}>
          <textarea
            rows={4}
            value={draft.notes ?? ''}
            onChange={(event) =>
              setDraft({ ...draft, notes: event.target.value || null })
            }
          />
        </FormField>
        <div className="task-form-grid">
          <FormField label="Due date" error={fieldErrors.dueDate}>
            <input
              type="date"
              value={draft.dueDate ?? ''}
              onChange={(event) =>
                setDraft({
                  ...draft,
                  dueDate: event.target.value || null,
                  dueTime: event.target.value ? draft.dueTime : null,
                })
              }
            />
          </FormField>
          <FormField label="Due time" error={fieldErrors.dueTime}>
            <input
              type="time"
              disabled={!draft.dueDate}
              value={draft.dueTime ?? ''}
              onChange={(event) =>
                setDraft({ ...draft, dueTime: event.target.value || null })
              }
            />
          </FormField>
          <FormField label="Priority" error={fieldErrors.priority}>
            <select
              value={draft.priority}
              onChange={(event) =>
                setDraft({
                  ...draft,
                  priority: event.target.value as TaskInput['priority'],
                })
              }
            >
              <option value="low">Low</option>
              <option value="medium">Medium</option>
              <option value="high">High</option>
            </select>
          </FormField>
          <FormField label="Status" error={fieldErrors.status}>
            <select
              value={draft.status}
              onChange={(event) =>
                setDraft({
                  ...draft,
                  status: event.target.value as TaskInput['status'],
                })
              }
            >
              <option value="todo">To do</option>
              <option value="in_progress">In progress</option>
              <option value="done">Done</option>
            </select>
          </FormField>
        </div>
        <FormField label="Project" error={fieldErrors.projectId}>
          <select
            value={draft.projectId ?? ''}
            onChange={(event) =>
              setDraft({ ...draft, projectId: event.target.value || null })
            }
          >
            <option value="">No project</option>
            {selectableProjects.map((project) => (
              <option key={project.id} value={project.id}>
                {project.name}
                {project.archived ? ' (archived)' : ''}
              </option>
            ))}
          </select>
        </FormField>
        <fieldset className="tag-picker">
          <legend>Tags</legend>
          {tags.length ? (
            <div className="tag-options">
              {tags.map((tag) => (
                <label key={tag.id}>
                  <input
                    type="checkbox"
                    checked={draft.tagIds.includes(tag.id)}
                    onChange={() => toggleTag(tag.id)}
                  />
                  <span>{tag.name}</span>
                </label>
              ))}
            </div>
          ) : (
            <p className="field-hint">
              Create tags from Manage projects & tags.
            </p>
          )}
          {fieldErrors.tagIds ? (
            <span className="field-error">{fieldErrors.tagIds}</span>
          ) : null}
        </fieldset>
        {formError ? (
          <p className="form-message error-message" role="alert">
            {formError}
          </p>
        ) : null}
        <div className="form-actions task-form-actions">
          <button className="button" type="button" onClick={onClose}>
            Cancel
          </button>
          <button
            className="button button-primary"
            disabled={mutation.isPending}
          >
            {mutation.isPending
              ? 'Saving…'
              : task
                ? 'Save task'
                : 'Create task'}
          </button>
        </div>
      </form>
    </Modal>
  );
}

export function Modal({
  title,
  onClose,
  children,
}: {
  title: string;
  onClose: () => void;
  children: ReactNode;
}) {
  const dialogRef = useRef<HTMLElement>(null);
  useEffect(() => {
    const previouslyFocused = document.activeElement as HTMLElement | null;
    const handleKeys = (event: KeyboardEvent) => {
      if (event.key === 'Escape') {
        onClose();
        return;
      }
      if (event.key !== 'Tab') return;
      const focusable = dialogRef.current?.querySelectorAll<HTMLElement>(
        'button:not([disabled]), input:not([disabled]), select:not([disabled]), textarea:not([disabled])',
      );
      if (!focusable?.length) return;
      const first = focusable[0];
      const last = focusable[focusable.length - 1];
      if (event.shiftKey && document.activeElement === first) {
        event.preventDefault();
        last.focus();
      } else if (!event.shiftKey && document.activeElement === last) {
        event.preventDefault();
        first.focus();
      }
    };
    window.addEventListener('keydown', handleKeys);
    return () => {
      window.removeEventListener('keydown', handleKeys);
      previouslyFocused?.focus();
    };
  }, [onClose]);
  return (
    <div className="modal-backdrop" role="presentation">
      <section
        ref={dialogRef}
        className="task-drawer"
        role="dialog"
        aria-modal="true"
        aria-labelledby="task-dialog-title"
      >
        <header className="drawer-header">
          <h2 id="task-dialog-title">{title}</h2>
          <button
            className="icon-button"
            type="button"
            onClick={onClose}
            aria-label="Close"
          >
            <X aria-hidden="true" />
          </button>
        </header>
        {children}
      </section>
    </div>
  );
}

function FormField({
  label,
  error,
  children,
}: {
  label: string;
  error?: string;
  children: ReactNode;
}) {
  return (
    <label className="form-field">
      <span>{label}</span>
      {children}
      {error ? <span className="field-error">{error}</span> : null}
    </label>
  );
}
