// Pads Markdown table cells with spaces so columns line up in plain text (AGENTS.md rule).
// Usage: node scripts/align-md-tables.mjs [dir=docs]
import { readFileSync, readdirSync, statSync, writeFileSync } from 'node:fs';
import { join } from 'node:path';

const WIDE = /\p{Emoji_Presentation}/u;
const SEPARATOR_CELL = /^:?-+:?$/;

const root = process.argv[2] ?? 'docs';
let changed = 0;
for (const file of listMarkdown(root)) {
  const before = readFileSync(file, 'utf8');
  const after = alignTables(before);
  if (after !== before) {
    writeFileSync(file, after);
    changed++;
  }
}
process.stdout.write(`Aligned tables in ${changed} file(s) under ${root}\n`);

function alignTables(text) {
  const lines = text.split('\n');
  const out = [];
  let inFence = false;
  for (let i = 0; i < lines.length; i++) {
    const line = lines[i] ?? '';
    if (/^\s*(```|~~~)/.test(line)) inFence = !inFence;
    if (inFence || !isTableLine(line)) {
      out.push(line);
      continue;
    }
    const block = [];
    while (i < lines.length && isTableLine(lines[i] ?? '')) block.push(lines[i++] ?? '');
    i--;
    out.push(...formatTable(block));
  }
  return out.join('\n');
}

function isTableLine(line) {
  return line.trimStart().startsWith('|');
}

function formatTable(block) {
  const rows = block.map(splitRow);
  const columns = Math.max(...rows.map((r) => r.length));
  const widths = Array.from({ length: columns }, (_, c) =>
    Math.max(3, ...rows.map((r) => (SEPARATOR_CELL.test(r[c] ?? '') ? 3 : displayWidth(r[c] ?? '')))),
  );
  return rows.map((row) => {
    const cells = Array.from({ length: columns }, (_, c) => {
      const cell = row[c] ?? '';
      if (SEPARATOR_CELL.test(cell)) return separator(cell, widths[c]);
      return cell + ' '.repeat(widths[c] - displayWidth(cell));
    });
    return `| ${cells.join(' | ')} |`;
  });
}

function splitRow(line) {
  const cells = [];
  let current = '';
  let inCode = false;
  let inLink = 0;
  const body = line.trim().replace(/^\|/, '').replace(/\|$/, '');
  for (let i = 0; i < body.length; i++) {
    const ch = body[i];
    if (ch === '\\' && body[i + 1] === '|') {
      current += '\\|';
      i++;
      continue;
    }
    if (ch === '`') inCode = !inCode;
    if (!inCode && body.startsWith('[[', i)) inLink++;
    if (!inCode && body.startsWith(']]', i) && inLink > 0) inLink--;
    if (ch === '|' && !inCode && inLink === 0) {
      cells.push(current.trim());
      current = '';
      continue;
    }
    current += ch;
  }
  cells.push(current.trim());
  return cells;
}

function separator(cell, width) {
  const left = cell.startsWith(':') ? ':' : '-';
  const right = cell.endsWith(':') ? ':' : '-';
  return left + '-'.repeat(width - 2) + right;
}

function displayWidth(text) {
  let width = 0;
  for (const ch of text) width += WIDE.test(ch) ? 2 : 1;
  return width;
}

function listMarkdown(dir) {
  const files = [];
  for (const name of readdirSync(dir)) {
    if (name.startsWith('.')) continue;
    const path = join(dir, name);
    if (statSync(path).isDirectory()) files.push(...listMarkdown(path));
    else if (name.endsWith('.md')) files.push(path);
  }
  return files;
}
