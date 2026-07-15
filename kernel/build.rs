use cc::Build;

const C_FILES: &[&str] = &[
    "src/user/c_api/c_code.c"
];

fn main() {
    println!("cargo:rerun-if-changed=src/user/c_api");

    for file in C_FILES {
        let compile = file.split('/').last().unwrap().replace(".c", "");

        Build::new()
            .file(file)
            .compile(compile.as_str());
    }
}