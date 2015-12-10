
macro_rules! expand {
    ($file_in:expr, $file_out:expr) => {{

        let out_dir = env::var_os("OUT_DIR").unwrap();
        let mut registry = syntex::Registry::new();
        serde_codegen::register(&mut registry);

        let src  = Path::new($file_in);
        let dest = Path::new(&out_dir).join($file_out);
        registry.expand("", &src, &dest).unwrap();
    }}
}


#[cfg(not(feature = "serde_macros"))]
mod inner {

    extern crate syntex;
    extern crate serde_codegen;
    use std::env;
    use std::path::Path;

    pub fn main() {
        expand!("src/config.rs.in", "config.rs");
        expand!("src/trello.rs.in", "trello.rs");
    }
}

#[cfg(feature = "serde_macros")]
mod inner {
    pub fn main() {}
}

fn main() {
    inner::main();
}