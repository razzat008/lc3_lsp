//! Minimal Language‑Server‑Protocol example: **`minimal_lsp.rs`**
//! =============================================================
//!
//! | ↔ / ← | LSP method | What the implementation does |
//! |-------|------------|------------------------------|
//! | ↔ | `initialize` / `initialized` | capability handshake |
//! | ← | `textDocument/publishDiagnostics` | pushes a dummy info diagnostic whenever the buffer changes |
//! | ← | `textDocument/definition` | echoes an empty location array so the jump works |
//! | ← | `textDocument/completion` | offers one hard‑coded item `HelloFromLSP` |
//! | ← | `textDocument/hover` | shows *Hello from minimal_lsp* markdown |
//! | ← | `textDocument/formatting` | pipes the doc through **rustfmt** and returns a full‑file edit |
//!
//! ### Quick start
//! ```bash
//! cd rust-analyzer/lib/lsp-server
//! cargo run --example minimal_lsp
//! ```
//!
//! ### Minimal manual session (all nine packets)
//! ```no_run
//! # 1. initialize - server replies with capabilities
//! Content-Length: 85

//! {"jsonrpc":"2.0","id":1,"method":"initialize","params":{"capabilities":{}}}
//!
//! # 2. initialized - no response expected
//! Content-Length: 59

//! {"jsonrpc":"2.0","method":"initialized","params":{}}
//!
//! # 3. didOpen - provide initial buffer text
//! Content-Length: 173

//! {"jsonrpc":"2.0","method":"textDocument/didOpen","params":{"textDocument":{"uri":"file:///tmp/foo.rs","languageId":"rust","version":1,"text":"fn  main( ){println!(\"hi\") }"}}}
//!
//! # 4. completion - expect HelloFromLSP
//! Content-Length: 139

//! {"jsonrpc":"2.0","id":2,"method":"textDocument/completion","params":{"textDocument":{"uri":"file:///tmp/foo.rs"},"position":{"line":0,"character":0}}}
//!
//! # 5. hover - expect markdown greeting
//! Content-Length: 135

//! {"jsonrpc":"2.0","id":3,"method":"textDocument/hover","params":{"textDocument":{"uri":"file:///tmp/foo.rs"},"position":{"line":0,"character":0}}}
//!
//! # 6. goto-definition - dummy empty array
//! Content-Length: 139

//! {"jsonrpc":"2.0","id":4,"method":"textDocument/definition","params":{"textDocument":{"uri":"file:///tmp/foo.rs"},"position":{"line":0,"character":0}}}
//!
//! # 7. formatting - rustfmt full document
//! Content-Length: 157

//! {"jsonrpc":"2.0","id":5,"method":"textDocument/formatting","params":{"textDocument":{"uri":"file:///tmp/foo.rs"},"options":{"tabSize":4,"insertSpaces":true}}}
//!
//! # 8. shutdown request - server acks and prepares to exit
//! Content-Length: 67

//! {"jsonrpc":"2.0","id":6,"method":"shutdown","params":null}
//!
//! # 9. exit notification - terminates the server
//! Content-Length: 54

//! {"jsonrpc":"2.0","method":"exit","params":null}
//! ```
//!
//======================================================================

#![allow(unused)]
pub mod parser;

use lsp_types::notification::DidChangeTextDocument;
use lsp_types::notification::DidOpenTextDocument;
use lsp_types::notification::Notification;
use lsp_types::notification::PublishDiagnostics;
use lsp_types::request::Request;
use std::{any, error::Error};

use anyhow::Result;
use lsp_server::{Connection, Message, Request as ServerRequest, RequestId, Response};

use lsp_server::ErrorCode as LspErrorCode;
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

#[allow(clippy::print_stderr)]
fn main() -> std::result::Result<(), Box<dyn Error + Sync + Send>> {
    let (connection, io_thread) = Connection::stdio();
    // return Err(anyhow::anyhow!("not implemented"));

    let caps = ServerCapabilities {
        text_document_sync: Some(TextDocumentSyncCapability::Kind(TextDocumentSyncKind::FULL)),
        completion_provider: Some(CompletionOptions::default()),
        definition_provider: Some(OneOf::Left(true)),
        hover_provider: Some(HoverProviderCapability::Simple(true)),
        ..Default::default()
    };

    // encoding capabilities for the protocol
    let init_value = serde_json::json!({
        "capabilities": caps,
        "offsetEncoding": ["utf-8"] //supported by the LSP protocol
    });

    let ini_params = connection.initialize(init_value)?;
    main_loop(connection, ini_params)?;
    io_thread.join()?;
    Ok(())
}

fn main_loop(
    connection: Connection,
    params: serde_json::Value,
) -> std::result::Result<(), Box<dyn Error + Sync + Send>> {
    let _init_params: InitializeParams = serde_json::from_value(params)?;

    let mut docs: FxHashMap<Uri, String> = FxHashMap::default();

    for msg in &connection.receiver {
        match msg {
            Message::Request(req) => {
                if connection.handle_shutdown(&req)? {
                    break;
                }
                if let Err(err) = handle_request(&connection, &req, &mut docs) {
                    println!("fucked handle_request")
                }
            }
            Message::Notification(note) => {
                if let Err(err) = handle_notification(&connection, &note, &mut docs) {
                    println!("[lsp] notification {} failed: {err}", note.method);
                }
            }
            Message::Response(resp) => println!("[lsp] response: {resp:?}"),
        }
    }
    Ok(())
}
fn handle_request(
    conn: &Connection,
    req: &ServerRequest,
    docs: &mut FxHashMap<Uri, String>,
) -> Result<()> {
    match req.method.as_str() {
        GotoDefinition::METHOD => {
            send_ok(
                conn,
                req.id.clone(),
                &lsp_types::GotoDefinitionResponse::Array(Vec::new()),
            )?;
        }
        Completion::METHOD => {
            let item = CompletionItem {
                label: "HelloFromLSP".into(),
                kind: Some(CompletionItemKind::FUNCTION),
                detail: Some("dummy completion".into()),
                ..Default::default()
            };
            send_ok(conn, req.id.clone(), &CompletionResponse::Array(vec![item]))?;
        }
        _ => send_err(
            conn,
            req.id.clone(),
            lsp_server::ErrorCode::MethodNotFound,
            "unhandled method",
        )?,
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

fn handle_notification(
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
