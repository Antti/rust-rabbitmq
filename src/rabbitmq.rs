#![feature(globs)]
#![crate_type="lib"]
#![crate_id="rabbitmq#1.0"]
#![license = "BSD"]
#![allow(non_camel_case_types)]

extern crate std;
extern crate libc;
use std::bool;
use std::cast;

pub mod rabbitmqc; //bindings

pub static AMQP_SASL_METHOD_PLAIN: u32 = rabbitmqc::AMQP_SASL_METHOD_PLAIN;
pub static AMQP_REPLY_SUCCESS: i32 = 200;

pub type amqp_rpc_reply = rabbitmqc::amqp_rpc_reply_t;

pub enum AMQPMethod {
  AMQP_QUEUE_DECLARE_METHOD = 0x0032000A,
  AMQP_QUEUE_DECLARE_OK_METHOD = 0x0032000B,
  AMQP_QUEUE_BIND_METHOD = 0x00320014,
  AMQP_QUEUE_BIND_OK_METHOD = 0x00320015,
  AMQP_CONNECTION_CLOSE_METHOD = 0x000A0032,
  AMQP_CONNECTION_CLOSE_OK_METHOD = 0x000A0033,
  AMQP_CHANNEL_CLOSE_METHOD = 0x00140028,
  AMQP_CHANNEL_CLOSE_OK_METHOD = 0x00140029,
}

trait TableField {
  fn value(&self) -> [u64, ..2u];
  fn kind(&self) -> u8;
  fn to_rabbit(&self) -> rabbitmqc::amqp_field_value_t {
    rabbitmqc::Struct_amqp_field_value_t_ { value: unsafe { cast::transmute(self.value()) }, kind: self.kind() }
  }
}

impl TableField for u32 {
  fn value(&self) -> [u64, ..2u] {
    [*self as u64, 0]
  }
  fn kind(&self) -> u8 {
    'I' as u8
  }
}

#[deriving(Show)]
pub struct amqp_message {
  pub body: Vec<u8>,
}

#[deriving(Show)]
pub struct amqp_queue_declare_ok {
  pub queue: ~str,
  pub message_count: u32,
  pub consumer_count: u32,
}

#[deriving(Show)]
pub struct amqp_queue_purge {
  pub ticket: u16,
  pub queue: ~str,
  pub nowait: bool
}

#[deriving(Default)]
pub struct amqp_table {
    pub entries: Vec<rabbitmqc::Struct_amqp_table_entry_t_>
}

#[deriving(Default)]
pub struct amqp_basic_properties {
    pub _flags: u32,
    pub content_type: ~str,
    pub content_encoding: ~str,
    pub headers: amqp_table,
    pub delivery_mode: u8,
    pub priority: u8,
    pub correlation_id: ~str,
    pub reply_to: ~str,
    pub expiration: ~str,
    pub message_id: ~str,
    pub timestamp: u64,
    pub _type: ~str,
    pub user_id: ~str,
    pub app_id: ~str,
    pub cluster_id: ~str,
}

impl amqp_message {
  pub fn str_body<'a>(&'a self) -> Option<&'a str> {
    std::str::from_utf8(self.body.as_slice())
  }
}

impl amqp_table {
  fn to_rabbit(&self) -> rabbitmqc::Struct_amqp_table_t_ {
    unsafe {
      rabbitmqc::Struct_amqp_table_t_ { num_entries: self.entries.len() as i32, entries: cast::transmute::<_,*mut rabbitmqc::amqp_table_entry_t>(self.entries.as_ptr())}
    }
  }

  pub fn add_entry<T: TableField>(&mut self, key: ~str, value: T) {
    self.entries.push(rabbitmqc::Struct_amqp_table_entry_t_ { key: str_to_amqp_bytes(key), value: unsafe { cast::transmute(value.to_rabbit()) } } )
  }
}

