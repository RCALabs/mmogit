/* tslint:disable */
/* eslint-disable */
export function init(): void;
export class VisitorBook {
  free(): void;
  /**
   * Create a new ephemeral visitor identity
   */
  constructor();
  /**
   * Sign a visitor message
   */
  sign_visit(message: string, user_agent?: string | null): string;
  /**
   * Get the visitor's public key
   */
  get_pubkey(): string;
  /**
   * Create a git commit message for this visit
   */
  create_commit_message(visitor_number: number): string;
}

export type InitInput = RequestInfo | URL | Response | BufferSource | WebAssembly.Module;

export interface InitOutput {
  readonly memory: WebAssembly.Memory;
  readonly __wbg_visitorbook_free: (a: number, b: number) => void;
  readonly visitorbook_new: () => [number, number, number];
  readonly visitorbook_sign_visit: (a: number, b: number, c: number, d: number, e: number) => [number, number, number, number];
  readonly visitorbook_get_pubkey: (a: number) => [number, number];
  readonly visitorbook_create_commit_message: (a: number, b: number) => [number, number];
  readonly init: () => void;
  readonly __wbindgen_exn_store: (a: number) => void;
  readonly __externref_table_alloc: () => number;
  readonly __wbindgen_export_2: WebAssembly.Table;
  readonly __wbindgen_malloc: (a: number, b: number) => number;
  readonly __wbindgen_realloc: (a: number, b: number, c: number, d: number) => number;
  readonly __externref_table_dealloc: (a: number) => void;
  readonly __wbindgen_free: (a: number, b: number, c: number) => void;
  readonly __wbindgen_start: () => void;
}

export type SyncInitInput = BufferSource | WebAssembly.Module;
/**
* Instantiates the given `module`, which can either be bytes or
* a precompiled `WebAssembly.Module`.
*
* @param {{ module: SyncInitInput }} module - Passing `SyncInitInput` directly is deprecated.
*
* @returns {InitOutput}
*/
export function initSync(module: { module: SyncInitInput } | SyncInitInput): InitOutput;

/**
* If `module_or_path` is {RequestInfo} or {URL}, makes a request and
* for everything else, calls `WebAssembly.instantiate` directly.
*
* @param {{ module_or_path: InitInput | Promise<InitInput> }} module_or_path - Passing `InitInput` directly is deprecated.
*
* @returns {Promise<InitOutput>}
*/
export default function __wbg_init (module_or_path?: { module_or_path: InitInput | Promise<InitInput> } | InitInput | Promise<InitInput>): Promise<InitOutput>;
