// Picks the 3D model of a placed instance (or of the drag preview) by its definition id.
import type { Vec3 } from '@intracraft/core-model';
import { instanceParams } from '@intracraft/library';
import { useMemo } from 'react';
import { registry } from '../../layout-store';
import { ChainConveyorModel } from './ChainConveyorModel';
import { FlowEndModel } from './FlowEndModel';
import { PhotoeyeModel } from './PhotoeyeModel';
import { ProcessStationModel } from './ProcessStationModel';
import { type ModelStyle, ModelStyleContext } from './parts';
import { RollerConveyorModel } from './RollerConveyorModel';
import { TransferModel } from './TransferModel';
import { TurntableModel } from './TurntableModel';
import type { ModelProps } from './types';

const MODELS: Record<string, (props: ModelProps) => React.JSX.Element> = {
  'conveyor.roller': RollerConveyorModel,
  'conveyor.chain': ChainConveyorModel,
  'conveyor.chain_turntable': (props) => <TurntableModel {...props} deck="chains" />,
  'conveyor.roller_turntable': (props) => <TurntableModel {...props} deck="rollers" />,
  'conveyor.transfer_rc': TransferModel,
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
      </ModelStyleContext>
    </group>
  );
}
