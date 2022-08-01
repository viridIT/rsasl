//! Interop testing client
//!
//! This client allows testing interoperability between different SASL implementations.

use miette::{IntoDiagnostic, WrapErr};
use rsasl::callback::{CallbackError, Context, Request, SessionCallback, SessionData};
use rsasl::prelude::*;
use rsasl::property::*;
use rsasl::validate::{NoValidation, Validate, ValidationError};
use std::io;
use std::io::Cursor;
use rsasl::mechanisms::scram::properties::*;

struct EnvCallback;
impl SessionCallback for EnvCallback {
    fn callback(
        &self,
        session_data: &SessionData,
        context: &Context,
        request: &mut Request<'_>,
    ) -> Result<(), SessionError> {
        fn var(key: &'static str) -> Result<String, SessionError> {
            std::env::var(key).map_err(|_| CallbackError::NoValue.into())
        }
        if request.is::<OverrideCBType>() {
            let cbtype = var("RSASL_CBNAME")?;
            request.satisfy::<OverrideCBType>(cbtype.as_str())?;
        } else if request.is::<ChannelBindings>() {
            let cbdata = var("RSASL_CBDATA")?;
            request.satisfy::<ChannelBindings>(cbdata.as_bytes())?;
        } else if request.is::<PasswordHash>() {
            if let Some("username") = context.get_ref::<AuthId>() {
                let data = base64::decode("p5gegiYLXQDvq+yCjFEV5WN/eu8i5rfy3/J5YKhyQgw=").unwrap();
                request.satisfy::<PasswordHash>(&data[..])?;
            }
        } else if request.is::<HashIterations>() {
            request.satisfy::<HashIterations>(&4096)?;
        } else if request.is::<Salt>() {
            request.satisfy::<Salt>(&[
                0xc0, 0x3d, 0x33, 0xfd, 0xce, 0x5d, 0xed, 0x2e,
                0x2a, 0xeb, 0x8e, 0xbc, 0x3b, 0x3d, 0x62, 0xb2
            ])?;
        }
        Ok(())
    }
    fn validate(
        &self,
        session_data: &SessionData,
        context: &Context,
        validate: &mut Validate<'_>,
    ) -> Result<(), ValidationError> {
        if session_data.mechanism().mechanism.as_str() == "PLAIN" {
            let authid = context.get_ref::<AuthId>();
            let authzid = context.get_ref::<AuthzId>();
            let password = context.get_ref::<Password>();
            println!(
                "plain validation; authid={:?}, authzid={:?}, password={:?}",
                authid, authzid, password
            );
        }
        Ok(())
    }
}

pub fn main() -> miette::Result<()> {
    let config = SASLConfig::builder()
        .with_default_mechanisms()
        .with_default_sorting()
        .with_callback(EnvCallback)
        .into_diagnostic()
        .wrap_err("Failed to generate SASL config")?;

    let mut server = SASLServer::<NoValidation>::new(config);
    for mech in server.get_available() {
        print!("{} ", mech.mechanism.as_str());
    }
    println!();

    let mut line = String::new();
    io::stdin()
        .read_line(&mut line)
        .into_diagnostic()
        .wrap_err("failed to read line from stdin")?;
    let selected = Mechname::new(line.trim().as_bytes())
        .into_diagnostic()
        .wrap_err(format!("selected mechanism '{}' is invalid", line))?;

    let mut session = server
        .start_suggested(selected)
        .into_diagnostic()
        .wrap_err("Failed to start SASL server session")?;

    let mut input = if session.are_we_first() {
        None
    } else {
        // Then we wait on the first line sent by the client.
        let mut line = String::new();
        io::stdin()
            .read_line(&mut line)
            .into_diagnostic()
            .wrap_err("failed to read line from stdin")?;
        Some(line)
    };

    while {
        let mut out = Cursor::new(Vec::new());
        let (state, _) = session
            .step64(input.as_deref().map(|s| s.trim().as_bytes()), &mut out)
            .into_diagnostic()
            .wrap_err("Unexpected error occurred during stepping the session")?;
        let mut output = out.into_inner();

        let output =
            String::from_utf8(output).expect("base64 encoded output is somehow not valid UTF-8");
        println!("{}", output);

        state.is_running()
    } {
        let mut line = String::new();
        io::stdin()
            .read_line(&mut line)
            .into_diagnostic()
            .wrap_err("failed to read line from stdin")?;
        input = Some(line);
    }

    Ok(())
}
