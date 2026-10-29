// Project logger. Use this instead of console.log outside tests.

export type LogLevel = 'debug' | 'info' | 'warn' | 'error';

export interface LogEntry {
  level: LogLevel;
  message: string;
  data?: unknown;
}

export interface LogSink {
  write(entry: LogEntry): void;
}

export interface Logger {
  debug(message: string, data?: unknown): void;
  info(message: string, data?: unknown): void;
  warn(message: string, data?: unknown): void;
  error(message: string, data?: unknown): void;
}

const LEVEL_ORDER: Record<LogLevel, number> = { debug: 0, info: 1, warn: 2, error: 3 };

export function createLogger(sink: LogSink, minLevel: LogLevel = 'info'): Logger {
  const log = (level: LogLevel) => (message: string, data?: unknown) => {
    if (LEVEL_ORDER[level] < LEVEL_ORDER[minLevel]) return;
    sink.write(data === undefined ? { level, message } : { level, message, data });
  };
  return { debug: log('debug'), info: log('info'), warn: log('warn'), error: log('error') };
}

export function createMemorySink(): LogSink & { entries: LogEntry[] } {
  const entries: LogEntry[] = [];
  return { entries, write: (entry) => entries.push(entry) };
}
