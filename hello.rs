extern crate amqp;
extern crate libc;

fn main(){
  println!("librabbitmq-c {}", amqp::version());
  let mut con = amqp::Connection::new(amqp::TcpSocket).unwrap();
  let result = con.socket_open(~"localhost", None);

  if result.is_err() {
  	let errno = std::os::errno();
  	unsafe {
  	  println!("{}", std::str::raw::from_c_str(libc::funcs::c95::string::strerror(errno as i32)));
  	}
  	fail!("Error openning socket: '{}', errno: {}", result.unwrap_err(), errno);
  } else{
  	println!("Connected to RabbitMQ");
  }
  let log = con.login(~"/", 0, None, 0, amqp::AMQP_SASL_METHOD_PLAIN, ~"guest", ~"guest");
  if log.is_ok() {
  	println!("Logged in");
  }else{
    println!("Error loggin in: {}", log);
  }
  let chan = con.channel_open(1).unwrap();
  // let mut table = amqp::amqp_table { entries: ~[] };
  // table.add_entry(~"hi", 1 as u32);
  let queue = con.queue_declare(chan, ~"testq123", false, false, false, false, None);
  println!("{}", queue);

  let properties = amqp::amqp_basic_properties { _flags: (1 << 15) , content_type: ~"text/plain", delivery_mode: 1, ..std::default::Default::default()};
  con.basic_publish(chan, ~"", ~"testq123", false, false, Some(properties), (~"xxxhello from rust!").into_bytes());
  con.basic_consume(chan, ~"", ~"testq123", false, false, false, None);
  loop {
    let msg = con.consume_message(None, None);
    match msg {
      Ok(msg) => println!("{}", msg.str_body()),
      Err(msg) => println!("Error consuming message: {}", msg)
    }
  }

  con.channel_close(chan, amqp::AMQP_REPLY_SUCCESS);
  con.connection_close(amqp::AMQP_REPLY_SUCCESS);
}