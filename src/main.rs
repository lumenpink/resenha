use clap::{Parser, Subcommand};
use resenha::{process_url, server};
use std::net::SocketAddr;
use std::path::PathBuf;

#[derive(Parser, Debug)]
#[command(
    name = "resenha",
    about = "Subsistema de Extração, Sanitização e Destilação Tipográfica na Borda",
    version
)]
struct Cli {
    #[command(subcommand)]
    command: Option<Commands>,

    /// URL da página a ser extraída (modo direto)
    #[arg(value_name = "URL")]
    url: Option<String>,

    /// Arquivo de saída (opcional, padrão: stdout)
    #[arg(short, long, value_name = "FILE")]
    output: Option<PathBuf>,

    /// Formato de saída (md ou json)
    #[arg(short, long, default_value = "md")]
    format: String,
}

#[derive(Subcommand, Debug)]
enum Commands {
    /// Inicia o microserviço HTTP (Axum)
    Serve {
        /// Porta do servidor HTTP
        #[arg(short, long, env = "RESENHA_PORT", default_value_t = 3030)]
        port: u16,

        /// Endereço de bind
        #[arg(short, long, env = "RESENHA_BIND", default_value = "0.0.0.0")]
        bind: String,
    },
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    tracing_subscriber::fmt()
        .with_env_filter(tracing_subscriber::EnvFilter::from_default_env())
        .init();

    let cli = Cli::parse();

    match cli.command {
        Some(Commands::Serve { port, bind }) => {
            let addr: SocketAddr = format!("{bind}:{port}").parse()?;
            let app = server::create_app();

            println!("✂ resenha escutando em http://{addr}");
            let listener = tokio::net::TcpListener::bind(addr).await?;
            axum::serve(listener, app).await?;
        }
        None => {
            let target_url = match cli.url {
                Some(u) => u,
                None => {
                    eprintln!(
                        "Erro: Nenhuma URL fornecida. Use 'resenha <URL>' ou 'resenha serve'."
                    );
                    std::process::exit(1);
                }
            };

            let markdown = process_url(&target_url).await?;

            if let Some(output_path) = cli.output {
                tokio::fs::write(&output_path, &markdown).await?;
                eprintln!("✓ Artigo destilado salvo em: {}", output_path.display());
            } else {
                print!("{markdown}");
            }
        }
    }

    Ok(())
}
