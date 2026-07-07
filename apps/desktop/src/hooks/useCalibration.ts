import { useCallback, useEffect, useMemo, useState } from 'react';
import {
  adapterCommandStorageKey,
  defaultScalePoints,
  geometryProfileStorageKey,
  geometryProfileStorageVersion,
  geometryProfileStorageVersionKey,
  makeAlignmentRecipe,
  makeProcessConfig,
  normalizeReconstructionMethod,
  reconstructionMethodStorageKey,
  requiresAdapterCommand,
  storedGeometryProfileOrDefault,
  type GeometryProfile,
  type Point3,
  type ReconstructionMethod,
  type UpAxis,
  type PickMode,
} from '../domains/calibration';
import { useLocalStorage } from './useLocalStorage';

function readStoredJson(key: string): unknown {
  if (typeof localStorage === 'undefined') return undefined;
  try {
    const saved = localStorage.getItem(key);
    return saved === null ? undefined : JSON.parse(saved);
  } catch {
    return undefined;
  }
}

export function useCalibration() {
  const [distance, setDistance] = useLocalStorage<number>('ag_calib_distance', 2.0);
  const [scalePoints, setScalePoints] =
    useLocalStorage<[Point3, Point3]>('ag_calib_scale_points', defaultScalePoints);
  const [upAxis, setUpAxis] = useLocalStorage<UpAxis>('ag_calib_up_axis', 'y');
  const [geometryProfile, setGeometryProfile] = useState<GeometryProfile>(() =>
    storedGeometryProfileOrDefault(
      readStoredJson(geometryProfileStorageKey),
      readStoredJson(geometryProfileStorageVersionKey),
    ));
  const [reconstructionMethod, setReconstructionMethod] = useLocalStorage<ReconstructionMethod>(
    reconstructionMethodStorageKey,
    normalizeReconstructionMethod(readStoredJson(reconstructionMethodStorageKey)),
  );
  const [adapterCommand, setAdapterCommand] = useLocalStorage<string>(
    adapterCommandStorageKey,
    '',
  );
  const [pickMode, setPickMode] = useLocalStorage<PickMode>('ag_calib_pick_mode', 'scale0');

  const [userPickedScalePoints, setUserPickedScalePoints] =
    useLocalStorage<[boolean, boolean]>('ag_calib_user_picked_scale_points', [false, false]);

  useEffect(() => {
    localStorage.setItem(geometryProfileStorageKey, JSON.stringify(geometryProfile));
    localStorage.setItem(
      geometryProfileStorageVersionKey,
      JSON.stringify(geometryProfileStorageVersion),
    );
  }, [geometryProfile]);

  const recipe = useMemo(
    () => makeAlignmentRecipe(
      distance,
      scalePoints,
      upAxis,
      geometryProfile,
    ),
    [distance, geometryProfile, scalePoints, upAxis],
  );
  const processConfig = useMemo(
    () => makeProcessConfig(
      scalePoints,
      distance,
      geometryProfile,
      upAxis,
      reconstructionMethod,
      adapterCommand,
    ),
    [adapterCommand, distance, geometryProfile, reconstructionMethod, scalePoints, upAxis],
  );
  const hasValidReconstructionAdapter = useMemo(
    () => !requiresAdapterCommand(reconstructionMethod) || adapterCommand.trim().length > 0,
    [adapterCommand, reconstructionMethod],
  );

  const isFloorCalibrated = true;

  const isScaleCalibrated = useMemo(() => (
    userPickedScalePoints.every(Boolean)
  ), [userPickedScalePoints]);

  const updateScalePoint = useCallback((index: number, val: Point3) => {
    setScalePoints((prev) => {
      const next = [...prev] as [Point3, Point3];
      next[index] = val;
      return next;
    });
    setUserPickedScalePoints((prev) => {
      const next = [...prev] as [boolean, boolean];
      next[index] = true;
      return next;
    });
  }, [setScalePoints, setUserPickedScalePoints]);

  const handlePick = useCallback((point: Point3) => {
    if (!pickMode) return;
    if (pickMode === 'scale0') {
      updateScalePoint(0, point);
      setPickMode('scale1');
    } else if (pickMode === 'scale1') {
      updateScalePoint(1, point);
      setPickMode('scale0');
    }
  }, [pickMode, updateScalePoint, setPickMode]);

  return {
    distance,
    setDistance,
    scalePoints,
    upAxis,
    setUpAxis,
    geometryProfile,
    setGeometryProfile,
    reconstructionMethod,
    setReconstructionMethod,
    adapterCommand,
    setAdapterCommand,
    hasValidReconstructionAdapter,
    pickMode,
    setPickMode,
    userPickedScalePoints,
    updateScalePoint,
    handlePick,
    recipe,
    processConfig,
    isFloorCalibrated,
    isScaleCalibrated,
  };
}
