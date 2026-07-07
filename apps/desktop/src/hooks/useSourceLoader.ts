import { useCallback, useEffect, useMemo, useRef, useState } from 'react';
import { open } from '@tauri-apps/plugin-dialog';
import { convertFileSrc, invoke } from '@tauri-apps/api/core';
import { type SourceMetadata } from '../components/Preview';
import { type ViewerLoadProgress, type ViewerSourceSummary } from '../classes/PlayCanvasViewer';
import { useLocalStorage } from './useLocalStorage';

const PREVIEW_SPLAT_LIMIT = 5_000_000;

function isRawSplatUrl(url: string) {
  return url.split(/[?#]/, 1)[0].toLowerCase().endsWith('.splat');
}

function sourcePathToUrl(path: string) {
  try {
    return convertFileSrc(path);
  } catch {
    return null;
  }
}

export function useSourceLoader({ setStatus }: { setStatus: (s: string) => void }) {
  const [inputPath, setInputPath] = useLocalStorage<string>('ag_input_path', '');
  const [sourceUrl, setSourceUrl] = useState<string | null>(null);
  const [sourceMetadata, setSourceMetadata] = useState<SourceMetadata | null>(null);
  const [isSourceLoading, setIsSourceLoading] = useState(false);
  const [sourceLoadProgress, setSourceLoadProgress] = useState<ViewerLoadProgress | null>(null);
  const [previewMaxSplats, setPreviewMaxSplats] = useState<number | null>(null);
  const sourceLoadTokenRef = useRef(0);
  const isSourceLoadingRef = useRef(false);

  const isSourceReady = useMemo(() => {
    return Boolean(sourceMetadata?.bounds);
  }, [sourceMetadata]);

  const handleBrowseInputPath = useCallback(async () => {
    const selected = await open({
      multiple: false,
      filters: [{ name: 'Point Cloud source', extensions: ['ply', 'splat'] }],
    });
    if (typeof selected === 'string') {
      setInputPath(selected);
    }
  }, [setInputPath]);

  const handleSourceProgress = useCallback((progress: ViewerLoadProgress) => {
    if (!isSourceLoadingRef.current) return;
    setSourceLoadProgress(progress);
    setStatus(sourceProgressStatus(progress.phase));
  }, [setStatus]);

  const handleSourceReady = useCallback((summary: ViewerSourceSummary) => {
    if (!isSourceLoadingRef.current) return;
    isSourceLoadingRef.current = false;
    setIsSourceLoading(false);
    setSourceLoadProgress(null);
    setStatus('source loaded');
    setSourceMetadata((prev) => ({
      path: prev?.path ?? inputPath,
      bytes: prev?.bytes ?? 0,
      format: prev?.format ?? (isRawSplatUrl(inputPath) ? 'splat' : 'ply'),
      splatCount: prev?.splatCount ?? summary.splatCount,
      previewSplatCount: prev?.previewSplatCount ?? summary.previewSplatCount,
      bounds: summary.bounds,
      previewPath: prev?.previewPath,
    }));
  }, [inputPath, setStatus]);

  const handleSourceError = useCallback((err: Error) => {
    if (!isSourceLoadingRef.current) return;
    isSourceLoadingRef.current = false;
    setIsSourceLoading(false);
    setSourceLoadProgress(null);
    setStatus(`error: ${err.message}`);
  }, [setStatus]);

  const loadSource = useCallback(async () => {
    if (!inputPath) return;
    const token = ++sourceLoadTokenRef.current;
    isSourceLoadingRef.current = true;
    setIsSourceLoading(true);
    setSourceLoadProgress(null);
    setSourceMetadata(null);
    setSourceUrl(null);
    setPreviewMaxSplats(null);
    setStatus('loading source');

    try {
      const metadata = await invoke<SourceMetadata>('load_source', { path: inputPath });
      if (sourceLoadTokenRef.current !== token) return;

      const canonicalUrl = sourcePathToUrl(metadata.previewPath || metadata.path);
      if (!canonicalUrl) {
        throw new Error('failed to convert file source path to safe browser URL');
      }

      const nextPreviewMaxSplats = shouldUseRawPreview(metadata)
        ? PREVIEW_SPLAT_LIMIT
        : null;
      if (sourceLoadTokenRef.current !== token) return;

      setPreviewMaxSplats(nextPreviewMaxSplats);
      setSourceUrl(canonicalUrl);
      setSourceMetadata(metadata);
    } catch (err) {
      if (sourceLoadTokenRef.current !== token) return;
      isSourceLoadingRef.current = false;
      setIsSourceLoading(false);
      setStatus(`error: ${err instanceof Error ? err.message : String(err)}`);
    }
  }, [inputPath, setStatus]);

  const cancelLoad = useCallback(() => {
    sourceLoadTokenRef.current += 1;
    isSourceLoadingRef.current = false;
    setIsSourceLoading(false);
    setSourceLoadProgress(null);
    setSourceUrl(null);
    setSourceMetadata(null);
    setPreviewMaxSplats(null);
    setStatus('source load cancelled');
  }, [setStatus]);

  // Auto-reload point cloud on mount if inputPath has been saved
  useEffect(() => {
    if (inputPath) {
      loadSource();
    }
    // eslint-disable-next-line react-hooks/exhaustive-deps
  }, []);

  return {
    inputPath,
    setInputPath,
    handleBrowseInputPath,
    sourceUrl,
    sourceMetadata,
    isSourceLoading,
    sourceLoadProgress,
    previewMaxSplats,
    isSourceReady,
    loadSource,
    cancelLoad,
    handleSourceProgress,
    handleSourceReady,
    handleSourceError,
  };
}

function shouldUseRawPreview(metadata: SourceMetadata) {
  return isRawSplatUrl(metadata.previewPath ?? metadata.path) && metadata.splatCount > PREVIEW_SPLAT_LIMIT;
}

function sourceProgressStatus(phase: ViewerLoadProgress['phase']) {
  switch (phase) {
    case 'decode':
      return 'decoding source';
    case 'build':
      return 'building preview';
    default:
      return 'loading source';
  }
}
