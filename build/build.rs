fn main() {
    if cfg!(target_os = "windows") {
        let mut res = winresource::WindowsResource::new();
        res.set_icon("./build/iron-doom.ico");
        res.set_language(0x0409);
        res.compile().unwrap();
    }
}