import rawCatalog from './catalog.json';
import { createCatalog } from './catalog-queries';

export * from './catalog-schema';
export * from './catalog-queries';

/** The built-in catalogue shipped with the repository. */
export const builtInCatalog = createCatalog(rawCatalog);
