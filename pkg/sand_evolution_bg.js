import * as wasm from './sand_evolution_bg.wasm';

const heap = new Array(32).fill(undefined);

heap.push(undefined, null, true, false);

function getObject(idx) { return heap[idx]; }

let heap_next = heap.length;

function dropObject(idx) {
    if (idx < 36) return;
    heap[idx] = heap_next;
    heap_next = idx;
}

function takeObject(idx) {
    const ret = getObject(idx);
    dropObject(idx);
    return ret;
}

function addHeapObject(obj) {
    if (heap_next === heap.length) heap.push(heap.length + 1);
    const idx = heap_next;
    heap_next = heap[idx];

    heap[idx] = obj;
    return idx;
}

const lTextDecoder = typeof TextDecoder === 'undefined' ? (0, module.require)('util').TextDecoder : TextDecoder;

let cachedTextDecoder = new lTextDecoder('utf-8', { ignoreBOM: true, fatal: true });

cachedTextDecoder.decode();

let cachedUint8Memory0 = new Uint8Array();

function getUint8Memory0() {
    if (cachedUint8Memory0.byteLength === 0) {
        cachedUint8Memory0 = new Uint8Array(wasm.memory.buffer);
    }
    return cachedUint8Memory0;
}

function getStringFromWasm0(ptr, len) {
    return cachedTextDecoder.decode(getUint8Memory0().subarray(ptr, ptr + len));
}

function isLikeNone(x) {
    return x === undefined || x === null;
}

let cachedFloat64Memory0 = new Float64Array();

function getFloat64Memory0() {
    if (cachedFloat64Memory0.byteLength === 0) {
        cachedFloat64Memory0 = new Float64Array(wasm.memory.buffer);
    }
    return cachedFloat64Memory0;
}

let cachedInt32Memory0 = new Int32Array();

function getInt32Memory0() {
    if (cachedInt32Memory0.byteLength === 0) {
        cachedInt32Memory0 = new Int32Array(wasm.memory.buffer);
    }
    return cachedInt32Memory0;
}

function debugString(val) {
    // primitive types
    const type = typeof val;
    if (type == 'number' || type == 'boolean' || val == null) {
        return  `${val}`;
    }
    if (type == 'string') {
        return `"${val}"`;
    }
    if (type == 'symbol') {
        const description = val.description;
        if (description == null) {
            return 'Symbol';
        } else {
            return `Symbol(${description})`;
        }
    }
    if (type == 'function') {
        const name = val.name;
        if (typeof name == 'string' && name.length > 0) {
            return `Function(${name})`;
        } else {
            return 'Function';
        }
    }
    // objects
    if (Array.isArray(val)) {
        const length = val.length;
        let debug = '[';
        if (length > 0) {
            debug += debugString(val[0]);
        }
        for(let i = 1; i < length; i++) {
            debug += ', ' + debugString(val[i]);
        }
        debug += ']';
        return debug;
    }
    // Test for built-in
    const builtInMatches = /\[object ([^\]]+)\]/.exec(toString.call(val));
    let className;
    if (builtInMatches.length > 1) {
        className = builtInMatches[1];
    } else {
        // Failed to match the standard '[object ClassName]'
        return toString.call(val);
    }
    if (className == 'Object') {
        // we're a user defined class or Object
        // JSON.stringify avoids problems with cycles, and is generally much
        // easier than looping through ownProperties of `val`.
        try {
            return 'Object(' + JSON.stringify(val) + ')';
        } catch (_) {
            return 'Object';
        }
    }
    // errors
    if (val instanceof Error) {
        return `${val.name}: ${val.message}\n${val.stack}`;
    }
    // TODO we could test for more things here, like `Set`s and `Map`s.
    return className;
}

let WASM_VECTOR_LEN = 0;

const lTextEncoder = typeof TextEncoder === 'undefined' ? (0, module.require)('util').TextEncoder : TextEncoder;

let cachedTextEncoder = new lTextEncoder('utf-8');

const encodeString = (typeof cachedTextEncoder.encodeInto === 'function'
    ? function (arg, view) {
    return cachedTextEncoder.encodeInto(arg, view);
}
    : function (arg, view) {
    const buf = cachedTextEncoder.encode(arg);
    view.set(buf);
    return {
        read: arg.length,
        written: buf.length
    };
});

