// A pallet moving through the 2D model. Its position is the front edge measured along the conveyor
// that carries it (the conveyor under its tail), valid at that conveyor's last update time.
import type { SimTime } from '@intracraft/core-model';

export interface Pallet {
  readonly id: number;
  readonly typeId: string;
  /** Which pallet side is parallel to travel; fixed at creation (no turning in the MVP lines). */
  readonly along: 'long' | 'short';
  /** Size along travel (m). */
  readonly length: number;
  /** Size across travel (m). */
  readonly width: number;
  readonly height: number;
  readonly loadMass: number;
  readonly loadHeight: number;
  readonly createdAt: SimTime;
  /** When it was put on its current conveyor. */
  enteredAt: SimTime;
  /** Front edge on the carrying conveyor (m from its entry). */
  front: number;
}
