export interface Recognizer {
  recognizePage(gray: Uint8Array, width: number, height: number): Promise<Record<string, unknown>>;
  dispose(): void;
}
export declare function createRecognizer(options: {
  modelBaseUrl: string;
  onProgress?: (message: string) => void;
  numThreads?: number;
  timeoutMs?: number;
}): Promise<Recognizer>;
