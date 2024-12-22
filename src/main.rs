use std::str::FromStr;

use chrono::NaiveTime;
use clap::{Parser, Subcommand};
use hex::ToHex;
use lightning_time::{LightningTime, LightningTimeColors};

/// A CLI for Lightning Time. Allows for easy conversion to/from ISO 8601. Omit the subcommand to print the current time.
#[derive(Debug, Parser)]
#[command(version, about, long_about)]
struct Args {
    #[command(subcommand)]
    subcommand: Option<Commands>,
}

#[derive(Debug, Subcommand)]
enum Commands {
    /// Converts Lightning Time to comma-separated hex colors, currently only supports default theme
    Colors {
        /// The time to convert to colors. If omitted uses the current time
        time: Option<String>,
    },
    /// Converts Lightning Time from %H:%M:%S%.f (ISO 8601 standard)
    From { iso: String },
    /// Converts Lightning Time to %H:%M:%S%.f (ISO 8601 standard)
    To { time: String },
}

fn main() -> Result<(), String> {
    let args = Args::parse();

    match args.subcommand {
        Some(cmd) => match cmd {
            Commands::Colors { time } => {
                let time = match time
                    .map(|t| LightningTime::from_str(&t))
                    .unwrap_or_else(|| Ok(LightningTime::now()))
                {
                    Ok(lt) => lt,
                    Err(e) => {
                        return Err(format!("Failed to parse Lightning Time: {e}"));
                    }
                };

                let LightningTimeColors { bolt, zap, spark } = time.colors(&Default::default());
                println!(
                    "#{},#{},#{}",
                    bolt.encode_hex::<String>(),
                    zap.encode_hex::<String>(),
                    spark.encode_hex::<String>()
                );
            }
            Commands::From { iso } => {
                let parsed = chrono::NaiveTime::parse_from_str(&iso, "%H:%M:%S%.f")
                    .map_err(|e| e.to_string())?;
                println!("{}", LightningTime::from(parsed));
            }
            Commands::To { time } => {
                let parsed = LightningTime::from_str(&time)
                    .map_err(|e| format!("Failed to parse Lightning Time: {e}"))?;

                println!("{}", NaiveTime::from(parsed));
            }
        },
        None => println!("{}", LightningTime::now()),
    }

    Ok(())
}
