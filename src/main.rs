extern crate libc;
use libc::{c_int, c_long, ftok, msgctl, msgget, msgrcv, msgsnd, IPC_CREAT, IPC_RMID};
use std::ffi::CString;
use std::ptr;

const MAX_MSG_SIZE: usize = 1024;

#[repr(C)]
struct MsgBuffer {
    msg_type: c_long,
    msg_text: [i8; MAX_MSG_SIZE],
}

fn generate_key() -> c_int {
    let key_path = CString::new("msg_queue_key").expect("CString::new failed");
    let key = unsafe { ftok(key_path.as_ptr(), 'b' as i32) };
    if key == -1 {
        let err = std::io::Error::last_os_error();
        eprintln!("Error generating key: {}", err);
    }
    key
}

fn create_or_get_message_queue() -> c_int {
    let key = generate_key();
    if key == -1 {
        panic!("ftok failed");
    }
    
    let msg_id = unsafe { msgget(key, IPC_CREAT | 0o666) };
    if msg_id == -1 {
        panic!("msgget failed");
    }
    
    msg_id
}

fn send_message(msg_id: c_int, message: &str) {
    let mut msg: MsgBuffer = MsgBuffer {
        msg_type: 1,
        msg_text: [0; MAX_MSG_SIZE],
    };

    for (i, byte) in message.bytes().enumerate() {
        msg.msg_text[i] = byte as i8;
    }

    if unsafe { msgsnd(msg_id, &msg as *const _ as *mut _, MAX_MSG_SIZE as usize, 0) } == -1 {
        panic!("msgsnd failed");
    }

    println!("Message sent: {}", message);
}

fn receive_message(msg_id: c_int) -> String {
    let mut msg: MsgBuffer = MsgBuffer {
        msg_type: 1,
        msg_text: [0; MAX_MSG_SIZE],
    };

    if unsafe { msgrcv(msg_id, &mut msg as *mut _ as *mut _, MAX_MSG_SIZE as usize, 1, 0) } == -1 {
        panic!("msgrcv failed");
    }

    let received_message = msg.msg_text.iter()
        .take_while(|&&byte| byte != 0)
        .map(|&byte| byte as u8)
        .collect::<Vec<u8>>();

    String::from_utf8(received_message).expect("Received invalid UTF-8")
}

fn remove_message_queue(msg_id: c_int) {
    if unsafe { msgctl(msg_id, IPC_RMID, ptr::null_mut()) } == -1 {
        panic!("msgctl failed");
    }
}

fn main() {
    let msg_id = create_or_get_message_queue();

    if unsafe { libc::fork() } == 0 {
        let message = "Hello from sender!";
        send_message(msg_id, message);
    } else {
        let received_message = receive_message(msg_id);
        println!("Message received: {}", received_message);
        remove_message_queue(msg_id);
    }
}

