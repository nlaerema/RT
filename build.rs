use wesl::ModulePath;

fn main() {
    wesl::Wesl::new("src/shaders").build_artifact(&ModulePath::from_path("/vert.wesl"), "vertex_shader");
    wesl::Wesl::new("src/shaders").build_artifact(&ModulePath::from_path("/frag.wesl"), "fragment_shader");
}