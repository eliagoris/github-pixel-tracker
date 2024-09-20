use std::io::{ Write, Read };
use std::net::{ TcpListener, TcpStream };
use std::fs::OpenOptions;
use std::sync::{ Arc, Mutex };
use std::thread;
use std::time::SystemTime;

// Função para servir o pixel invisível (1x1 GIF transparente)
fn pixel_data() -> &'static [u8] {
    &[
        0x47, 0x49, 0x46, 0x38, 0x39, 0x61, 0x01, 0x00, 0x01, 0x00, 0x80, 0xff, 0x00, 0xff, 0xff, 0xff,
        0x00, 0x00, 0x00, 0x21, 0xf9, 0x04, 0x01, 0x00, 0x00, 0x00, 0x00, 0x2c, 0x00, 0x00, 0x00, 0x00,
        0x01, 0x00, 0x01, 0x00, 0x00, 0x02, 0x02, 0x44, 0x01, 0x00, 0x3b,
    ]
}

// Função para lidar com a conexão de um cliente
fn handle_client(stream: TcpStream, log_file: Arc<Mutex<std::fs::File>>) {
    let mut stream = stream;
    let mut buffer = [0; 512];

    // Ler a requisição HTTP
    stream.read(&mut buffer).unwrap();

    // Registrar o acesso
    let log_entry = format!(
        "Access from IP: {}, Time: {:?}\n",
        stream.peer_addr().unwrap(),
        SystemTime::now()
    );
    let mut file = log_file.lock().unwrap();
    file.write_all(log_entry.as_bytes()).unwrap();

    // Criar a resposta HTTP para servir o pixel invisível
    let response = format!(
        "HTTP/1.1 200 OK\r\nContent-Type: image/gif\r\nContent-Length: {}\r\n\r\n",
        pixel_data().len()
    );

    // Enviar o cabeçalho e o conteúdo do pixel
    stream.write_all(response.as_bytes()).unwrap();
    stream.write_all(pixel_data()).unwrap();
    stream.flush().unwrap();
}

fn main() {
    // Iniciar o servidor TCP na porta 8080
    let listener = TcpListener::bind("0.0.0.0:8080").unwrap();
    println!("Server running on port 8080");

    // Abrir o arquivo de log e protegê-lo com um Mutex para acesso seguro entre threads
    let log_file = Arc::new(
        Mutex::new(OpenOptions::new().create(true).append(true).open("access_log.txt").unwrap())
    );

    // Aceitar conexões e lidar com elas em threads separadas
    for stream in listener.incoming() {
        let stream = stream.unwrap();
        let log_file = log_file.clone();

        thread::spawn(move || {
            handle_client(stream, log_file);
        });
    }
}
