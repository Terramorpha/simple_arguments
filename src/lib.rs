/*!

This crate is usefull if you want to easily have an okay cli interface for your program.

you can use it like this:
```rust
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

```

here, instead of only defining the arguments' names and converting them to the
correct type after, we make use of a special trait `Filler` (which is
implemented for all FromStr types) to automatically convert the arguments.


 */

use std::collections::HashMap;
use std::io::Write;
use std::str::FromStr;

#[derive(Debug, Clone)]
pub enum ArgError {
	Err(String),
	OutOfArgs,
}

/// This trait is the one which has to be implemented by every argument passed
/// to the Arguments struct.
pub trait Filler {
	fn fill(&mut self, s: &mut dyn Iterator<Item = &str>) -> Result<(), ArgError>;
	fn type_name(&self) -> &'static str {
		"unknown"
	}
}

struct BooleanFlag<'a> {
	value: &'a mut bool,
}

impl<'a> Filler for BooleanFlag<'a> {
	fn fill(&mut self, s: &mut dyn Iterator<Item = &str>) -> Result<(), ArgError> {
		*self.value = true;
		Ok(())
	}
	fn type_name(&self) -> &'static str {
		"flag"
	}
}

impl<T: FromStr> Filler for &mut T {
	fn fill(&mut self, s: &mut dyn Iterator<Item = &str>) -> Result<(), ArgError> {
		use std::any::type_name;

		let item = s.next().ok_or(ArgError::OutOfArgs)?;

		**self = T::from_str(item)
			.or_else(|err| Err(ArgError::Err(format!("error parsing {}", type_name::<T>()))))?;
		Ok(())
	}

	fn type_name(&self) -> &'static str {
		use std::any::type_name;
		&type_name::<Self>()[5..]
	}
}

struct Flag<'a> {
	description: String,
	value: Box<dyn Filler + 'a>,
}

/// the main struct which is responsible for managing all argument parsing logic
pub struct Arguments<'a> {
	flags: HashMap<String, Flag<'a>>,
	name: Option<String>,
}

impl<'a> Arguments<'a> {
	/// initialises a new `Arguments` struct. You can give it an optional
	/// executable name which will be used to create the usage String.
	pub fn new(name: Option<&str>) -> Self {
		Arguments {
			flags: HashMap::new(),
			name: name.map(|v| v.to_owned()),
		}
	}

	/// adds a new argument to the struct. filler must implement filler (all
	/// &mut T where T: FromStr implement Filler)
	pub fn add<T, S>(&mut self, filler: T, name: S, description: &str)
	where
		T: Filler + 'a,
		S: ToString,
	{
		let new_flag = Flag {
			description: description.to_owned(),
			value: Box::new(filler),
		};

		self.flags.insert(name.to_string(), new_flag);
	}

	/// fills every argument with the given arguments and returns a vector of
	/// all the arguments that weren't taken by any flag. If it fails, returns a
	/// string describing a parsing error or a lack of remaining arguments
	pub fn parse<S: AsRef<str>>(&mut self, arguments: &[S]) -> Result<Vec<String>, String> {
		let mut flags: Vec<&str> = Vec::new();
		let mut values: Vec<&str> = Vec::new();

		for a in arguments.iter().map(|s| s.as_ref()) {
			if a.starts_with("--") {
				flags.push(&a[2..]);
			} else {
				values.push(a);
			}
		}

		let mut values_iter = values.into_iter();
		for f in flags.into_iter() {
			let mut flag = self
				.flags
				.get_mut(f)
				.ok_or_else(|| format!("invalid flag: {}", &f))?;

			flag.value
				.fill(&mut values_iter)
				.or_else(|err| Err(format!("{}: {:?}", f, err)))?;
		}

		Ok(values_iter.map(|s| s.to_owned()).collect())
	}

	/// generates a usage string
	pub fn usage(&self) -> String {
		let mut o = String::new();

		let mut flags: Vec<_> = self.flags.iter().collect();
		flags.sort_by_key(|(name, fl)| name.to_owned());

		let max_len = flags.iter().fold(0, |acc, v| acc + v.0.len());
		if let Some(exec) = &self.name {
			o.push_str(&format!("usage:\n{} [flags] args...\n", exec));
		}
		for i in flags {
			o.push_str(&format!(
				"\t--{: <20} ({}) {}\n",
				i.0,
				i.1.value.type_name(),
				i.1.description,
				//width = max_len + 4
			));
		}
		o
	}

	/// since the default implementation of Filller for &mut bool would require
	/// the user tu write `./program --boolean-flag true` instead of just
	/// `./program --boolean-flag`, this functions adds a flag that, when given,
	/// will autmatically set the given variable to true
	pub fn add_bool<S: ToString>(&mut self, b: &'a mut bool, name: S, description: &str) {
		let filler = BooleanFlag { value: b };
		let flag = Flag {
			description: description.to_owned(),
			value: Box::new(filler),
		};
		self.flags.insert(name.to_string(), flag);
	}
}

#[test]
fn simple_test() {
	let mut number: usize = 12;
	let mut string: String = String::new();
	let mut boolean: bool = false;
	let mut arguments = Arguments::new();

	let a = &["--bool", "true", "--number", "123", "--string", "penis"];

	arguments.add(&mut number, "number", "a number");
	arguments.add(&mut boolean, "bool", "a boolean value");
	arguments.add(&mut string, "string", "a string");
	arguments.parse(a).unwrap();
	drop(arguments);

	assert_eq!(number, 123);
	assert_eq!(boolean, true);
	assert_eq!(string, "penis");
}
