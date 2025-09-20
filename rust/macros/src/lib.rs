use proc_macro::TokenStream;
use quote::quote;
use syn::{ parse_macro_input, ItemFn, LitStr };

#[proc_macro_attribute]
pub fn export_plugin(args: TokenStream, item: TokenStream) -> TokenStream {
    let input = parse_macro_input!(item as ItemFn);

    // default export name = function name
    let export_name = if args.is_empty() {
        input.sig.ident.to_string()
    } else {
        // accept single string literal: #[export_plugin("synth")]
        let s = parse_macro_input!(args as LitStr);
        s.value()
    };

    let fn_ident = &input.sig.ident;
    let vis = &input.vis;
    let sig = &input.sig;
    let block = &input.block;

    let wrapper_ident = syn::Ident::new(&export_name, fn_ident.span());

    let expanded =
        quote! {
        #vis #sig #block

        #[no_mangle]
        pub extern "C" fn #wrapper_ident(
            out_ptr: *mut f32,
            out_len: i32,
            freq: f32,
            amp: f32,
            duration_ms: i32,
            sample_rate: i32,
            channels: i32,
        ) {
            if out_ptr.is_null() { return; }
            let out_len_usz = out_len.max(0) as usize;
            let ch = channels.max(1) as usize;
            let frames = (out_len_usz / ch) as u32;
            let params = devalang_bindings::BufferParams { sample_rate: sample_rate.max(1) as u32, channels: channels.max(1) as u32, frames };
            unsafe {
                let out = core::slice::from_raw_parts_mut(out_ptr, out_len_usz);
                #fn_ident(out, params, devalang_bindings::Note::default(), freq, amp, duration_ms.max(1) as u32);
            }
        }
    };

    TokenStream::from(expanded)
}