function passStringToWasm0(arg, malloc, realloc) {

    if (realloc === undefined) {
        const buf = cachedTextEncoder.encode(arg);
        const ptr = malloc(buf.length);
        getUint8Memory0().subarray(ptr, ptr + buf.length).set(buf);
        WASM_VECTOR_LEN = buf.length;
        return ptr;
    }

    let len = arg.length;
    let ptr = malloc(len);

    const mem = getUint8Memory0();

    let offset = 0;

    for (; offset < len; offset++) {
        const code = arg.charCodeAt(offset);
        if (code > 0x7F) break;
        mem[ptr + offset] = code;
    }

    if (offset !== len) {
        if (offset !== 0) {
            arg = arg.slice(offset);
        }
        ptr = realloc(ptr, len, len = offset + arg.length * 3);
        const view = getUint8Memory0().subarray(ptr + offset, ptr + len);
        const ret = encodeString(arg, view);

        offset += ret.written;
    }

    WASM_VECTOR_LEN = offset;
    return ptr;
}

function makeMutClosure(arg0, arg1, dtor, f) {
    const state = { a: arg0, b: arg1, cnt: 1, dtor };
    const real = (...args) => {
        // First up with a closure we increment the internal reference
        // count. This ensures that the Rust closure environment won't
        // be deallocated while we're invoking it.
        state.cnt++;
        const a = state.a;
        state.a = 0;
        try {
            return f(a, state.b, ...args);
        } finally {
            if (--state.cnt === 0) {
                wasm.__wbindgen_export_2.get(state.dtor)(a, state.b);

            } else {
                state.a = a;
            }
        }
    };
    real.original = state;

    return real;
}
function __wbg_adapter_18(arg0, arg1, arg2) {
    wasm._dyn_core__ops__function__FnMut__A____Output___R_as_wasm_bindgen__closure__WasmClosure___describe__invoke__h3ede48a298b92dce(arg0, arg1, addHeapObject(arg2));
}

function __wbg_adapter_29(arg0, arg1) {
    wasm._dyn_core__ops__function__FnMut_____Output___R_as_wasm_bindgen__closure__WasmClosure___describe__invoke__h0dda942f5f46706a(arg0, arg1);
}

/**
*/
export function run() {
    wasm.run();
}

function handleError(f, args) {
    try {
        return f.apply(this, args);
    } catch (e) {
        wasm.__wbindgen_exn_store(addHeapObject(e));
    }
}

export function __wbindgen_cb_drop(arg0) {
    const obj = takeObject(arg0).original;
    if (obj.cnt-- == 1) {
        obj.a = 0;
        return true;
    }
    const ret = false;
    return ret;
};

export function __wbindgen_object_drop_ref(arg0) {
    takeObject(arg0);
};

export function __wbindgen_object_clone_ref(arg0) {
    const ret = getObject(arg0);
    return addHeapObject(ret);
};

export function __wbindgen_string_new(arg0, arg1) {
    const ret = getStringFromWasm0(arg0, arg1);
    return addHeapObject(ret);
};

export function __wbg_new_abda76e883ba8a5f() {
    const ret = new Error();
    return addHeapObject(ret);
};

export function __wbg_stack_658279fe44541cf6(arg0, arg1) {
    const ret = getObject(arg1).stack;
    const ptr0 = passStringToWasm0(ret, wasm.__wbindgen_malloc, wasm.__wbindgen_realloc);
    const len0 = WASM_VECTOR_LEN;
    getInt32Memory0()[arg0 / 4 + 1] = len0;
    getInt32Memory0()[arg0 / 4 + 0] = ptr0;
};

export function __wbg_error_f851667af71bcfc6(arg0, arg1) {
    try {
        console.error(getStringFromWasm0(arg0, arg1));
    } finally {
        wasm.__wbindgen_free(arg0, arg1);
    }
};

export function __wbindgen_number_get(arg0, arg1) {
    const obj = getObject(arg1);
    const ret = typeof(obj) === 'number' ? obj : undefined;
    getFloat64Memory0()[arg0 / 8 + 1] = isLikeNone(ret) ? 0 : ret;
    getInt32Memory0()[arg0 / 4 + 0] = !isLikeNone(ret);
};

