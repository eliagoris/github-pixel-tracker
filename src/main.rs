use std::io::{ Read, Write };
use std::net::{ TcpListener, TcpStream };
use std::fs::OpenOptions;
use std::sync::{ Arc, Mutex };
use std::thread;
use std::time::SystemTime;

fn pixel_data() -> &'static [u8] {
    &[
        0x47, 0x49, 0x46, 0x38, 0x39, 0x61, 0x01, 0x00, 0x01, 0x00, 0x80, 0xff, 0x00, 0xff, 0xff, 0xff,
        0x00, 0x00, 0x00, 0x21, 0xf9, 0x04, 0x01, 0x00, 0x00, 0x00, 0x00, 0x2c, 0x00, 0x00, 0x00, 0x00,
        0x01, 0x00, 0x01, 0x00, 0x00, 0x02, 0x02, 0x44, 0x01, 0x00, 0x3b,
    ]
}

fn handle_client(mut stream: TcpStream, log_file: Arc<Mutex<std::fs::File>>) {
    let mut buffer = [0; 512];

    // Ler a requisição HTTP
    if let Ok(_) = stream.read(&mut buffer) {
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
            "HTTP/1.1 200 OK\r\n\
             Content-Type: image/gif\r\n\
             Content-Length: {}\r\n\
             Connection: close\r\n\r\n",
            pixel_data().len()
        );

        // Enviar o cabeçalho e o conteúdo do pixel
        let _ = stream.write_all(response.as_bytes());
        let _ = stream.write_all(pixel_data());
        let _ = stream.flush(); // Certifique-se de que todos os dados são enviados corretamente
    }

    // Fecha o stream para evitar problemas com HTTP/2
    let _ = stream.shutdown(std::net::Shutdown::Both);
}

fn main() {
    // Mude a porta para 8080, pois Fly.io usa essa porta
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
