// Reading and writing project files (.intracraft.json, A04 §4). Older files are upgraded by
// explicit migrations v(n) → v(n+1); files from a newer version are rejected, never guessed.
import { type Layout, type Project, SCHEMA_VERSION, projectSchema } from './schema';

export const PROJECT_FILE_EXTENSION = '.intracraft.json';

/** Migrations by the version they upgrade from. Empty while only version 1 exists. */
const MIGRATIONS: Record<number, (raw: Record<string, unknown>) => Record<string, unknown>> = {};

export type ProjectReadResult = { ok: true; project: Project } | { ok: false; error: string };

/** Parses the text of a project file, applying migrations and validating the result. */
export function readProjectFile(text: string): ProjectReadResult {
  let raw: unknown;
  try {
    raw = JSON.parse(text);
  } catch {
    return { ok: false, error: 'The file is not valid JSON.' };
  }
  if (typeof raw !== 'object' || raw === null || Array.isArray(raw)) {
    return { ok: false, error: 'The file does not contain a project.' };
  }

  let data = raw as Record<string, unknown>;
  const version = data['schemaVersion'];
  if (typeof version !== 'number' || !Number.isInteger(version)) {
    return { ok: false, error: 'The file has no schemaVersion; it is not an IntraCraft project.' };
  }
  if (version > SCHEMA_VERSION) {
    return { ok: false, error: `The file uses schema version ${version}; this version of IntraCraft reads up to ${SCHEMA_VERSION}.` };
  }
  for (let v = version; v < SCHEMA_VERSION; v++) {
    const migrate = MIGRATIONS[v];
    if (migrate === undefined) return { ok: false, error: `No migration from schema version ${v}.` };
    data = { ...migrate(data), schemaVersion: v + 1 };
  }

  const parsed = projectSchema.safeParse(data);
  if (!parsed.success) {
    const first = parsed.error.issues[0];
    const where = first === undefined ? '' : ` at ${first.path.join('.') || 'top level'}: ${first.message}`;
    return { ok: false, error: `The project file is not valid${where}.` };
  }
  return { ok: true, project: parsed.data };
}

/** Text of a project file: stable key order, two-space indentation. Results are never included. */
export function writeProjectFile(name: string, seed: number, layout: Layout): string {
  const project: Project = { schemaVersion: SCHEMA_VERSION, name, seed, layout };
  return `${JSON.stringify(projectSchema.parse(project), null, 2)}\n`;
}

/** File name for a project name: safe characters only, with the project extension. */
export function projectFileName(name: string): string {
  const base = name.trim().replace(/[^\p{L}\p{N}_\- ]+/gu, '').replace(/\s+/g, '-') || 'layout';
  return `${base}${PROJECT_FILE_EXTENSION}`;
}