export function __wbg_instanceof_Window_acc97ff9f5d2c7b4(arg0) {
    let result;
    try {
        result = getObject(arg0) instanceof Window;
    } catch {
        result = false;
    }
    const ret = result;
    return ret;
};

export function __wbg_document_3ead31dbcad65886(arg0) {
    const ret = getObject(arg0).document;
    return isLikeNone(ret) ? 0 : addHeapObject(ret);
};

export function __wbg_innerWidth_ffa584f74d721fce() { return handleError(function (arg0) {
    const ret = getObject(arg0).innerWidth;
    return addHeapObject(ret);
}, arguments) };

export function __wbg_innerHeight_f4804c803fcf02b0() { return handleError(function (arg0) {
    const ret = getObject(arg0).innerHeight;
    return addHeapObject(ret);
}, arguments) };

export function __wbg_devicePixelRatio_476ddb014eb2520a(arg0) {
    const ret = getObject(arg0).devicePixelRatio;
    return ret;
};

export function __wbg_cancelAnimationFrame_679ac3913d7f9b34() { return handleError(function (arg0, arg1) {
    getObject(arg0).cancelAnimationFrame(arg1);
}, arguments) };

export function __wbg_matchMedia_0b5dc8aaf445df72() { return handleError(function (arg0, arg1, arg2) {
    const ret = getObject(arg0).matchMedia(getStringFromWasm0(arg1, arg2));
    return isLikeNone(ret) ? 0 : addHeapObject(ret);
}, arguments) };

export function __wbg_requestAnimationFrame_4181656476a7d86c() { return handleError(function (arg0, arg1) {
    const ret = getObject(arg0).requestAnimationFrame(getObject(arg1));
    return ret;
}, arguments) };

export function __wbg_get_55f248d76a5aa3d1(arg0, arg1, arg2) {
    const ret = getObject(arg0)[getStringFromWasm0(arg1, arg2)];
    return isLikeNone(ret) ? 0 : addHeapObject(ret);
};

export function __wbg_clearTimeout_7d6f7bfeed34b348(arg0, arg1) {
    getObject(arg0).clearTimeout(arg1);
};

export function __wbg_setTimeout_d6fcf0d9067b8e64() { return handleError(function (arg0, arg1, arg2) {
    const ret = getObject(arg0).setTimeout(getObject(arg1), arg2);
    return ret;
}, arguments) };

export function __wbg_matches_0ffc2232d99a6034(arg0) {
    const ret = getObject(arg0).matches;
    return ret;
};

export function __wbg_addListener_19238ce0935173e6() { return handleError(function (arg0, arg1) {
    getObject(arg0).addListener(getObject(arg1));
}, arguments) };

export function __wbg_removeListener_c08dac8493263a47() { return handleError(function (arg0, arg1) {
    getObject(arg0).removeListener(getObject(arg1));
}, arguments) };

export function __wbg_setProperty_e489dfd8c0a6bffc() { return handleError(function (arg0, arg1, arg2, arg3, arg4) {
    getObject(arg0).setProperty(getStringFromWasm0(arg1, arg2), getStringFromWasm0(arg3, arg4));
}, arguments) };

export function __wbg_x_419967b8271dcf59(arg0) {
    const ret = getObject(arg0).x;
    return ret;
};

export function __wbg_y_0f67486e0f88b265(arg0) {
    const ret = getObject(arg0).y;
    return ret;
};

export function __wbg_clientX_e39206f946859108(arg0) {
    const ret = getObject(arg0).clientX;
    return ret;
};

export function __wbg_clientY_e376bb2d8f470c88(arg0) {
    const ret = getObject(arg0).clientY;
    return ret;
};

export function __wbg_offsetX_8891849b36542d53(arg0) {
    const ret = getObject(arg0).offsetX;
    return ret;
};

export function __wbg_offsetY_1f52082687af467b(arg0) {
    const ret = getObject(arg0).offsetY;
    return ret;
};

export function __wbg_ctrlKey_4795fb55a59f026c(arg0) {
    const ret = getObject(arg0).ctrlKey;
    return ret;
};

