use pacs_core::{Pacs, PacsCommand, PacsError, Scope};

fn main() -> Result<(), PacsError> {
    let mut pacs = Pacs::init_home()?;

    // Delete project if it exists then create a new one
    let _ = pacs.delete_project("example");
    pacs.init_project("example", None)?;

    // Add commands
    pacs.add_command(
        PacsCommand {
            name: "hello_world".into(),
            command: "echo Hello World!".into(),
            cwd: None,
            tag: "misc".into(),
        },
        Scope::Project("example"),
    )?;

    pacs.add_command(
        PacsCommand {
            name: "deploy".into(),
            command: "echo Deploy...".into(),
            cwd: None,
            tag: "release".into(),
        },
        Scope::Project("example"),
    )?;

    pacs.add_command(
        PacsCommand {
            name: "release".into(),
            command: "echo Release...".into(),
            cwd: None,
            tag: "release".into(),
        },
        Scope::Project("example"),
    )?;

    // List all project commands
    println!("[COMMANDS]");
    for cmd in pacs.list_commands(Scope::Project("example"), None)? {
        println!("- {} [{}]", cmd.name, cmd.tag);
    }

    // List only release group
    println!("\n[RELEASE]");
    for cmd in pacs.list_by_tag(Scope::Project("example"), "release")? {
        println!("- {}", cmd.name);
    }

    // Run command
    println!("\nRun command:");
    pacs.run("hello_world", Scope::Project("example"))?;

    Ok(())
}
