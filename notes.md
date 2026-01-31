 
 ## URIs

```text
 foo://example.com:8042/over/there?name=ferret#nose
         \_/   \______________/\_________/ \_________/ \__/
          |           |            |            |        |
       scheme     authority       path        query   fragment
          |   _____________________|__
         / \ /                        \
         urn:example:animal:ferret:nose
 ```
 
```text
! Minimal Language‑Server‑Protocol example: **`minimal_lsp.rs`**
! =============================================================
!
! | ↔ / ← | LSP method | What the implementation does |
! |-------|------------|------------------------------|
! | ↔ | `initialize` / `initialized` | capability handshake |
! | ← | `textDocument/publishDiagnostics` | pushes a dummy info diagnostic whenever the buffer changes |
! | ← | `textDocument/definition` | echoes an empty location array so the jump works |
! | ← | `textDocument/completion` | offers one hard‑coded item `HelloFromLSP` |
! | ← | `textDocument/hover` | shows *Hello from minimal_lsp* markdown |
! | ← | `textDocument/formatting` | pipes the doc through **rustfmt** and returns a full‑file edit |
!
! ### Quick start
! ```bash
! cd rust-analyzer/lib/lsp-server
! cargo run --example minimal_lsp
! ```
!
! ### Minimal manual session (all nine packets)
! ```no_run
! # 1. initialize - server replies with capabilities
! Content-Length: 85

! {"jsonrpc":"2.0","id":1,"method":"initialize","params":{"capabilities":{}}}
!
! # 2. initialized - no response expected
! Content-Length: 59

! {"jsonrpc":"2.0","method":"initialized","params":{}}
!
! # 3. didOpen - provide initial buffer text
! Content-Length: 173

! {"jsonrpc":"2.0","method":"textDocument/didOpen","params":{"textDocument":{"uri":"file:///tmp/foo.rs","languageId":"rust","version":1,"text":"fn  main( ){println!(\"hi\") }"}}}
!
! # 4. completion - expect HelloFromLSP
! Content-Length: 139

! {"jsonrpc":"2.0","id":2,"method":"textDocument/completion","params":{"textDocument":{"uri":"file:///tmp/foo.rs"},"position":{"line":0,"character":0}}}
!
! # 5. hover - expect markdown greeting
! Content-Length: 135

! {"jsonrpc":"2.0","id":3,"method":"textDocument/hover","params":{"textDocument":{"uri":"file:///tmp/foo.rs"},"position":{"line":0,"character":0}}}
!
! # 6. goto-definition - dummy empty array
! Content-Length: 139

! {"jsonrpc":"2.0","id":4,"method":"textDocument/definition","params":{"textDocument":{"uri":"file:///tmp/foo.rs"},"position":{"line":0,"character":0}}}
!
! # 7. formatting - rustfmt full document
! Content-Length: 157

! {"jsonrpc":"2.0","id":5,"method":"textDocument/formatting","params":{"textDocument":{"uri":"file:///tmp/foo.rs"},"options":{"tabSize":4,"insertSpaces":true}}}
!
! # 8. shutdown request - server acks and prepares to exit
! Content-Length: 67

! {"jsonrpc":"2.0","id":6,"method":"shutdown","params":null}
!
! # 9. exit notification - terminates the server
! Content-Length: 54

! {"jsonrpc":"2.0","method":"exit","params":null}
! ```
!
======================================================================
```