export function __wbg_shiftKey_81014521a7612e6a(arg0) {
    const ret = getObject(arg0).shiftKey;
    return ret;
};

export function __wbg_altKey_2b8d6d80ead4bad7(arg0) {
    const ret = getObject(arg0).altKey;
    return ret;
};

export function __wbg_metaKey_49e49046d8402fb7(arg0) {
    const ret = getObject(arg0).metaKey;
    return ret;
};

export function __wbg_button_2bb5dc0116d6b89b(arg0) {
    const ret = getObject(arg0).button;
    return ret;
};

export function __wbg_buttons_047716c1296e3d1c(arg0) {
    const ret = getObject(arg0).buttons;
    return ret;
};

export function __wbg_movementX_f5947c282009d740(arg0) {
    const ret = getObject(arg0).movementX;
    return ret;
};

export function __wbg_movementY_2c81eed268321a0a(arg0) {
    const ret = getObject(arg0).movementY;
    return ret;
};

export function __wbg_pointerId_18be034781db46f3(arg0) {
    const ret = getObject(arg0).pointerId;
    return ret;
};

export function __wbg_deltaX_6b627fd6f4c19e51(arg0) {
    const ret = getObject(arg0).deltaX;
    return ret;
};

export function __wbg_deltaY_a5393ec7ac0f7bb4(arg0) {
    const ret = getObject(arg0).deltaY;
    return ret;
};

export function __wbg_deltaMode_a90be314f5c676f1(arg0) {
    const ret = getObject(arg0).deltaMode;
    return ret;
};

export function __wbg_now_8172cd917e5eda6b(arg0) {
    const ret = getObject(arg0).now();
    return ret;
};

export function __wbg_width_2f4b0cbbf1c850d9(arg0) {
    const ret = getObject(arg0).width;
    return ret;
};

export function __wbg_setwidth_afb418d3fbf71ba7(arg0, arg1) {
    getObject(arg0).width = arg1 >>> 0;
};

export function __wbg_height_a81d308a000d91d0(arg0) {
    const ret = getObject(arg0).height;
    return ret;
};

export function __wbg_setheight_3eb8729b59493242(arg0, arg1) {
    getObject(arg0).height = arg1 >>> 0;
};

export function __wbg_matches_206d50bc7cb1f89e(arg0) {
    const ret = getObject(arg0).matches;
    return ret;
};

export function __wbg_fullscreenElement_de98779ddf556e06(arg0) {
    const ret = getObject(arg0).fullscreenElement;
    return isLikeNone(ret) ? 0 : addHeapObject(ret);
};

export function __wbg_createElement_976dbb84fe1661b5() { return handleError(function (arg0, arg1, arg2) {
    const ret = getObject(arg0).createElement(getStringFromWasm0(arg1, arg2));
    return addHeapObject(ret);
}, arguments) };

export function __wbg_getElementById_3a708b83e4f034d7(arg0, arg1, arg2) {
    const ret = getObject(arg0).getElementById(getStringFromWasm0(arg1, arg2));
    return isLikeNone(ret) ? 0 : addHeapObject(ret);
};

export function __wbg_getBoundingClientRect_06acb6ac1c23e409(arg0) {
    const ret = getObject(arg0).getBoundingClientRect();
    return addHeapObject(ret);
};

export function __wbg_requestFullscreen_7d41309612540445() { return handleError(function (arg0) {
    getObject(arg0).requestFullscreen();
}, arguments) };

export function __wbg_setAttribute_d8436c14a59ab1af() { return handleError(function (arg0, arg1, arg2, arg3, arg4) {
    getObject(arg0).setAttribute(getStringFromWasm0(arg1, arg2), getStringFromWasm0(arg3, arg4));
}, arguments) };

export function __wbg_setPointerCapture_7cc6c6e831d5dae0() { return handleError(function (arg0, arg1) {
    getObject(arg0).setPointerCapture(arg1);
}, arguments) };

export function __wbg_debug_f15cb542ea509609(arg0) {
    console.debug(getObject(arg0));
};

export function __wbg_error_ef9a0be47931175f(arg0) {
    console.error(getObject(arg0));
};

export function __wbg_error_02ffd4185a83fe18(arg0, arg1) {
    console.error(getObject(arg0), getObject(arg1));
};

