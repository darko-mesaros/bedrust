use crate::utils::{check_for_config, initialize_config, print_warning};
use std::io;
use std::io::Write;

pub fn prompt_init_config() -> Result<(), anyhow::Error> {
    match check_for_config() {
        Ok(config) => match config {
            true => {
                print_warning("****************************************");
                print_warning("WARNING:");
                println!("You are trying to initialize the Bedrust configuration");
                println!("This will overwrite your configuration files in $HOME/.config/bedrust/");
                print!("ARE YOU SURE YOU WANT DO TO THIS? Y/N: ");
                io::stdout().flush()?; // so the answers are typed on the same line as above

                let mut confirmation = String::new();
                io::stdin().read_line(&mut confirmation)?;
                if confirmation.trim().eq_ignore_ascii_case("y") {
                    print_warning("I ask AGAIN");
                    print!("ARE YOU SURE YOU WANT DO TO THIS? Y/N: ");
                    io::stdout().flush()?; // so the answers are typed on the same line as above

                    let mut confirmation = String::new();
                    io::stdin().read_line(&mut confirmation)?;

                    if confirmation.trim().eq_ignore_ascii_case("y") {
                        println!("----------------------------------------");
                        println!("📜 | Initializing Bedrust configuration.");
                        initialize_config()?;
                    }
                }
            },
            false => {
                println!("----------------------------------------");
                println!("📜 | Initializing Bedrust configuration.");
                initialize_config()?;
            },
        },
        Err(e) => eprintln!("There was an issue checking for errors: {}",e)
    }
    print_warning("Bedrust will now exit");
    std::process::exit(0);
} 
