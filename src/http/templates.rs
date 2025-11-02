use minijinja::Environment;

pub fn build_env(http_external: String, version: String) -> Environment<'static> {
    let mut env = Environment::new();
    env.set_trim_blocks(true);
    env.set_lstrip_blocks(true);
    env.add_global("base", http_external.clone());
    env.add_global("version", version.clone());
    minijinja_embed::load_templates!(&mut env);
    env
}