export function __wbg_info_2874fdd5393f35ce(arg0) {
    console.info(getObject(arg0));
};

export function __wbg_log_4b5638ad60bdc54a(arg0) {
    console.log(getObject(arg0));
};

export function __wbg_warn_58110c4a199df084(arg0) {
    console.warn(getObject(arg0));
};

export function __wbg_style_e9380748cee29f13(arg0) {
    const ret = getObject(arg0).style;
    return addHeapObject(ret);
};

export function __wbg_charCode_b0f31612a52c2bff(arg0) {
    const ret = getObject(arg0).charCode;
    return ret;
};

export function __wbg_keyCode_72faed4278f77f2c(arg0) {
    const ret = getObject(arg0).keyCode;
    return ret;
};

export function __wbg_altKey_6dbe46bf3ae42d67(arg0) {
    const ret = getObject(arg0).altKey;
    return ret;
};

export function __wbg_ctrlKey_fd79f035994d9387(arg0) {
    const ret = getObject(arg0).ctrlKey;
    return ret;
};

export function __wbg_shiftKey_908ae224b8722a41(arg0) {
    const ret = getObject(arg0).shiftKey;
    return ret;
};

export function __wbg_metaKey_cdd15bf44efb510e(arg0) {
    const ret = getObject(arg0).metaKey;
    return ret;
};

export function __wbg_key_ad4fc49423a94efa(arg0, arg1) {
    const ret = getObject(arg1).key;
    const ptr0 = passStringToWasm0(ret, wasm.__wbindgen_malloc, wasm.__wbindgen_realloc);
    const len0 = WASM_VECTOR_LEN;
    getInt32Memory0()[arg0 / 4 + 1] = len0;
    getInt32Memory0()[arg0 / 4 + 0] = ptr0;
};

export function __wbg_code_06787cd3c7a60600(arg0, arg1) {
    const ret = getObject(arg1).code;
    const ptr0 = passStringToWasm0(ret, wasm.__wbindgen_malloc, wasm.__wbindgen_realloc);
    const len0 = WASM_VECTOR_LEN;
    getInt32Memory0()[arg0 / 4 + 1] = len0;
    getInt32Memory0()[arg0 / 4 + 0] = ptr0;
};

export function __wbg_getModifierState_135305ae40997dc7(arg0, arg1, arg2) {
    const ret = getObject(arg0).getModifierState(getStringFromWasm0(arg1, arg2));
    return ret;
};

export function __wbg_target_bf704b7db7ad1387(arg0) {
    const ret = getObject(arg0).target;
    return isLikeNone(ret) ? 0 : addHeapObject(ret);
};

export function __wbg_cancelBubble_8c0bdf21c08f1717(arg0) {
    const ret = getObject(arg0).cancelBubble;
    return ret;
};

export function __wbg_preventDefault_3209279b490de583(arg0) {
    getObject(arg0).preventDefault();
};

export function __wbg_stopPropagation_eca3af16f2d02a91(arg0) {
    getObject(arg0).stopPropagation();
};

export function __wbg_addEventListener_cbe4c6f619b032f3() { return handleError(function (arg0, arg1, arg2, arg3) {
    getObject(arg0).addEventListener(getStringFromWasm0(arg1, arg2), getObject(arg3));
}, arguments) };

export function __wbg_addEventListener_1fc744729ac6dc27() { return handleError(function (arg0, arg1, arg2, arg3, arg4) {
    getObject(arg0).addEventListener(getStringFromWasm0(arg1, arg2), getObject(arg3), getObject(arg4));
}, arguments) };

export function __wbg_removeEventListener_dd20475efce70084() { return handleError(function (arg0, arg1, arg2, arg3) {
    getObject(arg0).removeEventListener(getStringFromWasm0(arg1, arg2), getObject(arg3));
}, arguments) };

export function __wbg_appendChild_e513ef0e5098dfdd() { return handleError(function (arg0, arg1) {
    const ret = getObject(arg0).appendChild(getObject(arg1));
    return addHeapObject(ret);
}, arguments) };

export function __wbg_newnoargs_b5b063fc6c2f0376(arg0, arg1) {
    const ret = new Function(getStringFromWasm0(arg0, arg1));
    return addHeapObject(ret);
};

