import { Archive, ArchiveRestore, Pencil, Trash2 } from 'lucide-react';
import { type FormEvent, useState } from 'react';
import { IpcError } from '../../lib/ipc/errors';
import { type Project, type Tag } from '../../lib/ipc/tasks';
import { Modal } from './TaskForm';
import { useProjectMutations, useTagMutations } from './tasksApi';

export function TaxonomyManager({
  projects,
  tags,
  onClose,
}: {
  projects: Project[];
  tags: Tag[];
  onClose: () => void;
}) {
  const projectsApi = useProjectMutations();
  const tagsApi = useTagMutations();
  const [projectName, setProjectName] = useState('');
  const [projectColor, setProjectColor] = useState('#17735a');
  const [tagName, setTagName] = useState('');
  const [tagColor, setTagColor] = useState('#17735a');
  const [error, setError] = useState('');
  const showError = (value: Error) =>
    setError(
      value instanceof IpcError
        ? value.message
        : 'The change could not be saved.',
    );
  const addProject = (event: FormEvent) => {
    event.preventDefault();
    if (!projectName.trim()) return;
    setError('');
    projectsApi.create.mutate(
      { name: projectName, color: projectColor },
      { onSuccess: () => setProjectName(''), onError: showError },
    );
  };
  const addTag = (event: FormEvent) => {
    event.preventDefault();
    if (!tagName.trim()) return;
    setError('');
    tagsApi.create.mutate(
      { name: tagName, color: tagColor },
      { onSuccess: () => setTagName(''), onError: showError },
    );
  };
  const renameProject = (project: Project) => {
    const name = window.prompt('Project name', project.name);
    if (name?.trim() && name.trim() !== project.name)
      projectsApi.update.mutate(
        { id: project.id, input: { name, color: project.color } },
        { onError: showError },
      );
  };
  const renameTag = (tag: Tag) => {
    const name = window.prompt('Tag name', tag.name);
    if (name?.trim() && name.trim() !== tag.name)
      tagsApi.update.mutate(
        { id: tag.id, input: { name, color: tag.color } },
        { onError: showError },
      );
  };
  return (
    <Modal title="Manage projects & tags" onClose={onClose}>
      <div className="taxonomy-sections">
        <section>
          <h3>Projects</h3>
          <form className="inline-create" onSubmit={addProject}>
            <input
              aria-label="New project name"
              placeholder="New project"
              value={projectName}
              onChange={(event) => setProjectName(event.target.value)}
            />
            <input
              className="color-input"
              type="color"
              aria-label="New project color"
              value={projectColor}
              onChange={(event) => setProjectColor(event.target.value)}
            />
            <button className="button button-primary">Add</button>
          </form>
          <div className="taxonomy-list">
            {projects.map((project) => (
              <div className="taxonomy-row" key={project.id}>
                <span>
                  {project.name}
                  {project.archived ? <small>Archived</small> : null}
                </span>
                <div>
                  <input
                    className="color-input"
                    type="color"
                    aria-label={`Color for project ${project.name}`}
                    value={project.color ?? '#17735a'}
                    onChange={(event) =>
                      projectsApi.update.mutate(
                        {
                          id: project.id,
                          input: {
                            name: project.name,
                            color: event.target.value,
                          },
                        },
                        { onError: showError },
                      )
                    }
                  />
                  <button
                    className="icon-button"
                    type="button"
                    aria-label={`Edit project ${project.name}`}
                    onClick={() => renameProject(project)}
                  >
                    <Pencil aria-hidden="true" />
                  </button>
                  <button
                    className="icon-button"
                    type="button"
                    aria-label={`${project.archived ? 'Unarchive' : 'Archive'} project ${project.name}`}
                    onClick={() =>
                      projectsApi.archive.mutate(
                        { id: project.id, archived: !project.archived },
                        { onError: showError },
                      )
                    }
                  >
                    {project.archived ? (
                      <ArchiveRestore aria-hidden="true" />
                    ) : (
                      <Archive aria-hidden="true" />
                    )}
                  </button>
                  <button
                    className="icon-button danger-button"
                    type="button"
                    aria-label={`Delete project ${project.name}`}
                    onClick={() => {
                      if (
                        window.confirm(
                          `Delete “${project.name}”? Its tasks will remain.`,
                        )
                      )
                        projectsApi.remove.mutate(project.id, {
                          onError: showError,
                        });
                    }}
                  >
                    <Trash2 aria-hidden="true" />
                  </button>
                </div>
              </div>
            ))}
            {!projects.length ? (
              <p className="field-hint">No projects yet.</p>
            ) : null}
          </div>
        </section>
        <section>
          <h3>Tags</h3>
          <form className="inline-create" onSubmit={addTag}>
            <input
              aria-label="New tag name"
              placeholder="New tag"
              value={tagName}
              onChange={(event) => setTagName(event.target.value)}
            />
            <input
              className="color-input"
              type="color"
              aria-label="New tag color"
              value={tagColor}
              onChange={(event) => setTagColor(event.target.value)}
            />
            <button className="button button-primary">Add</button>
          </form>
          <div className="taxonomy-list">
            {tags.map((tag) => (
              <div className="taxonomy-row" key={tag.id}>
                <span>{tag.name}</span>
                <div>
                  <input
                    className="color-input"
                    type="color"
                    aria-label={`Color for tag ${tag.name}`}
                    value={tag.color ?? '#17735a'}
                    onChange={(event) =>
                      tagsApi.update.mutate(
                        {
                          id: tag.id,
                          input: { name: tag.name, color: event.target.value },
                        },
                        { onError: showError },
                      )
                    }
                  />
                  <button
                    className="icon-button"
                    type="button"
                    aria-label={`Edit tag ${tag.name}`}
                    onClick={() => renameTag(tag)}
                  >
                    <Pencil aria-hidden="true" />
                  </button>
                  <button
                    className="icon-button danger-button"
                    type="button"
                    aria-label={`Delete tag ${tag.name}`}
                    onClick={() => {
                      if (
                        window.confirm(
                          `Delete “${tag.name}”? Tasks will remain.`,
                        )
                      )
                        tagsApi.remove.mutate(tag.id, { onError: showError });
                    }}
                  >
                    <Trash2 aria-hidden="true" />
                  </button>
                </div>
              </div>
            ))}
            {!tags.length ? <p className="field-hint">No tags yet.</p> : null}
          </div>
        </section>
      </div>
      {error ? (
        <p className="form-message error-message" role="alert">
          {error}
        </p>
      ) : null}
    </Modal>
  );
}
