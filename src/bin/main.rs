use std::env::args;

type Error = Box<dyn std::error::Error>;

use simple_arguments::Arguments;

fn main() -> Result<(), Error> {
	let mut number: usize = 0;
	let mut string: String = String::new();
	let mut boolean: bool = false;
	let mut help: bool = false;
	let mut arguments = Arguments::new(Some("args_tester"));

	let mut args = args();
	let exec = args.next().unwrap();
	let a: Vec<_> = args.collect();

	arguments.add(&mut number, "number", "a number");
	arguments.add(&mut boolean, "bool", "a boolean value");
	arguments.add(&mut string, "string", "a string");
	arguments.add_bool(&mut help, "help", "displays the help message");

	let usage = arguments.usage();
	if let Err(e) = arguments.parse(&a[..]) {
		println!("{}", e);
		print!("{}", usage);
		return Ok(());
	}
	drop(arguments);

	if help {
		println!("{}", usage);
		return Ok(());
	}
	println!("{} {} {}", number, boolean, string);

	Ok(())
}
