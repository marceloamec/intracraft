import type { Layout } from '@intracraft/core-model';
import { createBuiltInRegistry } from '@intracraft/library';
import { describe, expect, it } from 'vitest';
import { resultsCsv } from './results-export';

const LAYOUT: Layout = {
  instances: [
    { id: 'roller1', definitionId: 'conveyor.roller', definitionVersion: '0.1.0', name: 'Infeed, left', position: { x: 0, y: 0, z: 0 }, yaw: 0, params: {}, attributes: {} },
    { id: 'sink1', definitionId: 'flow.sink', definitionVersion: '0.1.0', name: 'Out', position: { x: 3, y: 0, z: 0 }, yaw: 0, params: {}, attributes: {} },
  ],
  connections: [],
};

describe('results CSV', () => {
  it('writes one row per object and statistic, with units, quoting commas', () => {
    const csv = resultsCsv(LAYOUT, { roller1: { utilization: 0.5 }, sink1: { palletsReceived: 3, throughputPerHour: 120 } }, createBuiltInRegistry(), 90_000_000);
    expect(csv.split('\n')).toEqual([
      'simulated time (s),object id,name,type,statistic,value,unit',
      '90,roller1,"Infeed, left",Roller conveyor,utilization,0.5,fraction',
      '90,sink1,Out,Pallet sink,palletsReceived,3,count',
      '90,sink1,Out,Pallet sink,throughputPerHour,120,pallets/h',
      '',
    ]);
  });
});
