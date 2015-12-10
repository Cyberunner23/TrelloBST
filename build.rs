use std::path::Path;

//macro_rules! expand {
//    (&mut $registry:expr, $out_dir:expr, $file_in:expr, $file_out:expr) => {{
//        let mut src  = Path::new($file_in);
//        let mut dest = Path::new(&$out_dir).join($file_out);
//        $registry.expand("", &src, &dest).unwrap();
//    }}
//}


#[cfg(not(feature = "serde_macros"))]
mod inner {

    extern crate syntex;
    extern crate serde_codegen;
    use std::env;
    use std::path::Path;

    pub fn main() {
        let out_dir = env::var_os("OUT_DIR").unwrap();
        let mut registry = syntex::Registry::new();
        serde_codegen::register(&mut registry);

        let mut src_config  = Path::new("src/config.rs.in");
        let mut dest_config = Path::new(&out_dir).join("config.rs");
        registry.expand("", &src_config, &dest_config).unwrap();

        let mut src_trello  = Path::new("src/trello.rs.in");
        let mut dest_trello = Path::new(&out_dir).join("trello.rs");
        registry.expand("", &src_trello, &dest_trello).unwrap();
    }
}

#[cfg(feature = "serde_macros")]
mod inner {
    pub fn main() {}
}

fn main() {
    inner::main();
}