impl amqp_basic_properties {
  fn to_rabbit(&self) -> rabbitmqc::Struct_amqp_basic_properties_t_ {
    // let flags = 0; TODO: Calculate flags dynamicly, maybe by having all flags as Option<T>
    rabbitmqc::Struct_amqp_basic_properties_t_ { _flags: self._flags, content_type: str_to_amqp_bytes(self.content_type.clone()),
      content_encoding: str_to_amqp_bytes(self.content_encoding.clone()),
      headers: self.headers.to_rabbit(), delivery_mode: self.delivery_mode, priority: self.priority, correlation_id: str_to_amqp_bytes(self.correlation_id.clone()),
      reply_to: str_to_amqp_bytes(self.reply_to.clone()), expiration: str_to_amqp_bytes(self.expiration.clone()), message_id: str_to_amqp_bytes(self.message_id.clone()),
      timestamp: self.timestamp, _type: str_to_amqp_bytes(self._type.clone()), user_id: str_to_amqp_bytes(self.user_id.clone()),
      app_id: str_to_amqp_bytes(self.app_id.clone()), cluster_id: str_to_amqp_bytes(self.cluster_id.clone())
     }
  }
}

pub enum SocketType {
  TcpSocket
}

enum ConnectionState {
  ConnectionOpen,
  ConnectionClosed
}

pub struct Connection{
  state: rabbitmqc::amqp_connection_state_t,
  connection_state: ConnectionState
}

pub struct Channel {
  pub id: u16
}

#[unsafe_destructor]
impl std::ops::Drop for Connection {
  fn drop(&mut self) {
    self.connection_close(AMQP_REPLY_SUCCESS);
    unsafe{ rabbitmqc::amqp_destroy_connection(self.state) };
  }
}

impl Connection {
  pub fn new(socket_type: SocketType) -> Result<~Connection, ~str> {

    let state = match unsafe { rabbitmqc::amqp_new_connection() } {
      ptr if !ptr.is_null() => ptr,
      _ => return Err("Error allocating new connection".to_owned())
    };

    match socket_type{
      TcpSocket => match unsafe { rabbitmqc::amqp_tcp_socket_new(state) }{
        ptr if !ptr.is_null() => ptr,
        _ => { return Err("Error creating socket".to_owned())}
      }
    };

    Ok(~Connection { state: state, connection_state: ConnectionClosed })
  }

  pub fn socket_open(&mut self, hostname: ~str, port: Option<uint>) -> Result<(), (~str, i32)> {
    unsafe {
      match rabbitmqc::amqp_socket_open((*self.state).socket, hostname.to_c_str().unwrap(), port.unwrap_or(5672) as i32){
        0 => { self.connection_state = ConnectionOpen; Ok(()) },
        code => Err((error_string(code), code))
      }
    }
  }

  pub fn login(&self, vhost: &str, channel_max: int, frame_max: Option<int>, heartbeat: int,
             sasl_method: rabbitmqc::amqp_sasl_method_enum, login: &str, password: &str) -> Result<(),~str> {
    unsafe {
      let reply = rabbitmqc::amqp_login(self.state, vhost.to_c_str().unwrap(), channel_max as i32, frame_max.unwrap_or(131072) as i32, heartbeat as i32, sasl_method,
                           login.to_c_str().unwrap(), password.to_c_str().unwrap());
      match reply.reply_type {
        rabbitmqc::AMQP_RESPONSE_NORMAL => Ok(()),
        _ => Err(reply_to_error(reply))
      }
    }
  }

  pub fn channel_open(&self, channel: u16) -> Option<Channel> {
    unsafe {
      let response = rabbitmqc::amqp_channel_open(self.state, channel);
      if response.is_null(){
        None
      } else {
        Some(Channel{id: channel})
      }
    }
  }

  pub fn channel_close(&self, channel: Channel, code: i32) {
    unsafe {
      rabbitmqc::amqp_channel_close(self.state, channel.id, code);
    }
  }

  pub fn simple_rpc(&self, channel: Channel, request_id: AMQPMethod, reply_id: AMQPMethod, decoded_request_method: *mut libc::c_void) -> amqp_rpc_reply {
    let expected_reply_ids = ~[reply_id as u32, 0];
    unsafe {
      rabbitmqc::amqp_simple_rpc(self.state, channel.id, request_id as u32, cast::transmute(&expected_reply_ids), decoded_request_method)
    }
  }

