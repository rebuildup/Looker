#[cfg(windows)]
fn main() {
    embed_resource::compile("build/icon.rc", embed_resource::NONE);
}

#[cfg(not(windows))]
fn main() {}

