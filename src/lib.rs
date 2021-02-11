use ocaml_interop::*;

pub type IrminKey = OCamlList<String>;
pub struct ContextHash(pub Vec<u8>);
pub struct ProtocolHash(pub Vec<u8>);
enum TaggedHash<'a> {
    Hash(&'a [u8]),
}
pub struct IrminContextIndex {}
pub struct IrminContext {}

unsafe impl FromOCaml<ContextHash> for ContextHash {
    fn from_ocaml(v: OCaml<ContextHash>) -> Self {
        ContextHash(unsafe { v.field::<OCamlBytes>(0).to_rust() })
    }
}

unsafe impl ToOCaml<ContextHash> for ContextHash {
    fn to_ocaml<'a>(&self, cr: &'a mut OCamlRuntime) -> OCaml<'a, ContextHash> {
        let hash = TaggedHash::Hash(&self.0);
        ocaml_alloc_variant! {
            cr, hash => {
                TaggedHash::Hash(hash: OCamlBytes)
            }
        }
    }
}

unsafe impl FromOCaml<ProtocolHash> for ProtocolHash {
    fn from_ocaml(v: OCaml<ProtocolHash>) -> Self {
        ProtocolHash(unsafe { v.field::<OCamlBytes>(0).to_rust() })
    }
}

unsafe impl ToOCaml<ProtocolHash> for ProtocolHash {
    fn to_ocaml<'a>(&self, cr: &'a mut OCamlRuntime) -> OCaml<'a, ProtocolHash> {
        let hash = TaggedHash::Hash(&self.0);
        ocaml_alloc_variant! {
            cr, hash => {
                TaggedHash::Hash(hash: OCamlBytes)
            }
        }
    }
}

mod ffi {
    use super::{ContextHash, ProtocolHash, IrminContext, IrminContextIndex, IrminKey};
    use ocaml_interop::*;

    ocaml! {
        // string -> string * string * string -> (string * string) option -> (Context.index * Protocol_hash.t * Chain_id.t * Context_hash.t) Lwt.t
        pub fn context_init(data_dir: String, genesis: (String, String, String), sandbox_json_patch_context: Option<(String, String)>) -> Result<(IrminContextIndex, ContextHash), String>;
        pub fn context_close(index: IrminContextIndex);
        pub fn context_mem(ctxt: IrminContext, key: IrminKey) -> bool;
        pub fn context_dir_mem(ctxt: IrminContext, key: IrminKey) -> bool;
        pub fn context_get(ctxt: IrminContext, key: IrminKey) -> Option<OCamlBytes>;
        pub fn context_set(ctxt: IrminContext, key: IrminKey, value: OCamlBytes) -> IrminContext;
        pub fn context_remove_rec(ctxt: IrminContext, key: IrminKey) -> IrminContext;
        pub fn context_copy(ctxt: IrminContext, from: IrminKey, to: IrminKey) -> Option<IrminContext>;
        pub fn context_checkout(idx: IrminContextIndex, ctxt_hash: ContextHash) -> Option<IrminContext>;
        pub fn context_commit(time: OCamlInt64, message: String, ctxt: IrminContext) -> ContextHash;
        pub fn context_get_protocol(ctxt: IrminContext) -> ProtocolHash;
        pub fn context_set_protocol(ctxt: IrminContext, proto_hash: ProtocolHash) -> IrminContext;
        // TODO: (required?)
        // pub fn context_get_test_chain(ctxt: IrminContext) -> TestChainStatus;
        // pub fn context_set_test_chain(ctxt: IrminContext, id: TestChainStatus) -> IrminContext;
        // pub fn context_del_test_chain(ctxt: IrminContext) -> IrminContext;
    }
}

pub fn init<'a>(
    cr: &'a mut OCamlRuntime,
    data_dir: &str,
    genesis: (String, String, String),
    sandbox_json_patch_context: Option<(String, String)>,
) -> Result<(OCaml<'a, IrminContextIndex>, ContextHash), String> {
    ocaml_frame!(
        cr,
        (data_dir_root, genesis_root, sandbox_json_patch_context_root),
        {
            let data_dir = to_ocaml!(cr, data_dir, data_dir_root);
            let genesis = to_ocaml!(cr, genesis, genesis_root);
            let sandbox_json_patch_context = to_ocaml!(
                cr,
                sandbox_json_patch_context,
                sandbox_json_patch_context_root
            );
            match ffi::context_init(cr, data_dir, genesis, sandbox_json_patch_context).to_result() {
                Ok(result) => Ok((result.fst(), result.snd().to_rust())),
                Err(err) => Err(err.to_rust()),
            }
        }
    )
}

