use tlsn_common::config::ProtocolConfigValidator;
use tlsn_examples::get_crypto_provider_with_server_fixture;
use tlsn_verifier::{Verifier, VerifierConfig};
use tokio::io::AsyncWriteExt;
use tokio::net::TcpListener;
use tokio_util::compat::FuturesAsyncReadCompatExt;
use tokio_util::compat::TokioAsyncReadCompatExt;
use tlsn_server_fixture_certs::SERVER_DOMAIN;


#[tokio::main]
async fn main() {
    let listener = TcpListener::bind("127.0.0.1:5555").await.unwrap();
    println!("Verifier nasłuchuje na 127.0.0.1:5555");

    let (socket, _) = listener.accept().await.unwrap();
    println!("Połączono z Proverem");

    verifier(socket).await;
}

async fn verifier(
    socket: impl tokio::io::AsyncRead + tokio::io::AsyncWrite + Unpin + Send + 'static,
) {
    let config_validator = ProtocolConfigValidator::builder()
        .max_sent_data(1 << 12)
        .max_recv_data(1 << 14)
        .build()
        .unwrap();

    let verifier_config = VerifierConfig::builder()
        .protocol_config_validator(config_validator)
        .crypto_provider(get_crypto_provider_with_server_fixture())
        .build()
        .unwrap();

    let verifier = Verifier::new(verifier_config);

    let compat_socket = socket.compat();


    println!("🔹 Verifier zaczyna weryfikację...");
    match verifier.verify(compat_socket).await {
        Ok((mut partial_transcript, _)) => {
            partial_transcript.set_unauthed(0);
            let received = partial_transcript.received_unsafe().to_vec();
            let response =
                String::from_utf8(received.clone()).expect("Verifier expected received data");
            response
                .find("Herman Melville")
                .unwrap_or_else(|| panic!("Expected valid data from {}", SERVER_DOMAIN));
            println!("response : {}", response);
            println!("✅ Weryfikacja zakończona!");
        }
        Err(e) => {
            println!("❌ Błąd weryfikacji: {:?}", e);
        }
    }
}
