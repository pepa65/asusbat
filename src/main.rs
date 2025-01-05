// main.rs

use anyhow::{Context, Error, Ok, Result};
use clap::{CommandFactory, Parser, Subcommand};
use clap_complete::Shell;
use regex::Regex;
use std::{
	collections::HashMap,
	env,
	ffi::OsStr,
	fs, io,
	path::{Path, PathBuf},
	process::{self, Stdio},
};
use text_template::*;

const NAME: &str = "batlimit";
const UNITPATH: &str = "/etc/systemd/system/batlimit";
const KEYPATH: &str = "/sys/class/power_supply";
const LIMITKEY: &str = "charge_control_end_threshold";
const STARTKEY: &str = "charge_control_start_threshold";
const INTERVAL: u8 = 2;
const TARGETS: [&str; 6] = ["hibernate", "hybrid-sleep", "multi-user", "sleep", "suspend", "suspend-then-hibernate"];
const ENVVAR: &str = "BATLIMIT_BAT";

#[derive(Clone, Debug, PartialEq)]
struct Percent(u8);
impl std::str::FromStr for Percent {
	type Err = String;
	fn from_str(s: &str) -> Result<Self, Self::Err> {
		const ERR_MSG: &str = "Percent must be a number between 1 and 99";
		let percent = s.parse().map_err(|_e| ERR_MSG)?;
		if !(1..=99).contains(&percent) {
			return std::result::Result::Err(ERR_MSG.to_owned());
		}
		std::result::Result::Ok(Self(percent))
	}
}
impl std::fmt::Display for Percent {
	fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
		write!(f, "{}", self.0)
	}
}

#[derive(Parser)]
#[command(
	version,
	about,
	after_help = "\
	Commands can be abbreviated up to their first letter.\n\
	Root privileges required for: limit & clear, persist & unpersist",
	infer_subcommands(true)
)]
#[command(help_template(
	"\
{before-help}{name} {version} - {about}
{usage-heading} {usage}
{all-args}{after-help}
"
))]
struct Cli {
	#[command(subcommand)]
	command: Option<Command>,
}
#[derive(Subcommand)]
enum Command {
	/// Print battery info (default command)
	Info,
	/// Set battery charge limit: PERCENT (1..99)
	Limit { percent: Percent },
	/// Clear charge limit
	Clear,
	/// Persist charge limit with systemd: [PERCENT (1..99)]
	Persist { percent: Option<Percent> },
	/// Unpersist charge limit: disable and remove systemd services
	Unpersist,
	/// Generate completions: SHELL (bash|elvish|fish|powershell|zsh)
	#[command(long_about = "Generate shell completions. Example:
  batlimit shell bash > _bash
  mkdir -p ~/.local/share/bash_completion/completions
  mv _bash ~/.local/share/bash_completion/completions/batlimit")]
	Shell { shell: Shell },
	/// Output the readme file from the repo
	Readme,
}

struct Battery {
	bat_path: PathBuf,
}
impl Battery {
	fn new() -> Result<Self> {
		let bat_name = env::var(ENVVAR);
		if bat_name.is_ok() && !bat_name.clone().unwrap().is_empty() {
			let bat_path = Path::new(KEYPATH).join(bat_name.unwrap());
			return Ok(Self { bat_path });
		};
		for bat_name in ["BAT0", "BAT1", "BATT", "BATC"] {
			let bat_path = Path::new(KEYPATH).join(bat_name);
			if fs::metadata(&bat_path).is_ok() {
				return Ok(Self { bat_path });
			};
		}
		Err(Error::msg("Battery not found".to_owned()))
	}

	/// Write to a file with sudo, like `echo "$2" |sudo tee "$1" >/dev/null
	fn sudo_write<P: AsRef<Path>, C: AsRef<OsStr>>(path: P, contents: C) -> Result<()> {
		let echo = process::Command::new("echo").arg(contents).stdout(Stdio::piped()).spawn()?;
		process::Command::new("sudo")
			.arg("tee")
			.arg(path.as_ref().as_os_str())
			.stdin(Stdio::from(echo.stdout.ok_or("Nothing piped in by echo").map_err(Error::msg)?))
			.stdout(Stdio::null())
			.spawn()?
			.wait()?;
		Ok(())
	}

	fn get_limit(&self) -> Result<Percent> {
		fs::read_to_string(self.bat_path.join(LIMITKEY))
			.context(format!("Failed to read from {}", self.bat_path.join(LIMITKEY).display()))?
			.trim()
			.parse::<Percent>()
			.map_err(|e| Error::msg(format!("Failed to parse battery limit: {e}")))
	}

	fn limit(&self, limit: &Percent) -> Result<()> {
		let old_limit = self.get_limit()?;
		Self::sudo_write(self.bat_path.join(LIMITKEY), limit.to_string())?;
		if old_limit == *limit {
			println!("Unchanged: \x1b[1;32m{limit}\x1b[0m%");
		} else {
			println!("\x1b[1;31m{old_limit}\x1b[0m% -> \x1b[1;32m{limit}\x1b[0m%");
		}
		Ok(())
	}

	fn clear(&self) -> Result<()> {
		Self::sudo_write(self.bat_path.join(LIMITKEY), "100")?;
		println!("Cleared charge limit");
		Ok(())
	}

