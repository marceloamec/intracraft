// Picks the 3D model of a placed instance (or of the drag preview) by its definition id.
import type { Vec3 } from '@intracraft/core-model';
import { instanceParams } from '@intracraft/library';
import { useMemo } from 'react';
import { registry } from '../../layout-store';
import { ChainConveyorModel } from './ChainConveyorModel';
import { ElevatorModel } from './ElevatorModel';
import { FlowEndModel } from './FlowEndModel';
import { LiftTableModel } from './LiftTableModel';
import { PhotoeyeModel } from './PhotoeyeModel';
import { ProcessStationModel } from './ProcessStationModel';
import { COLORS, type ModelStyle, ModelStyleContext, n } from './parts';
import { RollerConveyorModel } from './RollerConveyorModel';
import { TransferModel } from './TransferModel';
import { TurntableModel } from './TurntableModel';
import type { ModelProps } from './types';

/** Base heights above this are drawn standing on a mezzanine plate (m). */
const MIN_RAISED_BASE = 0.01;
const PLATE_THICKNESS = 0.05;
/** The plate reaches this far beyond the object on each side (m). */
const PLATE_MARGIN = 0.3;

const MODELS: Record<string, (props: ModelProps) => React.JSX.Element> = {
  'conveyor.roller': RollerConveyorModel,
  'conveyor.chain': ChainConveyorModel,
  'conveyor.chain_turntable': (props) => <TurntableModel {...props} deck="chains" />,
  'conveyor.roller_turntable': (props) => <TurntableModel {...props} deck="rollers" />,
  'conveyor.transfer_rc': TransferModel,
  'conveyor.lift_table_manual': (props) => <LiftTableModel {...props} variant="manual" />,
  'conveyor.scissor_lift': (props) => <LiftTableModel {...props} variant="scissor" />,
  'conveyor.elevator': ElevatorModel,
  'flow.source': (props) => <FlowEndModel {...props} kind="source" />,
  'flow.sink': (props) => <FlowEndModel {...props} kind="sink" />,
  'flow.process_station': ProcessStationModel,
  'sensor.photoeye': PhotoeyeModel,
};

export function InstanceModel({
  instanceId,
  definitionId,
  params,
  position,
  yaw,
  style,
}: {
  /** Id used for picking; omitted for the drag preview. */
  instanceId?: string;
  definitionId: string;
  params: Record<string, number | boolean | string>;
  position: Vec3;
  yaw: number;
  style: ModelStyle;
}) {
  const values = useMemo(() => instanceParams(registry.get(definitionId), params), [definitionId, params]);
  const Model = MODELS[definitionId];
  if (Model === undefined) return null;
  return (
    <group
      position={[position.x, position.y, position.z]}
      rotation={[0, yaw, 0]}
      userData={instanceId === undefined ? { ghost: true } : { instanceId }}
    >
      <ModelStyleContext value={style}>
        {instanceId === undefined ? <Model params={values} /> : <Model params={values} instanceId={instanceId} />}
        {position.y > MIN_RAISED_BASE && <MezzaninePlate params={values} />}
      </ModelStyleContext>
    </group>
  );
}

/** A slab under an object on a raised base (ADR-0013 §8), so that it does not seem to float. */
function MezzaninePlate({ params }: { params: Record<string, number | boolean | string> }) {
  const length = n(params, 'length', 0.4);
  const width = n(params, 'usableWidth', n(params, 'platformWidth', 1));
  return (
    <mesh position={[length / 2, -PLATE_THICKNESS / 2, 0]} receiveShadow userData={{ noPick: true }}>
      <boxGeometry args={[length + 2 * PLATE_MARGIN, PLATE_THICKNESS, width + 2 * PLATE_MARGIN]} />
      <meshStandardMaterial color={COLORS.base} transparent opacity={0.45} roughness={0.9} />
    </mesh>
  );
}
