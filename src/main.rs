#![allow(unused_imports)]
use lsp_types::InitializeResult;
use lsp_types::ServerInfo;
use lsp_types::notification::DidChangeTextDocument;
use lsp_types::notification::DidOpenTextDocument;
use lsp_types::notification::Notification;
use lsp_types::notification::PublishDiagnostics;
use lsp_types::request::Request;
use std::{any, error::Error};

mod dispatch;
mod dispatcher;

use anyhow::Result;
use lsp_server::{
    Connection, ErrorCode as LspErrorCode, Message, Request as ServerRequest, RequestId, Response,
};

use lsp_types::{
    // capability helpers
    CompletionOptions,
    CompletionResponse,
    // core
    InitializeParams,
    OneOf,
    Position,
    PublishDiagnosticsParams,
    ServerCapabilities,
    TextDocumentSyncCapability,
    TextDocumentSyncKind,
};

use crate::dispatcher::Dispatcher;
use crate::dispatcher::main_loop;

#[allow(clippy::print_stderr)]
fn main() -> std::result::Result<(), Box<dyn Error + Sync + Send>> {
    let (connection, io_thread) = Connection::stdio();
    // return Err(anyhow::anyhow!("not implemented"));

    // setting up capabilities
    let caps = ServerCapabilities {
        text_document_sync: Some(TextDocumentSyncCapability::Kind(TextDocumentSyncKind::FULL)),
        completion_provider: Some(CompletionOptions {
            resolve_provider: Some(false),
            // insert_text:
            trigger_characters: Some(vec![".".to_string()]),
            ..Default::default()
        }),
        definition_provider: Some(OneOf::Left(true)),
        hover_provider: Some(lsp_types::HoverProviderCapability::Simple(true)),
        ..Default::default()
    };

    // lets encode this jsonrpc shit
    // let init_value = serde_json::json!({
    //     "capabilities": caps,
    //     "offsetEncoding": ["utf-8"] //supported by the LSP protocol
    // });
    //
    // let ini_params = connection.initialize(init_value)?;

    let init_result = InitializeResult {
        capabilities: caps,
        server_info: Some(ServerInfo {
            name: "lc3_lsp".into(),
            version: Some("0.1.0".into()),
        }),
    };

    let ini_params = connection.initialize(serde_json::to_value(init_result)?)?;
    let res = main_loop(connection, ini_params);
    io_thread.join()?;
    eprintln!("shutting down server");
    res
}
