use std::{
    env,
    net::{IpAddr, SocketAddr},
};

use http_body_util::Empty;
use hyper::{body::Bytes, Request, StatusCode, Uri};
use hyper_util::rt::TokioIo;
use tlsn_common::config::{ProtocolConfig, ProtocolConfigValidator};
use tlsn_core::transcript::Idx;
use tlsn_examples::get_crypto_provider_with_server_fixture;
use tlsn_prover::{Prover, ProverConfig};
use tlsn_server_fixture_certs::SERVER_DOMAIN;
use tlsn_verifier::{SessionInfo, Verifier, VerifierConfig};
use tokio::{io::{AsyncRead, AsyncWrite}, net::TcpStream};
use tokio_util::compat::{FuturesAsyncReadCompatExt, TokioAsyncReadCompatExt};
use tracing::instrument;

const DEFAULT_FIXTURE_PORT: u16 = 3000;
const SECRET: &str = "TLSNotary's private key 🤡";
const MAX_SENT_DATA: usize = 1 << 12;
const MAX_RECV_DATA: usize = 1 << 14;

#[instrument]
pub async fn run_tlsn_interactive(
) -> Result<(), Box<dyn std::error::Error>> {
    // Pobranie ustawień serwera
    let server_host = env::var("SERVER_HOST").unwrap_or_else(|_| "127.0.0.1".into());
    let server_port: u16 = env::var("SERVER_PORT")
        .unwrap_or_else(|_| DEFAULT_FIXTURE_PORT.to_string())
        .parse()?;
    let server_ip: IpAddr = server_host.parse()?;
    let server_addr = SocketAddr::from((server_ip, server_port));
    let uri_str = format!("https://{}:{}/formats/html", SERVER_DOMAIN, server_port);
    let uri: Uri = uri_str.parse()?;
    // Tworzymy połączenie duplex między Proverem a Verifierem
    // let (prover_socket, verifier_socket) = tokio::io::duplex(1 << 23);

    let verifier_addr = "127.0.0.1:5555".parse::<SocketAddr>().unwrap();
    let prover_socket = TcpStream::connect(verifier_addr).await.unwrap();
    // Uruchamiamy Provera i Verifiera równolegle

    // let verifier_fut = run_verifier(verifier_socket);
    let prover_result = run_prover(prover_socket, &server_addr, &uri).await;

    // let (prover_result, verifier_result) = tokio::join!(prover_fut, verifier_fut);

    // Obsłuż błędy: jeśli prover nie zakończył się powodzeniem, zwróć błąd
    prover_result?;

    // let (sent, received, session_info) = verifier_result?;

    // Ok((sent, received, session_info))
    Ok(())
}

#[instrument(skip(verifier_socket))]
async fn run_prover<T>(
    verifier_socket: T,
    server_addr: &SocketAddr,
    uri: &Uri,
) -> Result<(), Box<dyn std::error::Error>>
where
    T: AsyncWrite + AsyncRead + Send + Unpin + 'static,
{

    let server_domain = uri.authority().unwrap().host();

    let prover = Prover::new(
        ProverConfig::builder()
            .server_name(server_domain)
            .protocol_config(
                ProtocolConfig::builder()
                    .max_sent_data(MAX_SENT_DATA)
                    .max_recv_data(MAX_RECV_DATA)
                    .build()?,
            )
            .crypto_provider(get_crypto_provider_with_server_fixture())
            .build()?,
    )
    .setup(verifier_socket.compat())
    .await?;

    let tls_client_socket = tokio::net::TcpStream::connect(server_addr).await?;

    let (mpc_tls_conn, prover_future) = prover.connect(tls_client_socket.compat()).await?;

    let mpc_tls_conn = TokioIo::new(mpc_tls_conn.compat());

    let prover_task = tokio::spawn(prover_future);

    let (mut request_sender, connection) =
        hyper::client::conn::http1::handshake(mpc_tls_conn).await?;

    tokio::spawn(connection);

    let request = Request::builder()
        .uri(uri.clone())
        .header("Host", server_domain)
        .header("Connection", "close")
        .header("Secret", SECRET)
        .method("GET")
        .body(Empty::<Bytes>::new())?;

    let response = request_sender.send_request(request).await?;

    assert_eq!(response.status(), StatusCode::OK);
    let mut prover = prover_task.await??.start_prove();
    let idx_sent = Idx::new([0..prover.transcript().sent().len()]);
    let idx_recv = Idx::new([0..prover.transcript().received().len()]);
    prover.prove_transcript(idx_sent, idx_recv).await?;
    prover.finalize().await?;
    Ok(())
}

// #[instrument(skip(socket))]
// async fn run_verifier<T>(
//     socket: T,
// ) -> Result<(Vec<u8>, Vec<u8>, SessionInfo), Box<dyn std::error::Error>>
// where
//     T: AsyncWrite + AsyncRead + Send + Sync + Unpin + 'static,
// {
//     let config_validator = ProtocolConfigValidator::builder()
//         .max_sent_data(MAX_SENT_DATA)
//         .max_recv_data(MAX_RECV_DATA)
//         .build()?;
//     let verifier_config = VerifierConfig::builder()
//         .protocol_config_validator(config_validator)
//         .crypto_provider(get_crypto_provider_with_server_fixture())
//         .build()?;
//     let verifier = Verifier::new(verifier_config);
//     let (mut partial_transcript, session_info) = verifier.verify(socket.compat()).await?;
//     partial_transcript.set_unauthed(0);
//     let sent = partial_transcript.sent_unsafe().to_vec();
//     let received = partial_transcript.received_unsafe().to_vec();
//     Ok((sent, received, session_info))
// }

// Funkcje pomocnicze

fn revealed_ranges_received(prover: &mut tlsn_prover::Prover<tlsn_prover::state::Prove>) -> Idx {
    let recv_transcript = prover.transcript().received();
    let start = String::from_utf8_lossy(&recv_transcript)
        .find("Dick")
        .expect("Substring 'Dick' not found");
    let end = start + "Dick".len();
    Idx::new([0..start, end..recv_transcript.len()])
}

fn revealed_ranges_sent(prover: &mut tlsn_prover::Prover<tlsn_prover::state::Prove>) -> Idx {
    let sent_transcript = prover.transcript().sent();
    let secret_start = String::from_utf8_lossy(&sent_transcript)
        .find(SECRET)
        .expect("Secret not found");
    Idx::new([
        0..secret_start,
        secret_start + SECRET.len()..sent_transcript.len(),
    ])
}

fn bytes_to_redacted_string(bytes: &[u8]) -> String {
    String::from_utf8_lossy(bytes).replace('\0', "🙈")
}