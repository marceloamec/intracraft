export interface ModelProps {
  /** Instance parameters with the library defaults filled in (SI units). */
  params: Record<string, number | boolean | string>;
  /** Placed instance shown (absent for the drag preview); used to follow the running simulation. */
  instanceId?: string;
}
