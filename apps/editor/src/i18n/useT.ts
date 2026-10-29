import { useEditorStore } from '../store';
import { type StringKey, translate } from './strings';

export function useT(): (key: StringKey) => string {
  const language = useEditorStore((s) => s.language);
  return (key) => translate(language, key);
}
