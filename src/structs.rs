// use std::sync::Arc;

use std::fmt;

use tokio_rustls::TlsAcceptor;

#[derive(Clone)]
pub struct SharedData{
    pub mime: std::collections::HashMap<&'static str,&'static str>,
    pub serve_dir: String,
    pub tls_acceptor: Option<TlsAcceptor>,
}

impl fmt::Debug for SharedData{
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("SharedData")
            .field("mime",&self.mime.len())
            .field("serve_dir",&self.serve_dir)
            .field("tls_acceptor", if self.tls_acceptor.is_some(){&"Some(TlsAcceptor)"}else{&"None"})
            .finish()
    }
}