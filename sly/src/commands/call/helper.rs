use anyhow::{bail, Context, Result};
use candid::parser::value::IDLValue;
use candid::types::{Function, Type};
use candid::{IDLArgs, TypeEnv};
use clap::Parser as Clap;
use codespan_reporting::term;
use codespan_reporting::term::termcolor::{ColorChoice, StandardStream};
use ic_agent::{agent::agent_error::HttpErrorPayload, AgentError};

use crate::lib::candid::CandidParser;
use crate::lib::utils;

pub fn get_candid_type(idl_path: &str, method_name: &str) -> Result<Option<(TypeEnv, Function)>> {
    let (env, ty) = check_candid_file(idl_path)
        .with_context(|| format!("Failed when checking candid file: {}", idl_path))?;
    match ty {
        None => Ok(None),
        Some(actor) => {
            let method = env
                .get_method(&actor, method_name)
                .with_context(|| format!("Failed to get method: {}", method_name))?
                .clone();
            Ok(Some((env, method)))
        }
    }
}

pub fn check_candid_file(idl_path: &str) -> Result<(TypeEnv, Option<Type>)> {
    let mut candid_parser = CandidParser::default();
    let maybe_env = utils::result_flatten(
        candid_parser
            .parse(idl_path)
            .map(|_| candid_parser.construct_type_env()),
    );
    if let Err(diagnostic) = maybe_env {
        let writer = StandardStream::stderr(ColorChoice::Always);
        let config = codespan_reporting::term::Config::default();
        term::emit(&mut writer.lock(), &config, &candid_parser, &diagnostic)?;
        bail!("Candid check failed.");
    }
    Ok((
        candid_parser.get_type_env().clone(),
        candid_parser.get_service_for(idl_path).clone(),
    ))
}

pub fn blob_from_arguments(
    arguments: Option<&str>,
    arg_type: &ArgType,
    method_type: &Option<(TypeEnv, Function)>,
) -> Result<Vec<u8>> {
    let mut buffer = Vec::new();
    let arguments = if arguments == Some("-") {
        use std::io::Read;
        std::io::stdin().read_to_end(&mut buffer).unwrap();
        std::str::from_utf8(&buffer).ok()
    } else {
        arguments
    };

    match arg_type {
        ArgType::Raw => {
            let bytes = hex::decode(&arguments.unwrap_or(""))
                .context("Argument is not a valid hex string")?;
            Ok(bytes)
        }
        ArgType::Idl => {
            let arguments = arguments.unwrap_or("()");
            let args = arguments.parse::<IDLArgs>();
            let typed_args = match method_type {
                None => args
                    .context("Failed to parse arguments with no method type info")?
                    .to_bytes(),
                Some((env, func)) => {
                    let first_char = arguments.chars().next();
                    let is_candid_format = first_char.map_or(false, |c| c == '(');
                    // If parsing fails and method expects a single value, try parsing as IDLValue.
                    // If it still fails, and method expects a text type, send arguments as text.
                    let args = args.or_else(|e| {
                        if func.args.len() == 1 && !is_candid_format {
                            let is_quote = first_char.map_or(false, |c| c == '"');
                            if candid::types::Type::Text == func.args[0] && !is_quote {
                                Ok(IDLValue::Text(arguments.to_string()))
                            } else {
                                arguments.parse::<IDLValue>()
                            }
                            .map(|v| IDLArgs::new(&[v]))
                        } else {
                            Err(e)
                        }
                    });
                    args.context("Failed to parse arguments with method type info")?
                        .to_bytes_with_types(env, &func.args)
                }
            }
            .context("Failed to serialize Candid values")?;
            Ok(typed_args)
        }
    }
}

pub fn print_idl_blob(
    blob: &[u8],
    output_type: &ArgType,
    method_type: &Option<(TypeEnv, Function)>,
) -> Result<()> {
    let hex_string = hex::encode(blob);
    match output_type {
        ArgType::Raw => {
            println!("{}", hex_string);
        }
        ArgType::Idl => {
            let result = match method_type {
                None => candid::IDLArgs::from_bytes(blob),
                Some((env, func)) => candid::IDLArgs::from_bytes_with_types(blob, env, &func.rets),
            };
            println!(
                "{}",
                result.with_context(|| format!("Failed to deserialize blob 0x{}", hex_string))?
            );
        }
    }
    Ok(())
}

#[derive(Clap, Debug)]
pub enum ArgType {
    Idl,
    Raw,
}

impl std::str::FromStr for ArgType {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "idl" => Ok(ArgType::Idl),
            "raw" => Ok(ArgType::Raw),
            other => Err(format!("invalid argument type: {}", other)),
        }
    }
}

pub fn print_agent_result(
    result: Result<Vec<u8>, AgentError>,
    out_type: &ArgType,
    method_type: &Option<(TypeEnv, Function)>,
) -> Result<()> {
    match result {
        Ok(blob) => {
            print_idl_blob(&blob, out_type, method_type).context("Failed to print result blob")?;
            Ok(())
        }
        Err(AgentError::TransportError(_)) => Ok(()),
        Err(AgentError::HttpError(HttpErrorPayload {
            status,
            content_type,
            content,
        })) => {
            let mut error_message = format!("Server returned an HTTP Error:\n  Code: {}\n", status);
            match content_type.as_deref() {
                None => {
                    error_message.push_str(&format!(
                        "  Content:     {}\n",
                        String::from_utf8_lossy(&content)
                    ));
                }
                Some(x) => {
                    error_message.push_str(&format!("  ContentType: {}\n", x));
                    error_message.push_str(&format!(
                        "  Content:     {}\n",
                        String::from_utf8_lossy(&content)
                    ));
                }
            }
            bail!(error_message);
        }
        Err(s) => Err(s).context("Got an error when make the canister call")?,
    }
}
