LIB_RABBITMQ_LIB_PATH='/Users/andrii/dev/rabbitmq-c/build/librabbitmq'
LIB_RABBITMQ_PATH='/Users/andrii/dev/rabbitmq-c/librabbitmq'

desc 'generate bindings'
task :bindings do
  sh "./bindgen -builtins -l rabbitmq -I#{LIB_RABBITMQ_PATH} -o rabbitmq.rs gen.h"
end

desc 'generate crate'
file 'libamqp-288d5d4c-1.0.rlib' => ['rabbitmq.rs', 'amqp.rs'] do
  sh "rustc amqp.rs"
end

desc 'build test prog'
file 'hello' => ['hello.rs', 'libamqp-288d5d4c-1.0.rlib'] do |task|
  sh "rustc -L . -L #{LIB_RABBITMQ_LIB_PATH} hello.rs"
end


task :default => ['hello']