  pub fn queue_declare(&self, channel: Channel, queue: &str,  passive: bool, durable: bool, exclusive: bool, auto_delete: bool, arguments: Option<amqp_table>) -> Result<amqp_queue_declare_ok, ~str> {
    unsafe {
      let args = match arguments{
        Some(args) => args.to_rabbit(),
        None => (amqp_table{entries: vec!() }).to_rabbit()
      };

      let req = rabbitmqc::Struct_amqp_queue_declare_t_{
        ticket :      0,
        queue :       str_to_amqp_bytes(queue),
        passive :     bool::to_bit::<i32>(passive),
        durable :     bool::to_bit::<i32>(durable),
        exclusive :   bool::to_bit::<i32>(exclusive),
        auto_delete : bool::to_bit::<i32>(auto_delete),
        nowait :      0,
        arguments :   args,
      };

      let response = self.simple_rpc(channel, AMQP_QUEUE_DECLARE_METHOD, AMQP_QUEUE_DECLARE_OK_METHOD, cast::transmute(&req));
      if response.reply_type == rabbitmqc::AMQP_RESPONSE_NORMAL {
        let reply : &rabbitmqc::Struct_amqp_queue_declare_ok_t_ = cast::transmute(response.reply.decoded);
        Ok(amqp_queue_declare_ok { queue: amqp_bytes_to_str(reply.queue), message_count: reply.message_count, consumer_count: reply.consumer_count })
      }else{
        Err(reply_to_error(response))
      }
    }
  }

  pub fn queue_bind(&self, channel: Channel, queue: &str, exchange: &str, routing_key: &str, arguments: Option<amqp_table>) -> Result<(), ~str> {
    unsafe {
      let args = match arguments{
        Some(args) => args.to_rabbit(),
        None => (amqp_table{entries: vec!() }).to_rabbit()
      };
      let req = rabbitmqc::Struct_amqp_queue_bind_t_ {
        ticket: 0,
        queue: str_to_amqp_bytes(queue),
        exchange: str_to_amqp_bytes(exchange),
        routing_key: str_to_amqp_bytes(routing_key),
        nowait: 0,
        arguments: args,
      };
      let response = self.simple_rpc(channel, AMQP_QUEUE_DECLARE_METHOD, AMQP_QUEUE_DECLARE_OK_METHOD, cast::transmute(&req));
      if response.reply_type == rabbitmqc::AMQP_RESPONSE_NORMAL{
        Ok(())
      }else{
        Err(reply_to_error(response))
      }
    }
  }

  pub fn basic_publish(&self, channel: Channel, exchange: &str, routing_key: &str, mandatory: bool, immediate: bool, properties: Option<amqp_basic_properties>, body: Vec<u8>) -> i32 {
    unsafe{
      let props = match properties {
        Some(prop) => cast::transmute(&prop.to_rabbit()),
        None => std::ptr::null::<rabbitmqc::amqp_basic_properties_t>()
      };
      rabbitmqc::amqp_basic_publish(self.state, channel.id, str_to_amqp_bytes(exchange), str_to_amqp_bytes(routing_key), bool::to_bit::<i32>(mandatory), bool::to_bit::<i32>(immediate), props, vec_to_amqp_bytes(&body))
    }
  }

  pub fn basic_consume(&self, channel: Channel, queue: &str, consumer_tag: &str, no_local: bool, no_ack: bool, exclusive: bool, arguments: Option<amqp_table>) -> *mut rabbitmqc::amqp_basic_consume_ok_t{
    unsafe {
      let args = match arguments{
        Some(args) => args.to_rabbit(),
        None => (amqp_table{entries: vec!() }).to_rabbit()
      };
      rabbitmqc::amqp_basic_consume(self.state, channel.id, str_to_amqp_bytes(queue), str_to_amqp_bytes(consumer_tag),
        bool::to_bit::<i32>(no_local), bool::to_bit::<i32>(no_ack), bool::to_bit::<i32>(exclusive), args)
    }
  }

