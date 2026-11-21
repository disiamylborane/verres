use calculator_parsers::CalculatorObject;

fn main() -> rustyline::Result<()> {
    let mut calculator = CalculatorObject::new();
    let mut rl = rustyline::DefaultEditor::new()?;

    loop {
        let readline = rl.readline(">> ");
        match readline {
            Ok(line) => {
                rl.add_history_entry(line.as_str())?;
                let input = line.trim();

                if input.is_empty() {
                    continue
                }

                let result = calculator.execute(&line);
                println!("{}", result);
            }
            Err(rustyline::error::ReadlineError::Interrupted) => {
                println!("Ctrl+C");
                break;
            }
            Err(rustyline::error::ReadlineError::Eof) => {
                println!("Ctrl+D");
                break;
            }
            Err(err) => {
                println!("Error: {:?}", err);
            }
        }
    }

    Ok(())
}
