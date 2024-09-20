use std::io::{ Read, Write };
use std::net::{ TcpListener, TcpStream };
use std::thread;
use std::time::{ SystemTime, UNIX_EPOCH };
use std::env;
use rusqlite::{ params, Connection };

fn pixel_data() -> &'static [u8] {
    &[
        0x47, 0x49, 0x46, 0x38, 0x39, 0x61, 0x01, 0x00, 0x01, 0x00, 0x80, 0xff, 0x00, 0xff, 0xff, 0xff,
        0x00, 0x00, 0x00, 0x21, 0xf9, 0x04, 0x01, 0x00, 0x00, 0x00, 0x00, 0x2c, 0x00, 0x00, 0x00, 0x00,
        0x01, 0x00, 0x01, 0x00, 0x00, 0x02, 0x02, 0x44, 0x01, 0x00, 0x3b,
    ]
}

fn get_database_path() -> String {
    if cfg!(debug_assertions) {
        env::var("DATABASE_PATH").unwrap_or_else(|_| "data/access_log.db".to_string())
    } else {
        env::var("DATABASE_PATH").unwrap_or_else(|_| "/mnt/data/access_log.db".to_string())
    }
}

fn handle_client(mut stream: TcpStream) {
    let db_path = get_database_path();
    let mut buffer = [0; 512];

    // Read the HTTP request
    if let Ok(_) = stream.read(&mut buffer) {
        // Log the access
        let ip = stream.peer_addr().unwrap().to_string();
        let time = SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_secs();

        // Open a connection to the database
        let conn = Connection::open(&db_path).unwrap();

        // Insert the log entry into the database
        conn.execute(
            "INSERT INTO access_log (ip, time) VALUES (?1, ?2)",
            params![ip, time]
        ).unwrap();

        // Create the HTTP response to serve the invisible pixel
        let response = format!(
            "HTTP/1.1 200 OK\r\n\
             Content-Type: image/gif\r\n\
             Content-Length: {}\r\n\
             Cache-Control: no-store, must-revalidate\r\n\
             Pragma: no-cache\r\n\
             Expires: 0\r\n\
             Connection: close\r\n\r\n",
            pixel_data().len()
        );

        // Send the headers and the pixel data
        let _ = stream.write_all(response.as_bytes());
        let _ = stream.write_all(pixel_data());
        let _ = stream.flush(); // Ensure all data is sent correctly
    }

    // Close the stream to avoid issues with HTTP/2
    let _ = stream.shutdown(std::net::Shutdown::Both);
}

fn main() {
    // Get the port from the environment variable or default to 8080
    let port = env::var("PORT").unwrap_or_else(|_| "8080".to_string());

    let listener = TcpListener::bind(format!("0.0.0.0:{}", port)).unwrap();
    println!("Server running on port {}", port);

    let db_path = get_database_path();

    // Ensure the directory for the database exists
    let db_dir = std::path::Path::new(&db_path).parent().expect("Failed to get database directory");
    std::fs::create_dir_all(db_dir).expect("Failed to create database directory");

    // Open the database and create the table if it doesn't exist
    let conn = Connection::open(&db_path).unwrap();
    conn.execute(
        "CREATE TABLE IF NOT EXISTS access_log (
            id INTEGER PRIMARY KEY,
            ip TEXT NOT NULL,
            time INTEGER NOT NULL
        )",
        []
    ).unwrap();

    // Accept connections and handle them in separate threads
    for stream in listener.incoming() {
        let stream = stream.unwrap();

        thread::spawn(move || {
            handle_client(stream);
        });
    }
}
