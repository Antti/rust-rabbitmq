extern crate rabbitmq;
extern crate libc;

fn main(){
  println!("librabbitmq-c {}", rabbitmq::version());
  let mut con = rabbitmq::Connection::new(rabbitmq::TcpSocket).unwrap();
  let result = con.socket_open("localhost", None);

  if result.is_err() {
  	let io_msg;
    unsafe {
      let errno = std::os::errno();
  	  io_msg = format!("{}", std::str::raw::from_c_str(libc::funcs::c95::string::strerror(errno as i32)));
  	};
  	fail!("Error openning socket: '{}', errno: {}", result.unwrap_err(), io_msg);
  } else{
  	println!("Connected to RabbitMQ");
  }
  let log = con.login("/", 0, None, 0, rabbitmq::AMQP_SASL_METHOD_PLAIN, "guest", "guest");
  if log.is_ok() {
  	println!("Logged in");
  }else{
    println!("Error loggin in: {}", log);
  }
  let chan = con.channel_open(1).unwrap();
  // let mut table = rabbitmq::amqp_table { entries: vec!() };
  // table.add_entry("hi", 1 as u32);
  let queue = con.queue_declare(chan, "testq123", false, false, false, false, None);
  println!("{}", queue);

  let properties = rabbitmq::amqp_basic_properties { _flags: (1 << 15) , content_type: "text/plain".to_string(), delivery_mode: 1, ..std::default::Default::default()};
  con.basic_publish(chan, "", "testq123", false, false, Some(properties), Vec::from_slice(bytes!("xxxhello from rust!")));
  con.basic_consume(chan, "", "testq123", false, false, false, None);

  let continue_consuming = true;

  while continue_consuming {
    let to = rabbitmq::rabbitmqc::Struct_timeval {tv_sec: 1, tv_usec: 0};
    let msg = con.consume_message(Some(to), None);
    match msg {
      Ok(msg) => println!("{}", msg.str_body()),
      Err(msg) => println!("Error consuming message: {}", msg)
    }
  }
  con.channel_close(chan, rabbitmq::AMQP_REPLY_SUCCESS);
  con.connection_close(rabbitmq::AMQP_REPLY_SUCCESS);

}