import { parseSplatColumns, type ParsedSplatColumns } from '../utils/splatPreview';

const ctx = self as any;

type DecodeRequest = {
  type: 'decode-splat';
  id: number;
  buffer: ArrayBuffer;
  maxPreviewSplats?: number | null;
};

type DecodeSuccess = {
  type: 'decoded-splat';
  id: number;
  parsed: ParsedSplatColumns;
};

type DecodeFailure = {
  type: 'decode-error';
  id: number;
  message: string;
};

ctx.onmessage = (event: MessageEvent<DecodeRequest>) => {
  const message = event.data;
  if (message.type !== 'decode-splat') return;

  try {
    const parsed = parseSplatColumns(message.buffer, message.maxPreviewSplats);
    const transfers = Object.values(parsed.columns).map((column) => column.buffer as ArrayBuffer);
    ctx.postMessage(
      { type: 'decoded-splat', id: message.id, parsed } satisfies DecodeSuccess,
      transfers,
    );
  } catch (err) {
    ctx.postMessage({
      type: 'decode-error',
      id: message.id,
      message: err instanceof Error ? err.message : String(err),
    } satisfies DecodeFailure);
  }
};
