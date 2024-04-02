/* tslint:disable */
/* eslint-disable */
/**
* @param {string} s
* @returns {any}
*/
export function shb_to_html(s: string): any;
/**
* @param {string} s
* @returns {any}
*/
export function shb_to_html_db(s: string): any;
/**
* @param {string} s
* @returns {any}
*/
export function shb_to_db_info(s: string): any;
/**
* @param {string} s
* @returns {Promise<any>}
*/
export function lst_to_html(s: string): Promise<any>;
/**
* @param {string} s
* @returns {Promise<any>}
*/
export function lst_to_array(s: string): Promise<any>;
/**
*/
export class LST_Return {
  free(): void;
/**
*/
  content: string;
/**
*/
  l_errors: any;
/**
*/
  s_errors: any;
}
/**
*/
export class ParsingError {
  free(): void;
/**
*/
  from: number;
/**
*/
  line: number;
/**
*/
  message: string;
/**
*/
  to: number;
}
/**
*/
export class SHB_DB_Return {
  free(): void;
/**
*/
  norm_title: string;
/**
*/
  plain: string;
/**
*/
  sections: string;
/**
*/
  subtitle: string;
/**
*/
  title: string;
/**
*/
  tonic: number;
/**
*/
  tonic_kind: number;
}
/**
*/
export class SHB_Full_Return {
  free(): void;
/**
*/
  content: string;
/**
*/
  errors: any;
/**
*/
  plain: string;
/**
*/
  sections: string;
}
/**
*/
export class SHB_Return {
  free(): void;
/**
*/
  content: string;
/**
*/
  errors: any;
}

export type InitInput = RequestInfo | URL | Response | BufferSource | WebAssembly.Module;

export interface InitOutput {
  readonly memory: WebAssembly.Memory;
  readonly __wbg_parsingerror_free: (a: number) => void;
  readonly __wbg_get_parsingerror_line: (a: number) => number;
  readonly __wbg_set_parsingerror_line: (a: number, b: number) => void;
  readonly __wbg_get_parsingerror_from: (a: number) => number;
  readonly __wbg_set_parsingerror_from: (a: number, b: number) => void;
  readonly __wbg_get_parsingerror_to: (a: number) => number;
  readonly __wbg_set_parsingerror_to: (a: number, b: number) => void;
  readonly __wbg_shb_return_free: (a: number) => void;
  readonly __wbg_shb_full_return_free: (a: number) => void;
  readonly __wbg_get_shb_full_return_errors: (a: number) => number;
  readonly __wbg_set_shb_full_return_errors: (a: number, b: number) => void;
  readonly __wbg_shb_db_return_free: (a: number) => void;
  readonly __wbg_get_shb_db_return_sections: (a: number, b: number) => void;
  readonly __wbg_set_shb_db_return_sections: (a: number, b: number, c: number) => void;
  readonly __wbg_get_shb_db_return_title: (a: number, b: number) => void;
  readonly __wbg_set_shb_db_return_title: (a: number, b: number, c: number) => void;
  readonly __wbg_get_shb_db_return_norm_title: (a: number, b: number) => void;
  readonly __wbg_set_shb_db_return_norm_title: (a: number, b: number, c: number) => void;
  readonly __wbg_get_shb_db_return_subtitle: (a: number, b: number) => void;
  readonly __wbg_set_shb_db_return_subtitle: (a: number, b: number, c: number) => void;
  readonly __wbg_get_shb_db_return_tonic: (a: number) => number;
  readonly __wbg_set_shb_db_return_tonic: (a: number, b: number) => void;
  readonly __wbg_get_shb_db_return_tonic_kind: (a: number) => number;
  readonly __wbg_set_shb_db_return_tonic_kind: (a: number, b: number) => void;
  readonly __wbg_lst_return_free: (a: number) => void;
  readonly __wbg_get_lst_return_content: (a: number, b: number) => void;
  readonly __wbg_set_lst_return_content: (a: number, b: number, c: number) => void;
  readonly __wbg_get_lst_return_s_errors: (a: number) => number;
  readonly __wbg_set_lst_return_s_errors: (a: number, b: number) => void;
  readonly __wbg_get_lst_return_l_errors: (a: number) => number;
  readonly __wbg_set_lst_return_l_errors: (a: number, b: number) => void;
  readonly shb_to_html: (a: number, b: number) => number;
  readonly shb_to_html_db: (a: number, b: number) => number;
  readonly shb_to_db_info: (a: number, b: number) => number;
  readonly lst_to_html: (a: number, b: number) => number;
  readonly lst_to_array: (a: number, b: number) => number;
  readonly __wbg_set_shb_return_content: (a: number, b: number, c: number) => void;
  readonly __wbg_set_shb_full_return_content: (a: number, b: number, c: number) => void;
  readonly __wbg_set_shb_db_return_plain: (a: number, b: number, c: number) => void;
  readonly __wbg_set_shb_full_return_plain: (a: number, b: number, c: number) => void;
  readonly __wbg_set_shb_full_return_sections: (a: number, b: number, c: number) => void;
  readonly __wbg_set_parsingerror_message: (a: number, b: number, c: number) => void;
  readonly __wbg_get_shb_return_content: (a: number, b: number) => void;
  readonly __wbg_get_shb_full_return_content: (a: number, b: number) => void;
  readonly __wbg_get_shb_db_return_plain: (a: number, b: number) => void;
  readonly __wbg_get_shb_full_return_plain: (a: number, b: number) => void;
  readonly __wbg_get_shb_full_return_sections: (a: number, b: number) => void;
  readonly __wbg_get_parsingerror_message: (a: number, b: number) => void;
  readonly __wbg_get_shb_return_errors: (a: number) => number;
  readonly __wbg_set_shb_return_errors: (a: number, b: number) => void;
  readonly __wbindgen_malloc: (a: number, b: number) => number;
  readonly __wbindgen_realloc: (a: number, b: number, c: number, d: number) => number;
  readonly __wbindgen_export_2: WebAssembly.Table;
  readonly _dyn_core__ops__function__FnMut__A____Output___R_as_wasm_bindgen__closure__WasmClosure___describe__invoke__h6b49fc002d773d6d: (a: number, b: number, c: number) => void;
  readonly __wbindgen_add_to_stack_pointer: (a: number) => number;
  readonly __wbindgen_free: (a: number, b: number, c: number) => void;
  readonly __wbindgen_exn_store: (a: number) => void;
  readonly wasm_bindgen__convert__closures__invoke2_mut__h4741b0a938afd585: (a: number, b: number, c: number, d: number) => void;
}

export type SyncInitInput = BufferSource | WebAssembly.Module;
/**
* Instantiates the given `module`, which can either be bytes or
* a precompiled `WebAssembly.Module`.
*
* @param {SyncInitInput} module
*
* @returns {InitOutput}
*/
export function initSync(module: SyncInitInput): InitOutput;

/**
* If `module_or_path` is {RequestInfo} or {URL}, makes a request and
* for everything else, calls `WebAssembly.instantiate` directly.
*
* @param {InitInput | Promise<InitInput>} module_or_path
*
* @returns {Promise<InitOutput>}
*/
export default function __wbg_init (module_or_path?: InitInput | Promise<InitInput>): Promise<InitOutput>;
