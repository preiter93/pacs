use pacs_core::{Pacs, PacsCommand, PacsError};

fn main() -> Result<(), PacsError> {
    let mut pacs = Pacs::init_home()?;

    // Delete project if it exists then create a new one
    let _ = pacs.delete_project("example");
    pacs.init_project("example", None)?;
    pacs.set_active_project("example")?;

    // Add commands
    pacs.add_command(
        PacsCommand {
            name: "hello_world".into(),
            command: "echo Hello World!".into(),
            cwd: None,
            tag: "misc".into(),
        },
        Some("example"),
    )?;

    pacs.add_command(
        PacsCommand {
            name: "deploy".into(),
            command: "echo Deploy...".into(),
            cwd: None,
            tag: "release".into(),
        },
        Some("example"),
    )?;

    pacs.add_command(
        PacsCommand {
            name: "release".into(),
            command: "echo Release...".into(),
            cwd: None,
            tag: "release".into(),
        },
        Some("example"),
    )?;

    // List all project commands
    println!("[COMMANDS]");
    for cmd in pacs.list(Some("example"), None)? {
        println!("- {} [{}]", cmd.name, cmd.tag);
    }

    // List only release group
    println!("\n[RELEASE]");
    for cmd in pacs
        .list(Some("example"), None)?
        .into_iter()
        .filter(|c| c.tag == "release")
    {
        println!("- {}", cmd.name);
    }

    // Run command
    println!("\nRun command:");
    pacs.run("hello_world", Some("example"), None)?;

    Ok(())
}
