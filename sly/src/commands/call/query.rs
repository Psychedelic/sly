use crate::lib::candid::CandidParser;
use crate::lib::command::AsyncCommand;
use crate::lib::env::Env;
use crate::lib::utils;
use anyhow::{bail, Context, Result};
use async_trait::async_trait;
use candid::{
    parser::value::IDLValue,
    types::{Function, Type},
    IDLArgs, Principal, TypeEnv,
};
use clap::Clap;
use codespan_reporting::term;
use codespan_reporting::term::termcolor::{ColorChoice, StandardStream};
use ic_agent::{agent::agent_error::HttpErrorPayload, AgentError};

#[derive(Clap, Debug)]
pub struct QueryOpts {
    /// Canister id
    canister_id: String,
    /// Method name to call on the canister
    method_name: String,
    /// Argument to pass to the method, in Candid textual format
    argument: Option<String>,
    /// Path to a candid file to analyze the argument
    #[clap(long, short)]
    candid: Option<String>,
    /// The type of input (raw or idl).
    #[clap(long, short, possible_values = & (["raw", "idl"]), default_value = "idl")]
    in_type: ArgType,
    /// The type of output (raw or idl).
    #[clap(long, short, possible_values = & (["raw", "idl"]), default_value = "idl")]
    out_type: ArgType,
    /// An optional field to set the expiry time on requests. Can be a human
    /// readable time (like `100s`) or a number of seconds.
    #[clap(long)]
    ttl: Option<humantime::Duration>,
}

#[async_trait]
impl AsyncCommand for QueryOpts {
    async fn async_exec(self, env: &mut Env) -> Result<()> {
        let canister_id =
            &Principal::from_text(self.canister_id).context("Invalid canister principal format")?;

        let method_name = &self.method_name;

        let argument = self.argument.as_deref();

        let in_type = &self.in_type;

        let out_type = &self.out_type;

        let method_type = match &self.candid {
            None => None,
            Some(path) => get_candid_type(path, method_name)
                .context("Failed to get method type from candid file")?,
        };

        let arg_blob =
            blob_from_arguments(argument, in_type, &method_type).context("Invalid arguments")?;

        let effective_canister_id =
            utils::get_effective_canister_id(method_name, &arg_blob, canister_id)
                .context("Failed to get effective_canister_id for this call")?;

        let agent = env.create_agent().await?;
        let mut builder = agent.query(canister_id, method_name);

        let expire_after = self.ttl.map(|ht| ht.into());
        if let Some(d) = expire_after {
            builder.expire_after(d);
        }

        let result = builder
            .with_arg(&arg_blob)
            .with_effective_canister_id(effective_canister_id)
            .call()
            .await;

        match result {
            Ok(blob) => {
                print_idl_blob(&blob, out_type, &method_type)
                    .context("Failed to print result blob")?;
            }
            Err(AgentError::TransportError(_)) => return Ok(()),
            Err(AgentError::HttpError(HttpErrorPayload {
                status,
                content_type,
                content,
            })) => {
                let mut error_message =
                    format!("Server returned an HTTP Error:\n  Code: {}\n", status);
                match content_type.as_deref() {
                    None => {
                        error_message.push_str(&format!("  Content: {}\n", hex::encode(content)))
                    }
                    Some("text/plain; charset=UTF-8") | Some("text/plain") => {
                        error_message.push_str("  ContentType: text/plain\n");
                        error_message.push_str(&format!(
                            "  Content:     {}\n",
                            String::from_utf8_lossy(&content)
                        ));
                    }
                    Some(x) => {
                        error_message.push_str(&format!("  ContentType: {}\n", x));
                        error_message
                            .push_str(&format!("  Content:     {}\n", hex::encode(&content)));
                    }
                }
                bail!(error_message);
            }
            Err(s) => Err(s).context("Got an error when make the canister call")?,
        }

        Ok(())
    }
}

fn get_candid_type(idl_path: &str, method_name: &str) -> Result<Option<(TypeEnv, Function)>> {
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

fn check_candid_file(idl_path: &str) -> Result<(TypeEnv, Option<Type>)> {
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

fn blob_from_arguments(
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

fn print_idl_blob(
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
enum ArgType {
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
