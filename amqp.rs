#![feature(globs)]
#![crate_type="lib"]
#![crate_id="amqp#1.0"]
#![license = "BSD"]
#![allow(non_camel_case_types)]

extern crate std;
extern crate libc;
use std::cast;

mod rabbitmq;

pub static AMQP_SASL_METHOD_PLAIN: u32 = rabbitmq::AMQP_SASL_METHOD_PLAIN;
pub static AMQP_REPLY_SUCCESS: i32 = 200;

pub enum SocketType {
  TcpSocket
}

enum ConnectionState {
  ConnectionOpen,
  ConnectionClosed
}

pub struct Connection{
  state: rabbitmq::amqp_connection_state_t,
  socket: *mut rabbitmq::amqp_socket_t,
  connection_state: ConnectionState
}

pub struct Channel {
  pub id: u16
}

#[unsafe_destructor]
impl std::ops::Drop for Connection {
  fn drop(&mut self) {
    self.connection_close(AMQP_REPLY_SUCCESS);
  }
}

impl Connection {
  pub fn new(socket_type: SocketType) -> Result<~Connection, ~str> {

    fn new_connection() -> Option<rabbitmq::amqp_connection_state_t> {
      unsafe {
        match rabbitmq::amqp_new_connection(){
          ptr @ _ if !ptr.is_null() => Some(ptr),
          // ptr @ _ if ptr.to_uint() == 0 => None,
          _ => None
        }
      }
    }

    fn tcp_socket_new(state: rabbitmq::amqp_connection_state_t) -> Option<*mut rabbitmq::amqp_socket_t> {
      unsafe {
        match rabbitmq::amqp_tcp_socket_new(state){
          ptr @ _ if !ptr.is_null() => Some(ptr),
          _ => None
        }
      }
    }

    let state = match new_connection() {
      Some(s) => s,
      None => return Err(~"Error allocating new connection")
    };
    let socket = match socket_type{
      TcpSocket => match tcp_socket_new(state){
        Some(s) => s,
        None => return Err(~"Error creating socket")
      }
    };
    Ok(~Connection { state: state, socket: socket, connection_state: ConnectionClosed })
  }

  pub fn socket_open(&mut self, hostname: ~str, port: uint) -> Result<(), (~str, i32)> {
    unsafe {
      match rabbitmq::amqp_socket_open(self.socket, hostname.to_c_str().unwrap(), port as i32){
        0 => { self.connection_state = ConnectionOpen; Ok(()) },
        code @ _ => Err((error_string(code), code))
      }
    }
  }

  pub fn login(&self, vhost: ~str, channel_max: int, frame_max: int, heartbeat: int,
             sasl_method: rabbitmq::amqp_sasl_method_enum, login: ~str, password: ~str) -> rabbitmq::amqp_rpc_reply_t {
    unsafe {
      rabbitmq::amqp_login(self.state, vhost.to_c_str().unwrap(), channel_max as i32, frame_max as i32, heartbeat as i32, sasl_method,
                           login.to_c_str().unwrap(), password.to_c_str().unwrap())
    }
  }

  pub fn channel_open(&self, channel: u16) -> Channel {
    unsafe {
      let response = rabbitmq::amqp_channel_open(self.state, channel);
      Channel{id: channel}
    }
  }

  pub fn channel_close(&self, channel: Channel, code: i32) {
    unsafe {
      rabbitmq::amqp_channel_close(self.state, channel.id, code);
    }
  }

  pub fn connection_close(&mut self, code: i32) -> Option<rabbitmq::amqp_rpc_reply_t> {
    match self.connection_state {
      ConnectionOpen => {
        unsafe {
          self.connection_state = ConnectionClosed;
          Some(rabbitmq::amqp_connection_close(self.state, code))
        }
      },
      ConnectionClosed => None
    }
  }

  pub fn get_rpc_reply(&self) -> rabbitmq::amqp_rpc_reply_t {
    unsafe {
      rabbitmq::amqp_get_rpc_reply(self.state)
    }
  }
}


// top level
pub fn version() -> ~str {
  unsafe {
	  return std::str::raw::from_c_str(rabbitmq::amqp_version());
	}
}

pub fn version_number() -> uint {
  unsafe {
    return rabbitmq::amqp_version_number() as uint;
  }
}

fn error_string(error: i32) -> ~str {
  unsafe {
    return std::str::raw::from_c_str(rabbitmq::amqp_error_string2(error));
  }
}