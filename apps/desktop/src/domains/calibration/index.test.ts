import { describe, expect, it } from 'vitest';
import {
  defaultGeometryProfile,
  defaultScalePoints,
  geometryProfileStorageVersion,
  makeAlignmentRecipe,
  makeEditRecipe,
  makeProcessConfig,
  normalizeReconstructionMethod,
  requiresAdapterCommand,
  storedGeometryProfileOrDefault,
} from './index';

describe('calibration profile defaults', () => {
  it('defaults to object prop baking without carve, navmesh, or cluster filtering', () => {
    const config = makeProcessConfig(defaultScalePoints, 2);
    const recipe = makeAlignmentRecipe(2, defaultScalePoints);

    expect(defaultGeometryProfile).toBe('object-prop');
    expect(config.voxel).toEqual({ backend: 'gpu', size: 0.05, opacityThreshold: 0.1 });
    expect(config.voxelFill).toEqual({ mode: 'none', dilationSize: 0 });
    expect(config.voxelCarve.enabled).toBe(false);
    expect(config.reconstruction).toEqual({
      method: 'voxel',
      adapterCommand: undefined,
      targetTriangles: 50000,
      timeoutSeconds: 900,
    });
    expect(config.navmesh.enabled).toBe(false);
    expect(config.navmesh).toMatchObject({
      agentHeight: 1.6,
      agentRadius: 0.2,
      maxSlopeDegrees: 45,
      cellSize: 0.1,
      cellHeight: 0.05,
      walkableClimb: 0.25,
      minRegionSize: 4,
      mergeRegionSize: 12,
    });
    expect(recipe.editRecipe.operations).toEqual([]);
  });

  it('does not inject filterCluster for any bake profile', () => {
    expect(makeEditRecipe().operations).toEqual([]);
  });

  it('keeps room and terrain process profile behavior explicit', () => {
    const room = makeProcessConfig(defaultScalePoints, 2, 'interior-room');
    const terrain = makeProcessConfig(defaultScalePoints, 2, 'outdoor-terrain');

    expect(room.voxelFill.mode).toBe('exterior-fill');
    expect(room.voxelCarve.enabled).toBe(true);
    expect(room.navmesh.enabled).toBe(true);
    expect(terrain.voxelFill.mode).toBe('floor-fill');
    expect(terrain.voxelCarve.enabled).toBe(false);
    expect(terrain.navmesh.enabled).toBe(false);
  });

  it('migrates stale stored geometry profile to object prop once', () => {
    expect(storedGeometryProfileOrDefault('interior-room', undefined)).toBe('object-prop');
    expect(storedGeometryProfileOrDefault('interior-room', 1)).toBe('object-prop');
    expect(storedGeometryProfileOrDefault('interior-room', geometryProfileStorageVersion))
      .toBe('interior-room');
    expect(storedGeometryProfileOrDefault('bad-profile', geometryProfileStorageVersion))
      .toBe('object-prop');
  });

  it('normalizes and validates reconstruction methods', () => {
    expect(normalizeReconstructionMethod('sugar')).toBe('sugar');
    expect(normalizeReconstructionMethod('poisson')).toBe('poisson');
    expect(normalizeReconstructionMethod('bad')).toBe('voxel');
    expect(requiresAdapterCommand('voxel')).toBe(false);
    expect(requiresAdapterCommand('sugar')).toBe(true);
    expect(requiresAdapterCommand('poisson')).toBe(true);
  });

  it('adds adapter command for external reconstruction configs', () => {
    const config = makeProcessConfig(defaultScalePoints, 2, 'object-prop', 'y', 'sugar', 'run-sugar');

    expect(config.reconstruction).toEqual({
      method: 'sugar',
      adapterCommand: 'run-sugar',
      targetTriangles: 50000,
      timeoutSeconds: 900,
    });
  });
});
