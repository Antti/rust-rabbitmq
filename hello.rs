extern crate amqp;
extern crate libc;

fn main(){
  println!("librabbitmq-c {}", amqp::version());
  let mut con = amqp::Connection::new(amqp::TcpSocket).unwrap();
  let result = con.socket_open(~"localhost", 5672);
  if result.is_err() {
  	let errno = std::os::errno();
  	unsafe {
  	  println!("{}", std::str::raw::from_c_str(libc::funcs::c95::string::strerror(errno as i32)));
  	}
  	fail!("Error openning socket: '{}', errno: {}", result.unwrap_err(), errno);
  } else{
  	println!("Connected to RabbitMQ");
  }
  let log = con.login(~"/", 0, 131072, 0, amqp::AMQP_SASL_METHOD_PLAIN, ~"guest", ~"guest");
  if log.is_ok() {
  	println!("Logged in");
  }else{
    println!("Error loggin in: {}", log);
  }
  let chan = con.channel_open(1).unwrap();
  let queue = con.queue_declare(chan, ~"testq123", false, false, false, false);
  println!("{}", queue);

  con.channel_close(chan, amqp::AMQP_REPLY_SUCCESS);
  con.connection_close(amqp::AMQP_REPLY_SUCCESS);
}