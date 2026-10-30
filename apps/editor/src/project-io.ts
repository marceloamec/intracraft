// Saving and opening project files in the browser: a download for "Save" (and for exported
// results), a file picker for "Open". The content itself is written and checked by core-model (project-file.ts).
import { PROJECT_FILE_EXTENSION, projectFileName } from '@intracraft/core-model';
import { cameraActions } from './components/Viewport';
import { registry, useLayoutStore } from './layout-store';
import { resultsCsv } from './results-export';
import { useSimStore } from './sim-store';

export function saveProject(): void {
  const { projectName, projectText, setStatus } = useLayoutStore.getState();
  const name = projectFileName(projectName);
  download(name, projectText(), 'application/json');
  setStatus({ kind: 'saved', message: name, issues: [] });
}

/** Statistics of the current run as CSV, next to (never inside) the project file. */
export function exportResults(): void {
  const { projectName, layout, setStatus } = useLayoutStore.getState();
  const { statistics, time } = useSimStore.getState();
  const name = projectFileName(projectName).replace(PROJECT_FILE_EXTENSION, '-results.csv');
  download(name, resultsCsv(layout, statistics, registry, time), 'text/csv');
  setStatus({ kind: 'saved', message: name, issues: [] });
}

function download(fileName: string, text: string, type: string): void {
  const url = URL.createObjectURL(new Blob([text], { type }));
  const link = document.createElement('a');
  link.href = url;
  link.download = fileName;
  link.click();
  URL.revokeObjectURL(url);
}

export function openProject(): void {
  const input = document.createElement('input');
  input.type = 'file';
  input.accept = `${PROJECT_FILE_EXTENSION},.json,application/json`;
  input.onchange = async () => {
    const file = input.files?.[0];
    if (file === undefined) return;
    const { openProject: load, setStatus } = useLayoutStore.getState();
    const error = load(await file.text());
    // Let the new objects render once, then show all of them.
    if (error === null) requestAnimationFrame(() => requestAnimationFrame(() => cameraActions.frameAll?.()));
    setStatus(
      error === null
        ? { kind: 'opened', message: file.name, issues: [] }
        : { message: '', issues: [{ rule: 'projectFile', severity: 'error', message: `${file.name}: ${error}` }] },
    );
  };
  input.click();
}
