#![allow(unused)]
use crossbeam::channel::Sender;
use lsp_types::notification::{
    DidChangeTextDocument, DidOpenTextDocument, Notification, PublishDiagnostics,
};
use lsp_types::request::Request;
use lsp_types::{ClientCapabilities, CompletionParams};
use std::task::Wake;
use std::{any, error::Error};

use crate::dispatch::handlers::request::handle_completions;

use anyhow::Result;
use lsp_server::{
    Connection, ErrorCode as LspErrorCode, Message, Request as ServerRequest, RequestId, Response,
};

use lsp_types::{
    CompletionItem,
    CompletionItemKind,
    // capability helpers
    CompletionOptions,
    CompletionResponse,
    Diagnostic,
    DiagnosticSeverity,
    DidChangeTextDocumentParams,
    DidOpenTextDocumentParams,
    DocumentFormattingParams,
    Hover,
    HoverContents,
    HoverProviderCapability,
    // core
    InitializeParams,
    MarkedString,
    OneOf,
    Position,
    PublishDiagnosticsParams,
    Range,
    ServerCapabilities,
    TextDocumentSyncCapability,
    TextDocumentSyncKind,
    TextEdit,
    Uri,
    request::{Completion, GotoDefinition},
};
use rustc_hash::FxHashMap;

pub struct Dispatcher<'a> {
    sender: &'a Sender<lsp_server::Message>,
    client_capabilities: &'a serde_json::Value,
    version_check: u8,
}

pub fn main_loop(
    connection: Connection,
    client_capabilities: serde_json::Value,
) -> std::result::Result<(), Box<dyn Error + Sync + Send>> {
    let dispatcher = Dispatcher::new(&connection.sender, &client_capabilities);

    let mut docs: FxHashMap<Uri, String> = FxHashMap::default();

    for msg in &connection.receiver {
        match msg {
            Message::Request(req) => {
                if connection.handle_shutdown(&req)? {
                    break;
                }

                if let Err(err) = Dispatcher::handle_request(&connection, &req.clone(), &mut docs) {
                    println!("[lsp] failed to handle request {} failed: {err}", err);
                }
            }
            Message::Notification(note) => {
                if let Err(err) = Dispatcher::handle_notification(&connection, &note, &mut docs) {
                    println!("[lsp] notification {} failed: {err}", note.method);
                }
            }
            Message::Response(resp) => println!("[lsp] response: {resp:?}"),
        }
    }
    Ok(())
}

fn full_range(text: &str) -> Range {
    let last_line_idx = text.lines().count().saturating_sub(1) as u32;
    let last_col = text.lines().last().map_or(0, |l| l.chars().count()) as u32;
    Range::new(Position::new(0, 0), Position::new(last_line_idx, last_col))
}

fn send_ok<T: serde::Serialize>(conn: &Connection, id: RequestId, result: &T) -> Result<()> {
    let resp = Response {
        id,
        result: Some(serde_json::to_value(result)?),
        error: None,
    };

    conn.sender.send(Message::Response(resp))?;
    Ok(())
}

fn send_err(
    connection: &Connection,
    id: RequestId,
    code: lsp_server::ErrorCode,
    msg: &str,
) -> Result<()> {
    let resp = Response {
        id,
        result: None,
        error: Some(lsp_server::ResponseError {
            code: code as i32,
            message: msg.into(),
            data: None,
        }),
    };

    connection.sender.send(Message::Response(resp))?;
    Ok(())
}

fn publish_dummy_diag(conn: &Connection, uri: &Uri) -> Result<()> {
    let diag = Diagnostic {
        range: Range::new(Position::new(0, 0), Position::new(0, 1)),
        severity: Some(DiagnosticSeverity::INFORMATION),
        code: None,
        code_description: None,
        source: Some("minimal_lsp".into()),
        message: "dummy diagnostic".into(),
        related_information: None,
        tags: None,
        data: None,
    };
    let params = PublishDiagnosticsParams {
        uri: uri.clone(),
        diagnostics: vec![diag],
        version: None,
    };
    conn.sender
        .send(Message::Notification(lsp_server::Notification::new(
            PublishDiagnostics::METHOD.to_owned(),
            params,
        )))?;
    Ok(())
}

impl<'a> Dispatcher<'a> {
    fn new(
        sender: &'a Sender<lsp_server::Message>,
        client_capabilities: &'a serde_json::Value,
    ) -> Self {
        Self {
            sender,
            client_capabilities,
            version_check: 0,
        }
    }

    pub fn handle_request(
        conn: &Connection,
        req: &ServerRequest,
        _docs: &mut FxHashMap<Uri, String>,
    ) -> Result<bool> {
        match req.method.as_str() {
            GotoDefinition::METHOD => {
                send_ok(
                    conn,
                    req.id.clone(),
                    &lsp_types::GotoDefinitionResponse::Array(Vec::new()),
                )?;
            }
            Completion::METHOD => {
                let params: CompletionParams = serde_json::from_value(req.params.clone())?;
                let uri = &params.text_document_position.text_document.uri;
                let pos = &params.text_document_position.position;
                let doc = _docs.get(uri).map(String::as_str).unwrap_or("");
                let item = handle_completions(doc, pos);
                send_ok(conn, req.id.clone(), &CompletionResponse::Array(item))?;
            }
            _ => send_err(
                conn,
                req.id.clone(),
                lsp_server::ErrorCode::MethodNotFound,
                "unhandled method",
            )?,
        }
        Ok(false)
    }
    pub fn handle_notification(
        conn: &Connection,
        note: &lsp_server::Notification,
        docs: &mut FxHashMap<Uri, String>,
    ) -> Result<()> {
        match note.method.as_str() {
            DidOpenTextDocument::METHOD => {
                let p: DidOpenTextDocumentParams = serde_json::from_value(note.params.clone())?;
                let uri = p.text_document.uri;
                docs.insert(uri.clone(), p.text_document.text);
                publish_dummy_diag(conn, &uri)?;
            }
            DidChangeTextDocument::METHOD => {
                let p: DidChangeTextDocumentParams = serde_json::from_value(note.params.clone())?;
                if let Some(change) = p.content_changes.into_iter().next() {
                    let uri = p.text_document.uri;
                    docs.insert(uri.clone(), change.text);
                    publish_dummy_diag(conn, &uri)?;
                }
            }
            _ => {}
        }
        Ok(())
    }
}
