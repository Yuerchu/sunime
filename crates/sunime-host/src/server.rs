use sunime_ipc::pipe::IpcServer;
use sunime_ipc::messages::*;

use crate::engine::ImeState;

pub fn run(engine: sunime_core::Engine) {
    let mut state = ImeState::new(engine);

    let server = IpcServer::bind().unwrap_or_else(|e| {
        eprintln!("Failed to bind: {e}");
        std::process::exit(1);
    });

    println!("Listening on {}", server.addr());

    loop {
        let Ok(mut conn) = server.accept() else {
            continue;
        };

        loop {
            let Ok(request) = conn.read_request() else {
                break;
            };

            let response = match request {
                Request::Activate => {
                    println!("  IME activated");
                    Response::Ok
                }
                Request::Deactivate => {
                    println!("  IME deactivated");
                    Response::Ok
                }
                Request::OnTestKeyDown(params) => {
                    let handled = state.on_test_key_down(&params);
                    Response::TestKeyReply { handled }
                }
                Request::OnKeyDown(params) => state.on_key_down(&params),
                Request::OnTestKeyUp(_) => Response::TestKeyReply { handled: false },
                Request::OnKeyUp(_) => Response::KeyReply {
                    handled: false,
                    commit: None,
                    preedit: None,
                    candidates: None,
                },
            };

            if conn.write_response(&response).is_err() {
                break;
            }
        }
    }
}