  pub fn consume_message(&self, timeout: Option<rabbitmqc::Struct_timeval>, flags: Option<int>) -> Result<amqp_message, ~str> {
    unsafe {
      let to : *mut rabbitmqc::Struct_timeval = match timeout{
        Some(to) => cast::transmute(&to),
        None => cast::transmute(std::ptr::null::<rabbitmqc::Struct_timeval>())
      };
      let mut envelope = Vec::with_capacity(std::mem::size_of::<rabbitmqc::Struct_amqp_envelope_t_>());
      let penvelope  = envelope.as_mut_ptr();
      let reply = rabbitmqc::amqp_consume_message(self.state, penvelope, to, flags.unwrap_or(0) as i32);
      if reply.reply_type == rabbitmqc::AMQP_RESPONSE_NORMAL {
        let msg = amqp_message { body: amqp_bytes_to_vec((*penvelope).message.body) };
        destroy_envelope(penvelope);
        Ok(msg)
      } else {
        destroy_envelope(penvelope);
        Err(reply_to_error(reply))
      }
    }
  }

  pub fn connection_close(&mut self, code: i32) -> Option<amqp_rpc_reply> {
    match self.connection_state {
      ConnectionOpen => {
        unsafe {
          self.connection_state = ConnectionClosed;
          Some(rabbitmqc::amqp_connection_close(self.state, code))
        }
      },
      ConnectionClosed => None
    }
  }

  #[allow(dead_code)]
  fn maybe_release_buffers(&self) {
    unsafe {
      rabbitmqc::amqp_maybe_release_buffers(self.state);
    }
  }

}


// top level
pub fn version() -> ~str {
  unsafe {
	  return std::str::raw::from_c_str(rabbitmqc::amqp_version());
	}
}

pub fn version_number() -> uint {
  unsafe {
    return rabbitmqc::amqp_version_number() as uint;
  }
}

pub fn destroy_envelope(envelope: *mut rabbitmqc::amqp_envelope_t){
  unsafe {
    rabbitmqc::amqp_destroy_envelope(envelope);
  }
}

fn str_to_amqp_bytes(string: &str) -> rabbitmqc::amqp_bytes_t {
  unsafe {
    rabbitmqc::Struct_amqp_bytes_t_ { len: string.len() as u64, bytes: std::cast::transmute(string.to_c_str().unwrap()) }
  }
}

fn vec_to_amqp_bytes(vec: &Vec<u8>) -> rabbitmqc::amqp_bytes_t {
  unsafe {
    rabbitmqc::Struct_amqp_bytes_t_ { len: vec.len() as u64, bytes: std::cast::transmute(vec.as_ptr()) }
  }
}

fn amqp_bytes_to_str(bytes: rabbitmqc::amqp_bytes_t) -> ~str {
  unsafe {
    std::str::raw::from_buf_len(cast::transmute(bytes.bytes), bytes.len as uint)
  }
}
fn amqp_bytes_to_vec(bytes: rabbitmqc::amqp_bytes_t) -> Vec<u8> {
  unsafe {
    Vec::from_slice(std::slice::raw::from_buf_raw::<u8>(cast::transmute(bytes.bytes), bytes.len as uint))
  }
}

fn error_string(error: i32) -> ~str {
  unsafe {
    return std::str::raw::from_c_str(rabbitmqc::amqp_error_string2(error));
  }
}

fn reply_to_error(reply: rabbitmqc::amqp_rpc_reply_t) -> ~str {
  match reply.reply_type {
    rabbitmqc::AMQP_RESPONSE_NONE => "Missing RPC reply type".to_owned(),
    rabbitmqc::AMQP_RESPONSE_LIBRARY_EXCEPTION => error_string(reply.library_error),
    rabbitmqc::AMQP_RESPONSE_SERVER_EXCEPTION => match reply.reply.id {
      q if q == AMQP_CONNECTION_CLOSE_METHOD as u32 => "server connection error".to_owned(),
      q if q == AMQP_CHANNEL_CLOSE_METHOD as u32 => "server channel error".to_owned(),
      _ => format!("Unknown server error, method id {}", reply.reply.id)
    },
    rabbitmqc::AMQP_RESPONSE_NORMAL => "No error".to_owned(),
    _ => "Unknown reply_type".to_owned()
  }
}
