use std::io::Write;
use std::process::{Command, Stdio};
use crate::html::{HtmlOutput, html_raw};

/// Run content through a filter specified by a filter string like "exec:/path/to/script"
/// or "lua:/path/to/script.lua". The filter receives `args` as command-line arguments
/// (for exec) or function arguments (for Lua), and `input` as stdin/filter_write content.
/// The filter output is written to HtmlOutput.
pub fn run_filter(spec: &str, args: &[&str], input: &[u8]) {
    if let Some(cmd) = spec.strip_prefix("exec:") {
        run_exec_filter(cmd, args, input);
    } else if let Some(script) = spec.strip_prefix("lua:") {
        run_lua_filter(script, args, input);
    } else {
        // Default is exec (no prefix)
        run_exec_filter(spec, args, input);
    }
}

/// Run an exec filter: spawn subprocess, feed input to stdin, write stdout to HtmlOutput.
fn run_exec_filter(cmd: &str, args: &[&str], input: &[u8]) {
    let child = Command::new(cmd)
        .args(args)
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::inherit())
        .spawn();

    let mut child = match child {
        Ok(c) => c,
        Err(_) => {
            // Filter failed to start, output input unchanged
            html_raw(input);
            return;
        }
    };

    if let Some(mut stdin) = child.stdin.take() {
        let _ = stdin.write_all(input);
        // stdin is dropped here, closing the pipe
    }

    match child.wait_with_output() {
        Ok(output) => html_raw(&output.stdout),
        Err(_) => html_raw(input),
    }
}

/// Run a Lua filter using mlua.
fn run_lua_filter(script: &str, args: &[&str], input: &[u8]) {
    match run_lua_filter_inner(script, args, input) {
        Ok(()) => {}
        Err(_) => {
            // Lua filter failed, output input unchanged
            html_raw(input);
        }
    }
}

fn run_lua_filter_inner(script: &str, args: &[&str], input: &[u8]) -> Result<(), Box<dyn std::error::Error>> {
    let lua = mlua::Lua::new_with(
        mlua::StdLib::STRING | mlua::StdLib::TABLE | mlua::StdLib::MATH | mlua::StdLib::UTF8,
        mlua::LuaOptions::default(),
    )?;

    // Register html functions that write to HtmlOutput
    let html_fn = lua.create_function(|_, s: String| {
        crate::html::html(&s);
        Ok(())
    })?;
    lua.globals().set("html", html_fn)?;

    let html_txt_fn = lua.create_function(|_, s: String| {
        crate::html::html_txt(&s);
        Ok(())
    })?;
    lua.globals().set("html_txt", html_txt_fn)?;

    let html_attr_fn = lua.create_function(|_, s: String| {
        crate::html::html_attr(&s);
        Ok(())
    })?;
    lua.globals().set("html_attr", html_attr_fn)?;

    let html_url_path_fn = lua.create_function(|_, s: String| {
        crate::html::html_url_path(&s);
        Ok(())
    })?;
    lua.globals().set("html_url_path", html_url_path_fn)?;

    let html_url_arg_fn = lua.create_function(|_, s: String| {
        crate::html::html_url_arg(&s);
        Ok(())
    })?;
    lua.globals().set("html_url_arg", html_url_arg_fn)?;

    let html_include_fn = lua.create_function(|_, s: String| {
        let _ = crate::html::html_include(&s);
        Ok(())
    })?;
    lua.globals().set("html_include", html_include_fn)?;

    // Load and execute the script
    lua.load(std::fs::read_to_string(script)?).exec()?;

    // Call filter_open with args
    let filter_open: mlua::Function = lua.globals().get("filter_open")?;
    let lua_args: Vec<mlua::Value> = args.iter()
        .map(|a| mlua::Value::String(lua.create_string(a).unwrap()))
        .collect();
    filter_open.call::<()>(mlua::MultiValue::from_iter(lua_args))?;

    // Call filter_write with input
    let input_str = String::from_utf8_lossy(input);
    let filter_write: mlua::Function = lua.globals().get("filter_write")?;
    filter_write.call::<()>(input_str.as_ref())?;

    // Call filter_close
    let filter_close: mlua::Function = lua.globals().get("filter_close")?;
    filter_close.call::<()>(())?;

    Ok(())
}

/// Helper: capture HtmlOutput, run content generator, then pass through filter.
/// If no filter is set, content goes directly to output.
pub fn with_filter(filter_spec: Option<&str>, args: &[&str], content_fn: impl FnOnce()) {
    if let Some(spec) = filter_spec {
        HtmlOutput::start_capture();
        content_fn();
        let captured = HtmlOutput::stop_capture();
        run_filter(spec, args, &captured);
    } else {
        content_fn();
    }
}