pub fn close(cr: &mut OCamlRuntime, index: OCamlRef<IrminContextIndex>) {
    ffi::context_close(cr, index);
}

pub fn mem(cr: &mut OCamlRuntime, ctxt: OCamlRef<IrminContext>, key: Vec<String>) -> bool {
    ocaml_frame!(cr, (key_root), {
        let key = to_ocaml!(cr, key, key_root);
        ffi::context_mem(cr, ctxt, key).to_rust()
    })
}

pub fn dir_mem(cr: &mut OCamlRuntime, ctxt: OCamlRef<IrminContext>, key: Vec<String>) -> bool {
    ocaml_frame!(cr, (key_root), {
        let key = to_ocaml!(cr, key, key_root);
        ffi::context_dir_mem(cr, ctxt, key).to_rust()
    })
}

pub fn get(
    cr: &mut OCamlRuntime,
    ctxt: OCamlRef<IrminContext>,
    key: &Vec<String>,
) -> Option<Vec<u8>> {
    if key.len() == 1 && key[0] == "protocol" {
        return Some(get_protocol(cr, ctxt));
    }
    ocaml_frame!(cr, (key_root), {
        let key = to_ocaml!(cr, key, key_root);
        ffi::context_get(cr, ctxt, key).to_rust()
    })
}

pub fn get_protocol(cr: &mut OCamlRuntime, ctxt: OCamlRef<IrminContext>) -> Vec<u8> {
    let proto_hash: ProtocolHash = ffi::context_get_protocol(cr, ctxt).to_rust();
    proto_hash.0
}

pub fn set<'a>(
    cr: &'a mut OCamlRuntime,
    ctxt: OCamlRef<IrminContext>,
    key: &Vec<String>,
    value: &Vec<u8>,
) -> OCaml<'a, IrminContext> {
    if key.len() == 1 && key[0] == "protocol" {
        return set_protocol(cr, ctxt, value);
    }
    ocaml_frame!(cr, (key_root, value_root), {
        let key = to_ocaml!(cr, key.to_owned(), key_root);
        let value = to_ocaml!(cr, value, value_root);
        ffi::context_set(cr, ctxt, key, value)
    })
}

pub fn set_protocol<'a>(
    cr: &'a mut OCamlRuntime,
    ctxt: OCamlRef<IrminContext>,
    value: &Vec<u8>,
) -> OCaml<'a, IrminContext> {
    ocaml_frame!(cr, (value_root), {
        let value = ProtocolHash(value.clone());
        let value = to_ocaml!(cr, value, value_root);
        ffi::context_set_protocol(cr, ctxt, value)
    })
}

pub fn remove_rec<'a>(
    cr: &'a mut OCamlRuntime,
    ctxt: OCamlRef<IrminContext>,
    key: &Vec<String>,
) -> OCaml<'a, IrminContext> {
    ocaml_frame!(cr, (key_root), {
        let key = to_ocaml!(cr, key, key_root);
        ffi::context_remove_rec(cr, ctxt, key)
    })
}

pub fn copy<'a>(
    cr: &'a mut OCamlRuntime,
    ctxt: OCamlRef<IrminContext>,
    from_key: &Vec<String>,
    to_key: &Vec<String>,
) -> Option<OCaml<'a, IrminContext>> {
    ocaml_frame!(cr, (from_key_root, to_key_root), {
        let from_key = to_ocaml!(cr, from_key, from_key_root);
        let to_key = to_ocaml!(cr, to_key, to_key_root);
        ffi::context_copy(cr, ctxt, from_key, to_key).to_option()
    })
}

pub fn checkout<'a>(
    cr: &'a mut OCamlRuntime,
    ctxt_idx: OCamlRef<IrminContextIndex>,
    hash: &ContextHash,
) -> Option<OCaml<'a, IrminContext>> {
    ocaml_frame!(cr, (hash_root), {
        let hash = hash_root.keep(hash.to_ocaml(cr));
        ffi::context_checkout(cr, ctxt_idx, hash).to_option()
    })
}

pub fn commit<'a>(
    cr: &'a mut OCamlRuntime,
    time: i64,
    message: &str,
    ctxt: OCamlRef<IrminContext>,
) -> ContextHash {
    ocaml_frame!(cr, (time_root, message_root), {
        let time = to_ocaml!(cr, time, time_root);
        let message = to_ocaml!(cr, message, message_root);
        ffi::context_commit(cr, time, message, ctxt).to_rust()
    })
}