export function __wbg_get_765201544a2b6869() { return handleError(function (arg0, arg1) {
    const ret = Reflect.get(getObject(arg0), getObject(arg1));
    return addHeapObject(ret);
}, arguments) };

export function __wbg_call_97ae9d8645dc388b() { return handleError(function (arg0, arg1) {
    const ret = getObject(arg0).call(getObject(arg1));
    return addHeapObject(ret);
}, arguments) };

export function __wbg_new_0b9bfdd97583284e() {
    const ret = new Object();
    return addHeapObject(ret);
};

export function __wbg_self_6d479506f72c6a71() { return handleError(function () {
    const ret = self.self;
    return addHeapObject(ret);
}, arguments) };

export function __wbg_window_f2557cc78490aceb() { return handleError(function () {
    const ret = window.window;
    return addHeapObject(ret);
}, arguments) };

export function __wbg_globalThis_7f206bda628d5286() { return handleError(function () {
    const ret = globalThis.globalThis;
    return addHeapObject(ret);
}, arguments) };

export function __wbg_global_ba75c50d1cf384f4() { return handleError(function () {
    const ret = global.global;
    return addHeapObject(ret);
}, arguments) };

export function __wbindgen_is_undefined(arg0) {
    const ret = getObject(arg0) === undefined;
    return ret;
};

export function __wbg_is_40a66842732708e7(arg0, arg1) {
    const ret = Object.is(getObject(arg0), getObject(arg1));
    return ret;
};

export function __wbg_set_bf3f89b92d5a34bf() { return handleError(function (arg0, arg1, arg2) {
    const ret = Reflect.set(getObject(arg0), getObject(arg1), getObject(arg2));
    return ret;
}, arguments) };

export function __wbindgen_debug_string(arg0, arg1) {
    const ret = debugString(getObject(arg1));
    const ptr0 = passStringToWasm0(ret, wasm.__wbindgen_malloc, wasm.__wbindgen_realloc);
    const len0 = WASM_VECTOR_LEN;
    getInt32Memory0()[arg0 / 4 + 1] = len0;
    getInt32Memory0()[arg0 / 4 + 0] = ptr0;
};

export function __wbindgen_throw(arg0, arg1) {
    throw new Error(getStringFromWasm0(arg0, arg1));
};

export function __wbindgen_closure_wrapper109(arg0, arg1, arg2) {
    const ret = makeMutClosure(arg0, arg1, 7, __wbg_adapter_18);
    return addHeapObject(ret);
};

export function __wbindgen_closure_wrapper111(arg0, arg1, arg2) {
    const ret = makeMutClosure(arg0, arg1, 7, __wbg_adapter_18);
    return addHeapObject(ret);
};

export function __wbindgen_closure_wrapper113(arg0, arg1, arg2) {
    const ret = makeMutClosure(arg0, arg1, 7, __wbg_adapter_18);
    return addHeapObject(ret);
};

export function __wbindgen_closure_wrapper115(arg0, arg1, arg2) {
    const ret = makeMutClosure(arg0, arg1, 7, __wbg_adapter_18);
    return addHeapObject(ret);
};

export function __wbindgen_closure_wrapper117(arg0, arg1, arg2) {
    const ret = makeMutClosure(arg0, arg1, 7, __wbg_adapter_18);
    return addHeapObject(ret);
};

export function __wbindgen_closure_wrapper119(arg0, arg1, arg2) {
    const ret = makeMutClosure(arg0, arg1, 7, __wbg_adapter_29);
    return addHeapObject(ret);
};

export function __wbindgen_closure_wrapper121(arg0, arg1, arg2) {
    const ret = makeMutClosure(arg0, arg1, 7, __wbg_adapter_18);
    return addHeapObject(ret);
};

export function __wbindgen_closure_wrapper123(arg0, arg1, arg2) {
    const ret = makeMutClosure(arg0, arg1, 7, __wbg_adapter_18);
    return addHeapObject(ret);
};

export function __wbindgen_closure_wrapper125(arg0, arg1, arg2) {
    const ret = makeMutClosure(arg0, arg1, 7, __wbg_adapter_18);
    return addHeapObject(ret);
};

