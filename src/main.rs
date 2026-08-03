use std::io::Read;
use std::io::Write;
use std::net::TcpListener;
use std::net::TcpStream;
use std::sync::{LazyLock, Mutex};

static MAP: LazyLock<Mutex<[[Symbols; 3]; 3]>> = LazyLock::new(|| {
    Mutex::new([
        [Symbols::NONE, Symbols::NONE, Symbols::NONE],
        [Symbols::NONE, Symbols::NONE, Symbols::NONE],
        [Symbols::NONE, Symbols::NONE, Symbols::NONE],
    ])
});

static FIRSTPALAYER: LazyLock<Mutex<bool>> = LazyLock::new(|| Mutex::new(true)); // if true its X turn if False its Y turn

#[derive(PartialEq, Clone, Copy, Debug)]
enum Symbols {
    X,
    Y,
    NONE,
}

fn find_winner(map: &[[Symbols; 3]; 3]) -> Symbols {
    //check for diagonals
    if map[0][0] == map[1][1] && map[1][1] == map[2][2] {
        if map[0][0] != Symbols::NONE {
            return map[0][0];
        }
    }
    if map[2][0] == map[1][1] && map[1][1] == map[0][2] {
        if map[2][0] != Symbols::NONE {
            return map[2][0];
        }
    }

    //check for rows
    for i in 0..3 {
        if map[i][0] == map[i][1] && map[i][1] == map[i][2] {
            if map[i][0] != Symbols::NONE {
                return map[i][0];
            }
        }
        if map[0][i] == map[1][i] && map[1][i] == map[2][i] {
            if map[0][i] != Symbols::NONE {
                return map[0][i];
            }
        }
    }

    Symbols::NONE
}

fn display() -> Vec<u8> {
    let map: [[Symbols; 3]; 3];
    {
        let maap = MAP.lock().unwrap().clone();
        map = maap.clone();
    }
    let weird_string = format!(
        "{:?} {:?} {:?}\n
        {:?} {:?} {:?}\n
        {:?} {:?} {:?}\n",
        map[0][0],
        map[0][1],
        map[0][2],
        map[1][0],
        map[1][1],
        map[1][2],
        map[2][0],
        map[2][1],
        map[2][2]
    );
    let weird_u8 = weird_string.as_bytes().to_vec();
    weird_u8
}

fn player_handler(mut stram: TcpStream) {
    let mut message: Vec<u8> = Vec::new();
    let curent_map: Vec<u8> = display();
    stram.write_all(&curent_map).unwrap();

    let mut buffer: Vec<u8> = Vec::new();
    match stram.read_to_end(&mut buffer) {
        Ok(_) => {
            // the message that goes to player

            let s = String::from_utf8(buffer).unwrap();
            println!("mesasge from user{}", s);
            let chr: Vec<char> = s.chars().collect();

            let player_symbol = chr[0];
            let pos_y: i32 = chr[1].to_string().parse().unwrap();
            let pos_x: i32 = chr[2].to_string().parse().unwrap();

            let player_symbol_as_bool: bool;
            if player_symbol == 'X' {
                player_symbol_as_bool = true;
            } else {
                player_symbol_as_bool = false;
            }

            {
                let firstplayer = FIRSTPALAYER.lock().unwrap();
                let first = firstplayer.clone();
                if player_symbol_as_bool != first {
                    message = "skip".as_bytes().to_vec();
                    stram.write_all(&message).unwrap();
                }
            }

            let player_symbol_as_symbol: Symbols;
            if player_symbol == 'X' {
                player_symbol_as_symbol = Symbols::X;
            } else {
                player_symbol_as_symbol = Symbols::Y;
            }

            let valid: bool = false;
            let map: [[Symbols; 3]; 3];
            {
                let maap = MAP.lock().unwrap().clone();
                map = maap.clone();
            }
            if map[pos_y as usize][pos_x as usize] != Symbols::NONE {
                {
                    let mut maap = MAP.lock().unwrap().clone();
                    maap[pos_y as usize][pos_x as usize] = player_symbol_as_symbol;
                }
                {
                    let mut firstplayer = FIRSTPALAYER.lock().unwrap();
                    *firstplayer = !firstplayer.clone();
                }
                message = "valid".as_bytes().to_vec();
            }
            //send for resend
            if !valid {
                message = "resend".as_bytes().to_vec();
            }
        }
        Err(err) => eprintln!("error while reading: {}", err),
    }
    stram.write_all(&message).unwrap();
}

fn main() -> std::io::Result<()> {
    let listener = TcpListener::bind("0.0.0.0:6767")?;
    for stream in listener.incoming() {
        player_handler(stream?);
    }
    Ok(())
}