	fn persist(&self, percent: Option<Percent>) -> Result<()> {
		let limit = if percent.is_none() { self.get_limit()?.clone().to_string() } else { percent.clone().unwrap().to_string() };
		let mut values = HashMap::new();
		let startval = limit.parse::<u8>().unwrap() - INTERVAL;
		let startkey = self.bat_path.join(STARTKEY).display().to_string();
		let startstr = format!("echo {} >{}; ", startval.to_string().as_str(), &startkey);
		if fs::exists(self.bat_path.join(STARTKEY)).unwrap() {
			values.insert("start", startstr.as_str());
		} else {
			values.insert("start", "");
		};
		values.insert("limit", limit.as_str());
		let key = self.bat_path.join(LIMITKEY).display().to_string();
		values.insert("path", &key);
		// Compile-time include from the repo root
		let template = Template::from(include_str!("../unit.service"));
		for target in TARGETS {
			values.insert("target", target);
			let content = template.fill_in(&values).to_string();
			let path = format!("{UNITPATH}-{target}.service");
			Self::sudo_write(&path, content)?;
			let unit = format!("{NAME}-{target}.service");
			process::Command::new("sudo").args(format!("systemctl enable --now {unit} --quiet").split(' ')).spawn()?.wait()?;
		}
		println!("Systemd services created and enabled with limit {limit}");
		Ok(())
	}

	fn unpersist(&self) -> Result<()> {
		for target in TARGETS {
			let unit = format!("{NAME}-{target}.service");
			let path = format!("{UNITPATH}-{target}.service");
			if fs::metadata(&path).is_ok() {
				process::Command::new("sudo").args(format!("systemctl disable --now {unit} --quiet").split(' ')).spawn()?.wait()?;
				process::Command::new("sudo").arg("rm").arg(path).spawn()?.wait()?;
			}
		}
		println!("Systemd services disabled and removed");
		Ok(())
	}

	fn get_persist(&self) -> Option<Percent> {
		let mut percent = Percent(0);
		for target in TARGETS {
			let path = format!("{UNITPATH}-{target}.service");
			let file = fs::read_to_string(path).ok()?;
			if Some(file.clone()).is_none() {
				return Some(Percent(0));
			}
			let re = Regex::new(format!("(?m)^ExecStart=/bin/sh -c 'echo ([0-9]+) >{KEYPATH}/BAT./{LIMITKEY}'$").as_str()).unwrap();
			let pct = re.captures(&file)?.get(1)?.as_str().parse::<Percent>().unwrap();
			if percent == Percent(0) {
				percent = pct;
			} else if percent != pct {
				return Some(Percent(0));
			}
		}
		Some(percent)
	}

	fn info(&self) {
		const INFO: [(&str, &str, &str); 16] = [
			("manufacturer", "Brand", ""),
			("model_name", "Model", ""),
			("technology", "Battery Type", ""),
			("status", "Charge Status", ""),
			("capacity_level", "Battery Condition", ""),
			("charge_full", "Current Max. Capacity", " μAh"),
			("power_now", "Current Max. Capacity", " μAh"),
			("energy_now", "Current Max. Capacity", " μAh"),
			("energy_full", "Current Max. Capacity", " μAh"),
			("charge_full_design", "Design Max. Capacity", " μAh"),
			("energy_full_design", "Design Max. Capacity", " μAh"),
			("voltage_min_design", "Min. Voltage", " μV"),
			("voltage_now", "Current Voltage", " μV"),
			("capacity", "Charge Level", "%"),
			(STARTKEY, "Charge Start", "%"),
			(LIMITKEY, "Charge Limit", "%"),
		];
		let info = INFO
			.iter()
			.filter_map(|v| fs::read_to_string(self.bat_path.join(v.0)).ok().map(|value| (v.1, value.trim().to_owned(), v.2)))
			.collect::<Vec<_>>();
		let pad_size = info.iter().map(|(file, _, _)| file.len()).max().unwrap_or(0);
		let info_string = info.iter().map(|(file, value, unit)| format!("{:<pad_size$}  {}{}", file, value, unit)).collect::<Vec<_>>().join("\n");
		let path = &self.bat_path.display().to_string();
		let bat = path.split('/').last().unwrap();
		println!("[{bat}]");
		if !info_string.is_empty() {
			println!("{info_string}");
		};
		let persiststr = "Persist state";
		let persist = self.get_persist();
		if persist != Some(Percent(0)) {
			if persist.is_none() {
				println!("{persiststr:<pad_size$}  NO");
			} else {
				println!("{persiststr:<pad_size$}  {}%", persist.unwrap());
			}
		} else {
			println!("{persiststr:<pad_size$}  INCONSISTENT");
		}
		let healthstr = "health";
		if info.len() > 6 {
			// position of Current Max. Capacity and Current Design Capacity in info depends on which keys are found
			let health = 100 * info[5].1.parse::<u32>().unwrap_or(0) / info[6].1.parse::<u32>().unwrap_or(1);
			println!("{healthstr:<pad_size$}  {health}%");
		} else {
			println!("{healthstr:<pad_size$}  NO INFO");
		};
	}
}

fn main() -> Result<()> {
	let args = Cli::parse();
	let battery = Battery::new()?;
	match args.command {
		Some(Command::Limit { percent }) => {
			battery.limit(&percent)?;
		}
		Some(Command::Clear) => {
			battery.clear()?;
		}
		Some(Command::Persist { percent }) => {
			battery.persist(percent)?;
		}
		Some(Command::Readme) => {
			// Compile-time include from the repo root
			print!("{}", include_str!("../README.md"));
		}
		Some(Command::Unpersist) => {
			battery.unpersist()?;
		}
		Some(Command::Shell { shell }) => {
			clap_complete::generate(shell, &mut Cli::command(), env!("CARGO_PKG_NAME"), &mut io::stdout());
		}
		Some(Command::Info) => battery.info(),
		_ => battery.info(),
	};
	Ok(())
}
