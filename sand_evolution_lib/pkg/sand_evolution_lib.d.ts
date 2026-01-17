/* tslint:disable */
/* eslint-disable */

export function run(w: number, h: number, data: Uint8Array, script: string): Promise<void>;

export type InitInput = RequestInfo | URL | Response | BufferSource | WebAssembly.Module;

export interface InitOutput {
    readonly memory: WebAssembly.Memory;
    readonly run: (a: number, b: number, c: number, d: number, e: number, f: number) => any;
    readonly wgpu_render_pass_draw_indexed: (a: number, b: number, c: number, d: number, e: number, f: number) => void;
    readonly wgpu_render_pass_set_pipeline: (a: number, b: bigint) => void;
    readonly wgpu_render_pass_draw_indirect: (a: number, b: bigint, c: bigint) => void;
    readonly wgpu_render_pass_set_bind_group: (a: number, b: number, c: bigint, d: number, e: number) => void;
    readonly wgpu_render_pass_set_vertex_buffer: (a: number, b: number, c: bigint, d: bigint, e: bigint) => void;
    readonly wgpu_render_pass_set_push_constants: (a: number, b: number, c: number, d: number, e: number) => void;
    readonly wgpu_render_pass_multi_draw_indirect: (a: number, b: bigint, c: bigint, d: number) => void;
    readonly wgpu_render_pass_draw_indexed_indirect: (a: number, b: bigint, c: bigint) => void;
    readonly wgpu_render_pass_multi_draw_indirect_count: (a: number, b: bigint, c: bigint, d: bigint, e: bigint, f: number) => void;
    readonly wgpu_render_pass_multi_draw_indexed_indirect: (a: number, b: bigint, c: bigint, d: number) => void;
    readonly wgpu_render_pass_multi_draw_indexed_indirect_count: (a: number, b: bigint, c: bigint, d: bigint, e: bigint, f: number) => void;
    readonly wgpu_render_pass_draw: (a: number, b: number, c: number, d: number, e: number) => void;
    readonly wgpu_render_pass_set_viewport: (a: number, b: number, c: number, d: number, e: number, f: number, g: number) => void;
    readonly wgpu_render_pass_pop_debug_group: (a: number) => void;
    readonly wgpu_render_pass_write_timestamp: (a: number, b: bigint, c: number) => void;
    readonly wgpu_render_pass_push_debug_group: (a: number, b: number, c: number) => void;
    readonly wgpu_render_pass_set_scissor_rect: (a: number, b: number, c: number, d: number, e: number) => void;
    readonly wgpu_render_pass_set_blend_constant: (a: number, b: number) => void;
    readonly wgpu_render_pass_insert_debug_marker: (a: number, b: number, c: number) => void;
    readonly wgpu_render_pass_set_stencil_reference: (a: number, b: number) => void;
    readonly wgpu_render_pass_end_pipeline_statistics_query: (a: number) => void;
    readonly wgpu_render_pass_begin_pipeline_statistics_query: (a: number, b: bigint, c: number) => void;
    readonly wgpu_compute_pass_set_pipeline: (a: number, b: bigint) => void;
    readonly wgpu_compute_pass_set_bind_group: (a: number, b: number, c: bigint, d: number, e: number) => void;
    readonly wgpu_compute_pass_pop_debug_group: (a: number) => void;
    readonly wgpu_compute_pass_write_timestamp: (a: number, b: bigint, c: number) => void;
    readonly wgpu_compute_pass_push_debug_group: (a: number, b: number, c: number) => void;
    readonly wgpu_compute_pass_set_push_constant: (a: number, b: number, c: number, d: number) => void;
    readonly wgpu_compute_pass_dispatch_workgroups: (a: number, b: number, c: number, d: number) => void;
    readonly wgpu_compute_pass_insert_debug_marker: (a: number, b: number, c: number) => void;
    readonly wgpu_compute_pass_dispatch_workgroups_indirect: (a: number, b: bigint, c: bigint) => void;
    readonly wgpu_compute_pass_end_pipeline_statistics_query: (a: number) => void;
    readonly wgpu_compute_pass_begin_pipeline_statistics_query: (a: number, b: bigint, c: number) => void;
    readonly wgpu_render_bundle_draw_indexed: (a: number, b: number, c: number, d: number, e: number, f: number) => void;
    readonly wgpu_render_bundle_set_pipeline: (a: number, b: bigint) => void;
    readonly wgpu_render_bundle_draw_indirect: (a: number, b: bigint, c: bigint) => void;
    readonly wgpu_render_bundle_set_bind_group: (a: number, b: number, c: bigint, d: number, e: number) => void;
    readonly wgpu_render_bundle_set_vertex_buffer: (a: number, b: number, c: bigint, d: bigint, e: bigint) => void;
    readonly wgpu_render_bundle_set_push_constants: (a: number, b: number, c: number, d: number, e: number) => void;
    readonly wgpu_render_bundle_draw_indexed_indirect: (a: number, b: bigint, c: bigint) => void;
    readonly wgpu_render_bundle_draw: (a: number, b: number, c: number, d: number, e: number) => void;
    readonly wgpu_render_pass_execute_bundles: (a: number, b: number, c: number) => void;
    readonly wgpu_render_pass_set_index_buffer: (a: number, b: bigint, c: number, d: bigint, e: bigint) => void;
    readonly wgpu_render_bundle_insert_debug_marker: (a: number, b: number) => void;
    readonly wgpu_render_bundle_pop_debug_group: (a: number) => void;
    readonly wgpu_render_bundle_set_index_buffer: (a: number, b: bigint, c: number, d: bigint, e: bigint) => void;
    readonly wgpu_render_bundle_push_debug_group: (a: number, b: number) => void;
    readonly wasm_bindgen__closure__destroy__hf016fa347d5d15a0: (a: number, b: number) => void;
    readonly wasm_bindgen__closure__destroy__hf7f7264a96faf459: (a: number, b: number) => void;
    readonly wasm_bindgen__closure__destroy__h875bb32f038dfa14: (a: number, b: number) => void;
    readonly wasm_bindgen__convert__closures_____invoke__h79cf1748309790d4: (a: number, b: number, c: any, d: any) => void;
    readonly wasm_bindgen__convert__closures_____invoke__h8bb9e7af76a41f52: (a: number, b: number, c: any) => void;
    readonly wasm_bindgen__convert__closures_____invoke__h0985dd38368d640e: (a: number, b: number, c: any) => void;
    readonly wasm_bindgen__convert__closures_____invoke__h7323133c8b33ed5c: (a: number, b: number) => void;
    readonly __wbindgen_malloc: (a: number, b: number) => number;
    readonly __wbindgen_realloc: (a: number, b: number, c: number, d: number) => number;
    readonly __wbindgen_exn_store: (a: number) => void;
    readonly __externref_table_alloc: () => number;
    readonly __wbindgen_externrefs: WebAssembly.Table;
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
