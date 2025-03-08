use bluer::rfcomm::Stream;
use clap::{Parser, Subcommand, ValueEnum};
use tokio::io::{AsyncReadExt, AsyncWriteExt};

#[derive(Parser)]
#[command(version, about)]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    Anc {mode: Option<AncMode>}
}

#[derive(Debug, Clone, Copy, ValueEnum)]
enum AncMode {
    High,
    Mid,
    Low,
    Adaptive,
    Off,
    Transparency,
} impl TryFrom<u8> for AncMode {
    type Error = bluer::Error;
    fn try_from(value: u8) -> Result<Self, Self::Error> {
        match value {
            1 => Ok(Self::High),
            2 => Ok(Self::Mid),
            3 => Ok(Self::Low),
            4 => Ok(Self::Adaptive),
            5 => Ok(Self::Off),
            7 => Ok(Self::Transparency),
            _ => Err(Self::Error {
                kind: bluer::ErrorKind::Failed,
                message: "could not decode anc response from bt device".to_owned(),
            }),
        }
    }
}

async fn send_anc_command(anc_mode: AncMode, socket: &mut Stream) -> Result<(), std::io::Error> {
    let data: [u8; 13] = match anc_mode {
        AncMode::High => [85, 96, 1, 15, 240, 3, 0, 237, 1, 1, 0, 205, 71],
        AncMode::Mid => [85, 96, 1, 15, 240, 3, 0, 86, 1, 2, 0, 233, 83],
        AncMode::Low => [85, 96, 1, 15, 240, 3, 0, 103, 1, 3, 0, 230, 63],
        AncMode::Adaptive => [85, 96, 1, 15, 240, 3, 0, 118, 1, 4, 0, 225, 51],
        AncMode::Off => [85, 96, 1, 15, 240, 3, 0, 4, 1, 5, 0, 251, 219],
        AncMode::Transparency => [85, 96, 1, 15, 240, 3, 0, 97, 1, 7, 0, 228, 119],
    };

    socket.write_all(&data).await?;

    Ok(())
}

#[tokio::main(flavor="current_thread")]
async fn main() -> bluer::Result<()>{
    let cli = Cli::parse();

    let session = bluer::Session::new().await?;

    let adapter = session.default_adapter().await?;
    adapter.set_powered(true).await?;
    

    let device_addresses = adapter.device_addresses().await?;
    let ear_address = device_addresses
        .iter()
        .find(|&addr| matches!(addr, bluer::Address([0x2C, 0xBE, 0xEB, _, _, _]))).unwrap();


    let mut socket = Stream::connect(bluer::rfcomm::SocketAddr {
        addr: *ear_address,
        channel: 15}).await?;

    match cli.command {
        Commands::Anc {mode: None} => {
            socket.write_all(&[0x55, 0x60, 0x01, 0x1e, 0xc0, 0x01, 0x00, 0x0c, 0x03, 0x98, 0x19]).await?;
            let mut response_buffer: [u8; 16] = [0; 16];
            socket.read_exact(&mut response_buffer).await?;
            let anc_mode: AncMode = response_buffer[9].try_into()?;
            println!("{:?}", anc_mode);
        }
        Commands::Anc { mode: Some(mode) } => {
            send_anc_command(mode, &mut socket).await?;
        }
    }
    Ok(())
}