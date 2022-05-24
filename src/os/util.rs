pub fn spawn_and_exit(exe_path: &String, args: Vec<String>, url: &str) {
    let mut command_arguments = args;
    command_arguments.push(String::from(url));

    std::process::Command::new(exe_path)
        .args(command_arguments)
        .spawn()
        .expect(
            format!("Couldn't run browser program at {}", exe_path)
                .to_owned()
                .as_str(),
        );
    std::process::exit(0);
}
