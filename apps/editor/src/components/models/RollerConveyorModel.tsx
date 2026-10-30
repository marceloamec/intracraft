// Roller conveyor (S01): chain-driven rollers between two side frames, a chain guard on the drive
// side, one geared motor (under the deck or outside on a drive roller), optional guide disks on
// every third roller and an optional end stop, on feet braced according to the height.
import { type RunnerGuide, guideDiskPositions, guideDiskRollers, rollerLayout } from '@intracraft/geometry';
import { PALLET_TYPES } from '@intracraft/library';
import { ChainGuard, DriveUnit, EndStopPlate, FRAME_HEIGHT, GuideDisks, Legs, RollerBed, SideFrames, n, safe } from './parts';
import type { ModelProps } from './types';

/** Bottom-board widths of the EUR pallet across travel (S10): outer runners and centre runner. */
const EUR_RUNNERS = eurRunners();

export function RollerConveyorModel({ params }: ModelProps) {
  const length = n(params, 'length');
  const width = n(params, 'usableWidth');
  const height = n(params, 'surfaceHeight');
  const pitch = n(params, 'rollerPitch');
  const diameter = n(params, 'rollerDiameter');
  const side = params['driveSide'] === 'left' ? -1 : 1;
  const rollers = safe(() => rollerLayout(length, pitch, diameter).positions, []);
  // Guide disks guide runners that run along travel: EUR with the long side first.
  const guide = params['palletSideAlongTravel'] === 'long' ? ((params['runnerGuide'] ?? 'none') as RunnerGuide) : 'none';
  const diskZ = guideDiskPositions(guide, EUR_RUNNERS.width, EUR_RUNNERS);
  const diskX = guideDiskRollers(rollers.length).map((i) => rollers[i] ?? 0);
  const outside = params['driveMounting'] === 'outside';
  return (
    <>
      <RollerBed x0={0} length={length} width={width} surfaceHeight={height} pitch={pitch} diameter={diameter} />
      <SideFrames x0={0} length={length} width={width} top={height} />
      <ChainGuard x0={0} length={length} width={width} top={height} side={side} />
      {outside ? (
        <DriveUnit position={[length / 2, height - FRAME_HEIGHT / 2, side * (width / 2 + 0.22)]} axis="z" side={side} />
      ) : (
        <DriveUnit position={[length / 2, height - FRAME_HEIGHT - 0.17, side * (width / 2 - 0.2)]} axis="x" side={side} />
      )}
      {diskZ.length > 0 && <GuideDisks xs={diskX} zs={diskZ} top={height} diameter={diameter} />}
      {params['endStop'] === true && <EndStopPlate x={length} width={width} top={height} />}
      <Legs x0={0} length={length} width={width} height={height - FRAME_HEIGHT} />
    </>
  );
}

function eurRunners(): { width: number; outer: number; centre: number } {
  const eur = PALLET_TYPES.find((p) => p.id === 'eur');
  const boards = eur?.occupancy.alongWidth[0]?.solid ?? [];
  const size = (i: number) => (boards[i]?.[1] ?? 0) - (boards[i]?.[0] ?? 0);
  return { width: eur?.width ?? 0.8, outer: size(0), centre: size(1) };
}
